---
name: difftastic-upstream-sync
description: 将 happy-proto/difftastic fork 同步到上游 master 最新提交，或评估上游及官方库接口变化对 sidediff 接入的影响。
---

# Difftastic 上游同步

先读 [AGENTS.md](../../../AGENTS.md) 和 [改造索引](../../../docs/fork-changes/README.md)，按本次任务授权决定本地演练、提交和推送范围。只读检查不改变基线；用户明确指定版本或限制推送时优先遵循，不把本 skill 作为额外的发布或安装授权。

## 确定基线

1. 核对工作区、`origin`、`upstream` 和远端 `main`。保留已有改动；有脏工作区时在隔离 worktree 准备，不自行 stash 或重置。只 fetch 所需分支和候选 tag。
2. 默认获取 [Wilfred/difftastic](https://github.com/Wilfred/difftastic) 的 `refs/heads/master` 最新 commit，不等待稳定版发布；用户明确指定其它 ref／SHA 时遵循指定。可用 `git fetch upstream refs/heads/master:refs/remotes/upstream/master` 获取主线，再解析并固定本轮 SHA，不在执行中追逐移动目标。
3. 读取 `scripts/fork/upstream.json`，确认记录的旧 commit 存在且是当前 fork 主分支的祖先。`ref` 是跟踪来源，移动分支的当前位置不必等于旧 SHA；不要据此误判记录失效。无法核对时先查清历史，不凭 merge-base 猜测自有补丁边界。
4. 记录旧主分支 SHA、远端期望 SHA、新旧上游 SHA 和祖先关系。已包含目标基线时跳过合入，仍检查上游库接口变化和本次明确要求的维护事项。

## 在临时 worktree 适配

从当前 fork 主分支建立临时分支和 worktree，合入本轮固定的上游 commit：

```sh
git merge --no-commit --no-ff <new-upstream-commit>
```

逐项阅读编号改造记录，以净 diff 和回归行为决定如何处理冲突，不机械选择 ours/theirs。无冲突也要检查遗漏和重复。默认保留 fork 已发布的历史；只有用户明确授权重写时才改用基于已记录旧 commit 的 rebase，并保存恢复引用。

- **库接入**：保留字节入口、CLI 共用算法、配置读取、attributes 注入／批量查询和 JSON 对齐行为，继续避免逐文件启动 Git。性能测量默认关闭。
- **原生依赖**：核对 tree-sitter 与 sidediff/kat 的 `links` 兼容性、解析器 API 和 MSRV。自带 parser 的入口和 scanner 需保留 `difftastic_` 符号前缀；新加或更换 parser 时核对宿主是否有同名导出，并运行 `parser_symbols` 回归及 sidediff 的 Linux 链接检查。修改依赖与锁文件使用 Cargo，不顺带升级无关依赖。
- **Kotlin 源码副本**：核对上游 `tree-sitter-kotlin-sg` 依赖和锁定版本与 `vendored_parsers/tree-sitter-kotlin-sg/` 是否一致；升级时保留出处、许可证及两处原生符号隔离适配。上游支持符号隔离后恢复 registry 依赖，不长期停留在旧 parser。
- **维护边界**：保留中文 fork 定位、许可证、改造记录和本 skill；不要恢复上游的官方发布、站点部署或 Codecov 上传流程。

检查新上游是否已经提供适用的库接口。验证其字节输入、attributes、配置和 JSON 能否替代现有接入；上游吸收同一目的时移除重复补丁，保留原编号并记录吸收版本。不能仅因出现 `lib.rs` 就宣布可以退役 fork。

完成适配后更新 `scripts/fork/upstream.json` 及受影响的原编号记录，不将当次执行日志加入长期维护文档。

## 验证与消费端

- 执行 `cargo fmt --all -- --check`、`cargo test --locked`，覆盖原 CLI 和库接口专项测试。检查新旧上游间的主要行为变化，并用净 diff／必要的 range-diff 检查补丁范围。
- workflow 发生变化时按仓库约定生成并验证 `actions.lock`，保留兼容范围引用与 fork 的发布边界。
- 在 sidediff 的隔离验证环境中将依赖临时指向演练结果，按其 `AGENTS.md` 执行检查与测试。核对 Git attributes、二进制与编码、冲突和内容／高亮行为；按变化风险决定是否重跑性能样本。
- 本地 path 验证结束后恢复远程 Git 依赖。最终 manifest 只保留 `git = "https://github.com/happy-proto/difftastic.git"`，不添加 `rev`、`version`、`tag` 或 `branch`。消费端具体提交通过 Cargo 更新 `Cargo.lock`；fork 的验证通过不等于消费端已经更新。

## 交付

在当前授权内将演练结果快进到本地主分支并正常推送。若用户明确授权了重放并改写远端历史，使用同步前记录的精确远端 SHA：

```sh
git push --force-with-lease=refs/heads/main:<expected-old-sha> origin main
```

远端已变化时重新整合和验证，不覆盖其他人的提交。回读远端默认分支和最新 SHA；Actions 已启用时跟进该 SHA 的相关检查，关闭时明确报告，不自行开启。清理本轮临时 worktree 和不再需要的恢复引用。

最终简述新旧上游 SHA、主要上游变化、哪些 fork 改造保留／需要适配／已被吸收，以及源码验证、远端检查和 sidediff 锁文件更新各自的状态。不发布官方 CLI、不开 crates.io 发布，也不替换本机安装。
