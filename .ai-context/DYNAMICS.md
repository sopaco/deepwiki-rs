# Dynamics — Active Issues & Constraints

> **Last updated:** 2026-04-16
> **Stability:** Dynamic — Update as issues arise/resolve

---

## ⚡ Quick Scan

| Status | Issue | Impact | Workaround |
|--------|-------|--------|------------|
| 🟢 Clean | No active issues | — | — |

---

## 🔴 Active Issues

*No active issues at this time.*

<!-- Template for adding issues:
### [Issue Title]

**What:** {Brief description of the issue}

**Impact:** {How this affects development/workflow}

**Workaround:**
```
{Steps or code to work around the issue}
```

**Resolution Path:** {Planned fix or "Not yet planned"}
-->

---

## 🟡 Known Constraints

### LLM API Rate Limits

**What:** Cloud LLM providers have rate limits that can slow down large codebase analysis.

**Impact:** Processing large projects may experience delays or require retries.

**Mitigation:** 
- Use `--cache-dir` to enable response caching
- Consider using Ollama for local processing without rate limits
- Break large projects into smaller analysis runs

---

### Memory Usage for Large Projects

**What:** Analyzing very large codebases can consume significant memory during the preprocessing phase.

**Impact:** May cause slowdowns on machines with limited RAM.

**Mitigation:**
- Use `--exclude` to skip unnecessary directories (node_modules, target, etc.)
- Process specific subdirectories separately

---

## 🟢 Recently Resolved

| Issue | Resolution | Date |
|-------|------------|------|
| *None recorded yet* | — | — |

---

## 📋 Under Consideration

### Potential: Git History Analysis

**Topic:** Analyzing git commit history to track architecture evolution.

**Trigger for change:** If users request features showing how architecture has changed over time.

---

### Potential: Multi-Repository Analysis

**Topic:** Supporting analysis across multiple related repositories.

**Trigger for change:** Enterprise users with microservices architectures across many repos.

---

## 🔄 Update Log

| Date | Change |
|------|--------|
| 2026-04-16 | Initial version created during .ai-context setup |

---

*Remember: This file changes frequently. Always check the "Last updated" date. If it's been > 2 weeks, verify against current code state.*