# 0003：内部库与同步维护

状态：维护中，上游基线见 [记录文件](../../scripts/fork/upstream.json)。

本 fork 以一个长期 `main` 分支保留上游基线及全部自有适配。README 使用中文说明 sidediff 专用、临时库接口和消费方式；保留上游归属与许可证，禁止发布到 crates.io。

`.agents/skills/difftastic-upstream-sync/SKILL.md` 负责选择稳定版基线、临时 worktree 演练、按记录适配和消费端验证。记录文件同时保存 tag 和精确 commit，避免使用不准确的 merge-base 作为补丁重放边界。

只保留测试工作流，移除官方发行包、crates.io 发布、手册部署和 Codecov 上传流程。维护中不自动安装或替换本机 CLI；GitHub Actions 的启用状态与源码验证分别验收。

消费端 `Cargo.toml` 只声明 Git 地址，具体提交由 `Cargo.lock` 管理。更新本 fork 不会自动改变消费端已锁定的提交，需要单独更新并验证消费端。
