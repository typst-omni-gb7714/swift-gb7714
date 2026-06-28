// ===========================================================================
// PoC: 用 Rust WASM 插件做"解析 + 著录(GB/T 7714 子集)"，绕开 hayagriva。
// 引用粘合层(被引集合 / 编号 / 文中标注)交给 Typst 原生机制(state/label/link)。
// ===========================================================================

#set text(font: ("Times New Roman", "Songti SC"), size: 11pt, lang: "zh")
#set page(margin: 2cm)

#let bibdata = read("refs.bib")
#let bibfmt = plugin("bibfmt.wasm")

// CBOR runs -> Typst 内容。富文本(斜体/链接)在这里复活，且全程无需 eval。
#let render-runs(runs) = {
  for r in runs {
    if r.k == "emph" { emph(r.t) }
    else if r.k == "link" { link(r.at("u"))[#r.t] }
    else { r.t }
  }
}

// ---- 引用粘合层：纯原生 Typst（state + label + link）-------------------------
#let cited = state("cited-keys", ())

// .bib 里所有的键（用来判断一个 @label 到底是不是文献引用）
#let all-keys = bibdata.matches(regex("@\w+\s*\{\s*([^,\s]+)")).map(m => m.captures.at(0))

// 接上原生 @key 语法：拦截指向文献条目的 ref，登记并渲染上标 [n]。
// 注意：编号仍是"key 在被引数组里的下标"，由我们算，不是 Typst 的 counter。
#show ref: it => {
  let key = str(it.target)
  if key in all-keys {
    cited.update(k => if key in k { k } else { k + (key,) })
    context {
      let ks = cited.get()
      let n = ks.position(x => x == key) + 1
      super(link(it.target)[\[#n\]])
    }
  } else {
    it // 不是文献键，交还给 Typst 默认行为
  }
}

// 参考文献表：只取被引的 key，按引用顺序，调用插件一次性著录
#let render-bibliography() = context {
  let order = cited.final()
  if order.len() == 0 { return }
  let payload = order.join("\n")
  let entries = cbor(bibfmt.format_refs(bytes(bibdata), bytes(payload)))
  set par(hanging-indent: 1.6em, justify: true)
  for (i, runs) in entries.enumerate() {
    let key = order.at(i)
    block(below: 0.7em)[\[#(i + 1)\]#h(0.5em)#render-runs(runs)#label(key)]
  }
}

// ===========================================================================
= 一、机制验证：插件直接吐出"成品"著录

下面整段参考文献，每一条的标点、姓名缩写、`等`/`et al.` 截断、`[M]/[J]`
标识、斜体刊名、可点击链接，**全部由 Rust 插件生成**，Typst 只负责排版：

#let all = cbor(bibfmt.format_refs(bytes(bibdata), bytes("")))
#set par(hanging-indent: 1.6em, justify: true)
#for (i, runs) in all.enumerate() {
  block(below: 0.6em)[\[#(i + 1)\]#h(0.5em)#render-runs(runs)]
}

#line(length: 100%, stroke: 0.4pt + gray)

= 二、引用粘合层：原生 @ 语法 + 只列被引文献

历史的走向有其模式 @Morris2010，而社会规范塑造社会角色
@Sunstein1996。讨论西方文明的源头时 @Rogers2011，
我们再次回到诺姆问题 @Sunstein1996（重复引用，编号应保持 [2]）。

下面这张表**只列出上文真正引用过的 3 条**，按引用先后编号，
文中 [n] 可点击跳转到对应条目：

== 参考文献

#render-bibliography()
