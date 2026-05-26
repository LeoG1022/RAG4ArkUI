# stats/ — Token 与改动量统计

每轮迭代（meta + business）的 token 消耗 / 改动量记录池。
**仅记录 agent 提交**（agent 在 `git commit` 后主动调用）；用户手动提交不记录。

---

## 文件

| 文件 | 内容 |
|---|---|
| `tokens.jsonl` | append-only，每轮一行 JSON 记录 |
| `summaries/<date>-*.md` | 用户触发分析产物 |

`stats/` 路径分类为 **business**（操作数据，类比 `reports/`）。

---

## tokens.jsonl 字段定义

```json
{
  "timestamp": "2026-05-18T16:30:00Z",   // commit 时刻
  "round": 14,                            // meta 必填；business 留 null
  "feature": null,                         // business 必填（feature 名）；meta 留 null
  "iteration": null,                       // business：该 feature 第几次迭代
  "type": "meta",                          // "meta" | "business" | "mixed"
  "commit": "abc1234",                     // 短 hash
  "subject": "Round 14: ...",              // commit 标题（一行）
  "files_changed": 10,
  "lines_added": 100,
  "lines_removed": 20,
  "input_tokens_actual": null,             // 用户从 UI 读后手填
  "output_tokens_actual": null,            // 同上
  "duration_sec": null,                    // 可选；agent 跑这轮耗时
  "notes": null                            // 可选备注
}
```

**为什么 token 字段是 null**：Agent 通过工具调用拿不到自己实际 token 用量；用户从 Claude Code UI 看到准确数字后用 `annotate` 子命令回填。

**lines_added/removed** 是确定性数据（`git diff --numstat`）；可作 token 启发式估算（粗略 `lines_added * 5`）。

---

## 工具

```bash
# agent 提交后主动调用（用户手动提交无需调用）
bash scripts/log-tokens.sh --from-commit HEAD

# 用户标注 actual token
bash scripts/log-tokens.sh annotate <commit-or-round> \
  --input-tokens 50000 --output-tokens 15000 --duration 1800

# 一次性回填历史（首次安装时跑一次）
bash scripts/log-tokens.sh backfill

# 统计分析（用户触发）
bash scripts/stats-report.sh                  # 全量摘要
bash scripts/stats-report.sh --by-round       # 按 round 列表
bash scripts/stats-report.sh --by-feature     # 按 feature 列表
bash scripts/stats-report.sh --trend          # 累计趋势
bash scripts/stats-report.sh --top 5          # token / 行数最多的 N 轮
```

---

## 约定

- **记录触发**：agent 每次 `git commit` 后主动调用（用户手动提交不记录）
- **统计分析**：用户手动触发，不进 pre-commit / CI
- **token 回填**：用户随时可 annotate，多次 annotate 后写覆盖
- **summaries/**：用户产物，可放周报/季度回顾等

---

## 下一步

- 想看历史 token 用量 → `bash scripts/stats-report.sh`
- 想标本轮 token → `bash scripts/log-tokens.sh annotate <round> --input-tokens N --output-tokens N`
- 添加新统计维度 → 改 `scripts/stats-report.sh`（保留向后兼容字段）
