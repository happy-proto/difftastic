# 0002：tree-sitter 与 Rust 版本

状态：维护中，上游基线见 [记录文件](../../scripts/fork/upstream.json)。

sidediff 同时依赖 kat 和 difftastic。两者的 tree-sitter 都声明原生 `links = "tree-sitter"`，Cargo 不能同时链接两个不兼容版本。当前统一为 0.27.1，并将 query match 的 `captures` 字段读取适配为 `captures()` 方法。

tree-sitter 0.27 会拒绝 Solidity 1.2.13 高亮查询中的分组末尾锚点，已有 `test_configs_valid` 能复现失败，后续配置读取还会因共享缓存锁中毒而失败。构建 Solidity 查询时应用 [上游 PR #80](https://github.com/JoranHonig/tree-sitter-solidity/pull/80) 的最小修复：直接捕获 `struct_expression` 的 `type` 字段，保留类型高亮意图。不修改注册表源码、不额外引入语法库 fork；上游发行版吸收后删除该替换。

tree-sitter 0.27.1 要求 Rust 1.90，因此同步调整本 fork 的 MSRV、工具链和测试配置；不能继续沿用上游的 Rust 1.85 声明。

自带的四种 parser 在编译时给入口和 external scanner 符号添加 `difftastic_` 前缀，Rust FFI 使用对应 `link_name`。Linux 链接 sidediff 时，kat 的 Kotlin／LaTeX scanner 与本库同名，导致重复符号；仅统一 tree-sitter 版本不足以解决。保持各自 scanner 与各自生成的 parser 配套，不通过允许重复定义或修改第三方缓存绕过问题。`tests/parser_symbols.rs` 模拟宿主导出同名 scanner，先复现链接失败，再验证两种语言仍能正常计算差异。

入口为 `Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`build.rs`、`.github/workflows/test.yml` 和 `src/parse/tree_sitter_parser.rs`。验证包括解析器测试、原 CLI 和 sidediff 的 Cargo 依赖解析与内容／高亮回归。

同步时先核对 kat、上游 difftastic 的实际 tree-sitter 约束和解析器 API。上游吸收相同版本后移除多余适配；需要升级 kat 时在消费端明确处理，不能为解决链接冲突顺带升级无关依赖。
