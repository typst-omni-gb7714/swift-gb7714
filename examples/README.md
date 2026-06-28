# examples —— 可行性验证最小用例

> 会话中逐步验证 swift 架构各环节的最小复现用例。每个 `.typ` 都能
> `typst compile <文件>.typ` 直接编出来。下表是每个用例**验证的点**。
> （扁平放置是刻意的：多个用例共享 `verbatim.csl`，且各 `.typ` 按裸文件名取相对资产。）

| # | 用例 | 文件 | 验证了什么 |
|---|---|---|---|
| 1 | wasm 著录 + 富文本回传 | `test.typ` · `refs.bib` · `bibfmt.wasm` | Rust 插件解析 `.bib` + GB7714 著录子集 → CBOR runs → Typst 渲染斜体/可点击链接；引用编号用纯 Typst 原语（`state`/`label`/`link`）做 |
| 2 | 原生引擎当透传粘合 | `native-bib.typ` · `gen.yml` · `verbatim.csl` | 成品串塞进 hayagriva YAML 的 `title` 字段 + verbatim CSL 透传 → 原生 `@key`/编号/只列被引全部白拿 |
| 3 | 哨兵还原富文本 | `native-bib2.typ` · `gen2.yml` | 纯文本里埋哨兵，作用域限定的 `show regex` 把 `_…_`／链接哨兵升级为真斜体/真链接 |
| 4 | `bibliography` 吃 `bytes` | `bytestest.typ` | `#bibliography(bytes(..))` 可行 → wasm 生成的 YAML 无需落地文件，全程在一次 `typst compile` 内闭环 |
| 5 | CSL 版式属性被 honor | `hangtest.typ` · `hangtest.yml` · `verbatim-hang.csl` | `hanging-indent` / `second-field-align` 等 `<bibliography>` 属性 Typst 认 |
| 6 | 原生按注入键排序 | `aysort.typ` · `aysort.yml` · `verbatim-aysort.csl` | 引用顺序故意打乱，原生 bib 仍按注入排序字段的键输出 → 著者-出版年制的拼音/笔画排序可“wasm 算键、原生交付” |

`../plugin/` 是用例 1 用到的 Rust 插件源码。构建：

```sh
cd ../plugin
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/bibfmt.wasm ../examples/bibfmt.wasm
```

> 注：用例 2–6 不依赖 wasm —— 它们用**手写的 YAML** 模拟 wasm 的成品输出，
> 专门验证「原生 #bibliography 当引擎」这条链路。只有用例 1 直接调插件。
