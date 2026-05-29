# 28 — github-url-fix

> 日期：2026-05-29
> 涉及代码：24 文件 / 57 处 `keerecles/RAG4ArkUI` → `LeoG1022/RAG4ArkUI` sed 替换
> 类型：bug 修复（agent 凭空假设的 username 全 repo 死链清扫）

## 本轮目标

修 22 个历史 commit 累计的 GitHub URL 死链 —— agent 之前所有文档假设 username 是
`keerecles`（凭空），实际是 `LeoG1022`。让用户向链接真能点开。

## 改动要点

- **替换范围**：active 文档（README/RELEASE/mdbook/Cargo.toml/scripts）· 跳过 `feedback/` 历史归档
- **替换工具**：grep -rln + xargs sed -i ''
- **正交性**：与「Actions 没触发」问题无关 · workflow yml 用 `${{ github.repository }}` 自动取
- **决策原则**：URL 错是 functional bug（死链 404）· 不同于「真活」风格漂移 · 这次允许 sed

## 验证结果

- ✅ `grep keerecles --exclude-dir=feedback` 0 命中
- ✅ `grep LeoG1022 --include="*.md" --include="*.toml" --include="*.yml" --include="*.rs" --include="*.sh"` 57 处
- ✅ feedback/ 历史归档保留不动（agent 决策原则 · GLOSSARY 已声明）
- ⏳ 此 commit push 后 · 期望触发 ci.yml / book.yml workflow（验证 GitHub Actions 配置）

## 残留 / 下一轮

- [ ] **关键**：等 push 后用户告知 GitHub Actions 页面看到的状态
- [ ] 用户在仓库 Settings → Pages → Source 选 "GitHub Actions"（一次性）
- [ ] 后续若改 GitHub username / fork 仓库 · 需再次全 repo sed
- [x] active 文档 24 个文件 / 57 处 URL 修正
- [x] feedback/ 历史归档保留（不擅自重写）
- [x] 配套 meta feedback 13（解释这次决策）
