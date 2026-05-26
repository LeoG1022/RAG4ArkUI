# Example Mapping — [Domain Name]

> **Delete this file** and replace it with your project's actual mapping tables.
> This file shows the required structure for any `mapping-*.md` file.
>
> Every mapping file MUST contain an `## Anti-Patterns` section
> (checked by `check-consistency.sh` M-MAP-AP rule).

---

## API Mapping Table

| Source API / Pattern | Target API / Pattern | Notes |
|---|---|---|
| `SourceFoo()` | `TargetFoo()` | Direct equivalent |
| `SourceBar(x, y)` | `TargetBar(x).withY(y)` | Parameter restructuring |
| `SourceBaz` | — | No direct equivalent; use workaround XYZ |

---

## Common Patterns

### Pattern 1: [Name]

**Source:**
```
// Source platform code
SourceFoo(items) { item -> render(item) }
```

**Target:**
```
// Target platform code
TargetFoo(items, item => render(item), item => item.id.toString())
```

**Key differences:**
- Difference 1
- Difference 2

---

## Anti-Patterns

> This section is **required** by M-MAP-AP consistency check.

### ❌ Anti-Pattern 1: [Name]

**Problem:**
```
// Wrong: does not have required property
TargetFoo(items, item => render(item))
//              ^^^ missing keyGenerator
```

**Fix:**
```
// Correct: always provide the key function
TargetFoo(items, item => render(item), item => item.id.toString())
```

**Check:**
```bash
grep -n "TargetFoo(" src/ | grep -v ", item =>"
# Any match = missing keyGenerator
```

---

### ❌ Anti-Pattern 2: [Name]

**Problem:**
```
// Wrong: resource not released on component destroy
onCreate() { this.timer = setInterval(...) }
// missing: onDestroy() { clearInterval(this.timer) }
```

**Fix:**
```
onDestroy() {
  clearInterval(this.timer)
}
```

**Check:**
```bash
grep -c "setInterval" src/ && grep -c "clearInterval" src/
# setInterval count should match clearInterval count
```
