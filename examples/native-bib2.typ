#set text(font: ("Times New Roman", "Songti SC"), size: 11pt, lang: "zh")
#set page(margin: 2cm)

= 思路1：原生 bib 引擎 + Typst 层后处理(show regex) 还原富文本

历史走向 @Morris2010，社会规范 @Sunstein1996，源头 @Rogers2011，重复 @Sunstein1996。

// 作用域限定：只在这个 block 内对 bib 文本做 O(n) 哨兵升级，不影响正文
#[
  // 斜体哨兵 _x_
  #show regex("_[^_]+_"): it => emph(it.text.slice(1, -1))
  // 链接哨兵 ⟦url⟧⟨显示⟩
  #show regex("⟦[^⟧]+⟧⟨[^⟩]+⟩"): it => {
    let m = it.text.match(regex("⟦([^⟧]+)⟧⟨([^⟩]+)⟩"))
    link(m.captures.at(0))[#m.captures.at(1)]
  }
  #bibliography("gen2.yml", style: "verbatim.csl", title: "参考文献")
]
