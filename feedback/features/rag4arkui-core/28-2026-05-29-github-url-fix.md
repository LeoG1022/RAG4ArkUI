# 28 — github-url-fix

> 日期：2026-05-29
> 涉及代码：24 文件 / 57 处 `keerecles/RAG4ArkUI` → `LeoG1022/RAG4ArkUI` sed 替换
> 类型：bug 修复（agent 凭空假设的 username 全 repo 死链清扫）

## 本轮目标

修 22 个历史 commit 累计的 GitHub URL 死链 —— agent 之前所有文档假设 username 是
`keerecles`（凭空），实际是 `LeoG1022`。让用户向链接真能点开。

## Plan

> 必备节（FAIL 级 M-FEATURE-PLAN）

- 范围：active 文档（README/RELEASE/mdbook/Cargo.toml/scripts/workflow yml 注释）
- 排除：feedback/ 历史归档（GLOSSARY 已声明的「不擅自重写历史」原则）
- 工具：grep -rln + xargs sed -i ''（macOS sed 语法）
- 验证：grep keerecles 应只剩 feedback/ 下命中

## 改动要点

- workflow yml 实际功能不受 username hardcode 影响（softprops/action-gh-release 用
  `${{ github.repository }}` 自动取 · actions/deploy-pages 自动用当前 repo）
- 这次修的全是用户向死链 · 与 Actions 触发问题正交
- URL 错是 functional bug（死链 404）· 不同于「真活」风格漂移 · 这次允许 sed

## 对话摘要

> 必备节（FAIL 级 M-FEATURE-PLAN）

按时序：
1. 用户：「推送到github了 在setting的action并没有看到有什么任务运行」
2. Agent 查 `git remote -v` 发现实际 GitHub URL 是 `LeoG1022/RAG4ArkUI`
3. Agent 顺手 grep 发现 24 文件 / 57 处 username 错（active + feedback 各自）
4. Agent 老实告知用户「URL 错 ≠ Actions 不触发」+ 借机清扫死链
5. 替换完成 + 双轨归档（meta 13 + feature 28 + STATUS-github-url-fix）

## 验证结果

- ✅ `grep keerecles --exclude-dir=feedback` 0 命中
- ✅ `grep LeoG1022 --include="*.md" --include="*.toml" --include="*.yml" --include="*.rs" --include="*.sh"` 57 处
- ✅ feedback/ 历史归档保留不动
- ⏳ push 后 · 期望触发 ci.yml + book.yml workflow（验证 GitHub Actions 配置）

## 残留 / 下一轮

- [ ] **关键**：等 push 后用户告知 GitHub Actions 页面看到什么
- [ ] 用户在仓库 Settings → Pages → Source 选 "GitHub Actions"（一次性）
- [ ] 后续若改 GitHub username / fork · 需再次全 repo sed
- [x] active 文档 24 个文件 / 57 处 URL 修正
- [x] feedback/ 历史归档保留（不擅自重写）
- [x] 配套 meta feedback 13
