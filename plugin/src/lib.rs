//! bibfmt — proof-of-concept Typst WASM plugin.
//!
//! Idea under test: do the *whole* bibliography rendering (parse + GB/T 7714
//! 著录) in Rust, and hand Typst back ready-to-print, rich-text reference
//! entries — bypassing hayagriva's CSL renderer entirely.
//!
//! Exported function `format_refs(bib, keys)`:
//!   - `bib`  : raw .bib (BibLaTeX) content, UTF-8
//!   - `keys` : cited keys IN CITATION ORDER, separated by `\n` or `,`.
//!              Empty => render every entry in file order.
//! Returns CBOR: `[[Run, ...], ...]` — one list of styled runs per entry, in
//! the same order as `keys`. A `Run` is `{k, t, u?}`:
//!   - k = "text" | "emph" | "link"
//!   - t = display text
//!   - u = url (only for links)
//! The Typst side turns runs into content, so italics / clickable links survive
//! the round-trip and nothing needs `eval`.

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::*;

use biblatex::*;
use core::str;
use serde::Serialize;
use serde_cbor::to_vec;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
initiate_protocol!();

#[derive(Serialize)]
struct Run {
    k: &'static str,
    t: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    u: Option<String>,
}

impl Run {
    fn text(s: impl Into<String>) -> Run {
        Run { k: "text", t: s.into(), u: None }
    }
    fn emph(s: impl Into<String>) -> Run {
        Run { k: "emph", t: s.into(), u: None }
    }
    fn link(disp: impl Into<String>, url: impl Into<String>) -> Run {
        Run { k: "link", t: disp.into(), u: Some(url.into()) }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn format_refs(bib_u8: &[u8], keys_u8: &[u8]) -> Result<Vec<u8>, String> {
    let bib = str::from_utf8(bib_u8).map_err(|e| format!("bib not utf-8: {e}"))?;
    let keys_raw = str::from_utf8(keys_u8).map_err(|e| format!("keys not utf-8: {e}"))?;

    let bibliography =
        Bibliography::parse(bib).map_err(|e| format!("parse failed: {e}"))?;

    let by_key: HashMap<&str, &Entry> =
        bibliography.iter().map(|e| (e.key.as_str(), e)).collect();

    let order: Vec<String> = if keys_raw.trim().is_empty() {
        bibliography.iter().map(|e| e.key.clone()).collect()
    } else {
        keys_raw
            .split(|c| c == ',' || c == '\n' || c == ';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let mut out: Vec<Vec<Run>> = Vec::with_capacity(order.len());
    for key in &order {
        match by_key.get(key.as_str()) {
            Some(entry) => out.push(format_entry(entry)),
            None => out.push(vec![Run::text(format!("〔未找到文献键: {key}〕"))]),
        }
    }

    to_vec(&out).map_err(|e| format!("cbor serialize failed: {e}"))
}

// ---------------------------------------------------------------------------
// GB/T 7714-2015 numeric — a deliberately small subset, enough to prove the
// mechanism on book / article / thesis entries with CN + EN names.
// ---------------------------------------------------------------------------

fn format_entry(entry: &Entry) -> Vec<Run> {
    let get = |name: &str| -> Option<String> {
        entry
            .fields
            .iter()
            .find(|(k, _)| k.as_str() == name)
            .map(|(_, c)| c.format_verbatim())
            .filter(|s| !s.trim().is_empty())
    };
    let persons = |name: &str| -> Vec<Person> {
        entry
            .fields
            .iter()
            .find(|(k, _)| k.as_str() == name)
            .and_then(|(_, c)| <Vec<Person> as Type>::from_chunks(c).ok())
            .unwrap_or_default()
    };

    let langid = get("langid").unwrap_or_default().to_lowercase();
    let is_zh = langid.contains("chinese") || langid.contains("zh") || langid == "cn";

    let ty = entry.entry_type.to_string().to_lowercase();
    let has_url = get("url").is_some();
    let marker = doc_type_marker(&ty, has_url);

    let mut runs: Vec<Run> = Vec::new();

    // 1) primary responsibility (authors)
    let authors = fmt_name_list(&persons("author"), is_zh);
    if !authors.is_empty() {
        runs.push(Run::text(format!("{authors}. ")));
    }

    // 2) title : subtitle [type-marker]
    let mut title = get("title").unwrap_or_else(|| "[无题名]".into());
    if let Some(sub) = get("subtitle") {
        title = format!("{title}: {sub}");
    }
    runs.push(Run::text(format!("{title}[{marker}]. ")));

    // 3) type-specific body
    match marker {
        "J" => {
            // journal, year, volume(number): pages.
            if let Some(j) = get("journal").or_else(|| get("journaltitle")) {
                runs.push(Run::emph(j)); // <- styled run (proves rich text round-trips)
            }
            let mut tail = String::new();
            if let Some(y) = year(&get) {
                tail.push_str(&format!(", {y}"));
            }
            if let Some(v) = get("volume") {
                tail.push_str(&format!(", {v}"));
                if let Some(n) = get("number") {
                    tail.push_str(&format!("({n})"));
                }
            }
            if let Some(p) = get("pages") {
                tail.push_str(&format!(": {}", dash(&p)));
            }
            tail.push('.');
            runs.push(Run::text(tail));
        }
        _ => {
            // book-ish: [translators]. location: publisher, year: pages.
            let translators = fmt_name_list(&persons("translator"), is_zh);
            if !translators.is_empty() {
                let verb = if is_zh { "译" } else { "(trans.)" };
                runs.push(Run::text(format!("{translators}{verb}. ")));
            }
            let mut tail = String::new();
            let loc = get("location").or_else(|| get("address"));
            if let Some(l) = loc {
                tail.push_str(&format!("{l}: "));
            }
            if let Some(pb) = get("publisher") {
                tail.push_str(&pb);
            }
            if let Some(y) = year(&get) {
                if !tail.is_empty() {
                    tail.push_str(", ");
                }
                tail.push_str(&y);
            }
            if let Some(p) = get("pages") {
                tail.push_str(&format!(": {}", dash(&p)));
            }
            if !tail.is_empty() {
                tail.push('.');
                runs.push(Run::text(tail));
            }
        }
    }

    // 4) optional clickable URL (proves link round-trips)
    if let Some(u) = get("url") {
        runs.push(Run::text(" "));
        runs.push(Run::link(u.clone(), u));
    }

    runs
}

fn doc_type_marker(ty: &str, has_url: bool) -> &'static str {
    match ty {
        "article" => "J",
        "book" | "mvbook" | "inbook" | "bookinbook" | "incollection" | "collection" => "M",
        "inproceedings" | "proceedings" | "conference" => "C",
        "phdthesis" | "mastersthesis" | "thesis" => "D",
        "techreport" | "report" => "R",
        "patent" => "P",
        "online" | "electronic" | "www" => "EB/OL",
        "misc" if has_url => "EB/OL",
        _ => "Z",
    }
}

fn year(get: &impl Fn(&str) -> Option<String>) -> Option<String> {
    get("year").or_else(|| {
        get("date").map(|d| d.chars().take(4).collect::<String>())
    })
}

fn dash(p: &str) -> String {
    p.replace("--", "-")
}

/// One person, GB/T 7714 style.
/// CN names: family+given concatenated (given is usually empty for parsed CN).
/// Western names: "Family I I" (given-name initials).
fn fmt_person(p: &Person, is_zh: bool) -> String {
    let given = p.given_name.trim();
    if given.is_empty() {
        return p.name.clone();
    }
    if is_zh {
        return format!("{}{}", p.name, given);
    }
    let initials: Vec<String> = given
        .split(|c: char| c == ' ' || c == '-' || c == '.')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .collect();
    if initials.is_empty() {
        p.name.clone()
    } else {
        format!("{} {}", p.name, initials.join(" "))
    }
}

/// A list of names with GB/T 7714 truncation: >3 names, or an explicit
/// `and others`, collapses the tail to 等 / et al.
fn fmt_name_list(people: &[Person], is_zh: bool) -> String {
    let etc = if is_zh { "等" } else { "et al." };
    let sep = ", ";

    let mut names: Vec<String> = Vec::new();
    let mut had_others = false;
    for p in people {
        if p.name.eq_ignore_ascii_case("others") {
            had_others = true;
            break;
        }
        names.push(fmt_person(p, is_zh));
    }

    if names.is_empty() {
        return String::new();
    }
    if names.len() > 3 {
        return format!("{}{sep}{etc}", names[..3].join(sep));
    }
    if had_others {
        return format!("{}{sep}{etc}", names.join(sep));
    }
    names.join(sep)
}
