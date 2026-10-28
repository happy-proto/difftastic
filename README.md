# difftastic：sidediff 内部库 fork

这是 [Wilfred/difftastic](https://github.com/Wilfred/difftastic) 的长期维护 fork，专供 [happy-proto/sidediff](https://github.com/happy-proto/sidediff) 在进程内调用语法差异计算。

在上游提供满足需求的库接口之前，我们临时维护这层适配，减少逐文件启动 `difft`、查询 Git attributes 和物化临时文件的开销。差异算法沿用上游，保留 `difft` CLI；这个接口服务于 sidediff，不承诺作为通用 Rust 库的兼容性。

当前以 **0.71.0** 为上游基线，精确记录见 [upstream.json](scripts/fork/upstream.json)。需要 **Rust 1.90 或更新版本**。本 fork 通过 Git 依赖使用，不发布到 crates.io，也不提供独立的 CLI 发行包。

## 在 sidediff 中使用

```toml
[dependencies]
difftastic = { git = "https://github.com/happy-proto/difftastic.git" }
```

具体提交由消费端的 `Cargo.lock` 锁定，manifest 不设置 `version`、`rev`、`tag` 或 `branch`。更新 fork 后，由消费端运行 `cargo update -p difftastic` 并验证。

`JsonDiff` 提供三种接入能力：

- `from_env()` 读取一次 `DFT_*` 配置；库接口固定输出 JSON，执行完整差异计算。
- `set_binary_attributes()` 接收调用者查询的 Git attributes，避免逐文件启动 Git；也保留 `prefetch_attributes()` 批量查询接口。
- `diff_slices()` 比较调用者提供的字节，路径仅用于语言识别等元数据；另有 `diff_files()` 文件入口。

```rust
use std::{collections::HashMap, path::Path};

let mut engine = difftastic::JsonDiff::from_env()?;
engine.set_binary_attributes(HashMap::from([("example.rs".to_owned(), false)]));
let json = engine.diff_slices(
    "example.rs",
    Path::new("old/example.rs"),
    Path::new("new/example.rs"),
    b"fn value() -> i32 { 1 }\n",
    b"fn value() -> i32 { 2 }\n",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

调用者应提供本次加载所有路径的 attributes；缺少路径时，仍会回退到原生 Git 查询。`diff_slices()` 不读取或写入传入路径，但算法仍会创建字符串、语法树和 JSON；输入大小、并发、结果保留和内存预算由 sidediff 控制。继承的 jemalloc 全局分配器及其他接入约束见 [库适配记录](docs/fork-changes/0001-library-api.md)。

## 开发与同步上游

```sh
cargo fmt --all -- --check
cargo test --locked
```

维护主分支为 `main`；`origin` 指向本 fork，`upstream` 指向 Wilfred/difftastic。自有改动见 [编号改造记录](docs/fork-changes/README.md)，同步上游使用仓库内的 [difftastic-upstream-sync skill](.agents/skills/difftastic-upstream-sync/SKILL.md)。

上游的功能说明、CLI 安装和用户手册见 [官方手册](https://difftastic.wilfred.me.uk/)。本仓库保留上游的测试和源码归属；不启用上游的站点部署、官方发行包或 crates.io 发布流程。

## 许可证与归属

原项目由 Wilfred Hughes 等贡献者维护，主体遵循 [MIT License](LICENSE)。`vendored_parsers/` 中的解析器各自遵循其目录下的许可证；样本文件的授权说明保留在原文件中。这个 fork 不是上游的官方 Rust 库发行版。
