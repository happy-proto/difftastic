# 0001：进程内差异接口

状态：维护中，上游基线见 [记录文件](../../scripts/fork/upstream.json)。

## 目的与行为

将原 CLI 实现移入 `src/lib.rs`，`src/main.rs` 仅调用 `cli_main()`，保持 CLI 行为。`JsonDiff` 共享原有识别、解析、结构化／行级差异和 JSON 展示实现，不另写一套算法。

库目标新增了 rustdoc 测试入口；原有三处内部示意片段明确标为 `text`，公开字节接口的 Rust 示例作为 doctest 编译并执行，不全局关闭文档测试。

- `from_env()` 使用固定参数读取 `DFT_*` 配置，不修改宿主环境；库接口固定完整差异和 JSON 输出。
- `diff_slices()` 接收字节切片，路径仅作语言识别和覆盖规则元数据；`diff_files()` 在读取失败时返回错误。
- attributes 按加载批量查询，或由调用者通过 `set_binary_attributes()` 注入。注入结果覆盖旧缓存；未提供路径时保留原 Git 查询回退。
- `profile` 仅在设置 `SIDEDIFF_PROFILE_LOAD` 时计时。数据保存在当前线程，调用者可将工作线程样本合并；嵌套耗时存在重叠，不能简单相加。

## 接入约束

这是给 sidediff 使用的内部接口，不是稳定的公共 API。原 CLI 专用入口仍可能退出进程；不要把 `cli_main()` 当库计算接口调用。原有 jemalloc 全局分配器也随库保留，会影响整个宿主进程；宿主不能再同时声明另一个全局分配器。

读取字节不代表低内存：文本解码、AST、行级回退和 JSON 都有分配。difftastic 的 byte／graph 阈值只限制部分算法，文件准入和并发预算由消费端控制。

## 验证与同步边界

运行 `cargo test --locked`，保留原 CLI 测试，并验证字节入口、JSON 与 CLI 的一致性、attributes 覆盖和并发调用。消费端需另跑 sidediff 的常规测试及实际加载样本，覆盖 Git 对象、编码、冲突、元数据和高亮。

主要入口：`src/lib.rs`、`src/options.rs`、`src/display/json.rs`、`src/gitattributes.rs` 和 `src/profile.rs`。上游如果重排 CLI 或 diff 流程，保持同一算法入口，不机械重放整个旧 `main.rs`。

上游开放库接口后，先验证字节输入、配置、attributes 和 JSON 行为是否满足消费端；确认能替代后删除重复适配，并在本记录注明吸收版本。
