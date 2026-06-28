// ===========================================================================
// 架构 B（用户要的）：wasm 成品字符串 -> hayagriva YAML 字段 -> 原生 #bibliography
// CSL 只做透传，于是原生 @key / 编号 / 只列被引 / 排序 全部白拿。
// 这里先用手写的 gen.yml 代替 wasm 输出，专门验证"原生 bib 当引擎"这条路。
// ===========================================================================

#set text(font: ("Times New Roman", "Songti SC"), size: 11pt, lang: "zh")
#set page(margin: 2cm)

= 正文（原生 @ 语法，原生编号）

历史的走向有其模式 @Morris2010，而社会规范塑造社会角色 @Sunstein1996。
讨论西方文明的源头时 @Rogers2011，我们再次回到诺姆问题 @Sunstein1996
（重复引用，编号应复用）。

#bibliography("gen.yml", style: "verbatim.csl", title: "参考文献")
