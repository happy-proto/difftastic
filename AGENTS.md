All AI contributions to difftastic must follow AI_POLICY.md.

Do not update CHANGELOG.md unless the user explicitly requests it.

## sidediff 内部 fork

- 本仓库是 `happy-proto/difftastic`，用于 sidediff 在上游开放合适的库接口之前内部接入。自有文档和注释使用中文，手写提交使用英文 Conventional Commits；保留上游归属与许可证。
- 长期维护分支为 `main`；`origin` 指向本 fork，`upstream` 指向 `Wilfred/difftastic`。上游基线记录在 `scripts/fork/upstream.json`，必须是主分支的祖先。
- 按目的组织提交，普通维护保留历史；上游同步使用 [.agents/skills/difftastic-upstream-sync/SKILL.md](.agents/skills/difftastic-upstream-sync/SKILL.md)。提交、推送或历史改写遵循当前任务授权，不覆盖远端他人的提交。
- 自有改造维护在 `docs/fork-changes/`，已有改造更新原编号；上游吸收改造后保留记录并注明状态。
- 检查使用 `cargo fmt --all -- --check` 和 `cargo test --locked`，覆盖库、原 CLI 和库调用专项测试。tree-sitter 需与 sidediff/kat 的原生 `links` 依赖兼容。
- workflow 变更时用 `gh-actions-lock .github/workflows/test.yml --no-narrow` 生成原生锁文件，并以同一路径运行 `--verify`。
- 消费端 Git 依赖仅声明仓库地址，使用 `Cargo.lock` 锁定提交；不在 manifest 添加 `version`、`rev`、`tag` 或 `branch`。
- 本 fork 不向 crates.io 发布，也不恢复上游的官方发行包、站点部署和覆盖率上传工作流。
