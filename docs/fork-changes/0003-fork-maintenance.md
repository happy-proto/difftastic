# 0003：内部库与同步维护

状态：维护中，上游基线见 [记录文件](../../scripts/fork/upstream.json)。

本 fork 以一个长期 `main` 分支保留上游基线及全部自有适配。README 使用中文说明 sidediff 专用、临时库接口和消费方式；保留上游归属与许可证，禁止发布到 crates.io。

`.agents/skills/difftastic-upstream-sync/SKILL.md` 默认获取上游 `master` 的最新 commit，在每轮开始时固定 SHA；不以最新稳定发行版为默认目标。记录文件保存跟踪 ref 和精确 commit，并验证该 commit 是 fork 主分支的祖先。同步在临时 worktree 中合入上游，按记录适配并验证消费端，普通同步保留已有历史。

只保留测试工作流，移除官方发行包、crates.io 发布、手册部署和 Codecov 上传流程。维护中不自动安装或替换本机 CLI；GitHub Actions 的启用状态与源码验证分别验收。

测试平台对应 sidediff 的 Linux x64、macOS Apple Silicon、Windows x64 和 Windows ARM64，保留原有样本输出、MIME、打包等验证；不沿用额外的 cross 构建矩阵。同分支新提交取消过时验证。

消费端 `Cargo.toml` 只声明 Git 地址，具体提交由 `Cargo.lock` 管理。更新本 fork 不会自动改变消费端已锁定的提交，需要单独更新并验证消费端。
