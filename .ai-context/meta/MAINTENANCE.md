# AI Context Maintenance Guide

> How to keep this knowledge base accurate and useful. Last updated: 2026-04-16.

---

## 🎯 Purpose of This Guide

This document tells you (the coding agent) how to maintain the `.ai-context` knowledge base. Following these guidelines ensures knowledge stays accurate without becoming a maintenance burden.

---

## 📋 Maintenance Triggers

### Automatic Update Triggers
Update knowledge when you observe:

| Observation | Action | File to Update |
|-------------|--------|----------------|
| Code contradicts documented behavior | Fix documentation | Relevant reference file |
| New component created | Add entry | `ARCHITECTURE.md` |
| Major design choice made | Document rationale | `DECISIONS.md` |
| Issue blocking work discovered | Add entry | `DYNAMICS.md` |
| Issue resolved | Remove/move entry | `DYNAMICS.md` |
| API contract changed | Update section | `ARCHITECTURE.md` |

### Explicit Update Requests
When user asks to "update the knowledge base" or "refresh ai-context":
1. Review each file for accuracy
2. Compare documented state against current code
3. Update outdated sections
4. Update the "Last updated" date at file top
5. Summarize changes to user

---

## ✍️ Writing Guidelines

### For PROJECT-ESSENCE.md
- Keep it under 100 lines
- Focus on "what" and "why", not "how"
- No code snippets, no API details
- Target audience: Someone who knows nothing about the project
- Update frequency: Rare (quarterly or major version)

### For ARCHITECTURE.md
- Use diagrams over paragraphs (ASCII art or Mermaid)
- Component-level granularity, not file-level
- Show data flow, not implementation details
- Include: Responsibilities, Dependencies, Interfaces
- Omit: Internal algorithms, variable names, code snippets
- Update frequency: Monthly or per sprint

### For DECISIONS.md
- Format: `## ADR-XXX: [Title]` followed by Context, Decision, Rationale, Trade-offs
- Include decisions that are non-obvious or controversial
- Omit trivial decisions (naming conventions, etc.)
- When revisiting a decision, add a new section noting the change
- Update frequency: As decisions are made/changed

### For DYNAMICS.md
- Keep entries actionable
- Format: `## 🔴 [Issue Title]` with status emoji
- Include: Impact, Workaround (if any), Resolution plan
- Remove resolved issues promptly (don't accumulate history)
- Update frequency: As needed (most dynamic file)

---

## 🔄 Update Workflow

### When Making Updates

```
1. Identify which file needs updating
2. Read the current content
3. Make minimal, focused changes
4. Update "Last updated" date at top
5. Proceed with your task
```

### What NOT to Do

- ❌ Don't rewrite entire files for minor changes
- ❌ Don't add details that belong in code comments
- ❌ Don't duplicate information across files
- ❌ Don't document every file/function — stay high-level
- ❌ Don't keep resolved issues in DYNAMICS.md

---

## 📊 Knowledge Audit Checklist

Periodically (or when requested), perform this audit:

```
□ PROJECT-ESSENCE.md: Does it still describe what the project does?
□ ARCHITECTURE.md: Do components still exist and have same responsibilities?
□ DECISIONS.md: Are decisions still valid or have they been superseded?
□ DYNAMICS.md: Are all issues still active? Are any resolved?
□ SKILL.md: Is the activation guidance still accurate?
```

---

## 🛠️ Automation Opportunities

### Scripts That Could Help

These scripts don't exist yet but could be added to `meta/scripts/`:

| Script | Purpose |
|--------|---------|
| `check-drift.rs` | Compare documented components against actual structure |
| `audit-dynamics.rs` | Check for stale issues (>30 days without update) |
| `list-decisions.rs` | Extract decision titles for quick reference |

### When to Create Scripts

If you find yourself repeating a maintenance task:
1. Consider if a script would help
2. If yes, create it in the project's scripts or tools directory
3. Document it in this file
4. Run it when appropriate

---

## 📏 Quality Standards

### Knowledge Quality Checklist

Before considering knowledge "good":

- [ ] Can someone new understand PROJECT-ESSENCE.md in 2 minutes?
- [ ] Does ARCHITECTURE.md show the big picture without implementation details?
- [ ] Are decisions in DECISIONS.md justified with rationale?
- [ ] Does DYNAMICS.md only contain actionable, current issues?
- [ ] Is every file dated with last update?

### Anti-Patterns to Avoid

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| Copy-pasting code | Becomes stale immediately | Link to source files instead |
| Documenting every file | Overwhelms and distracts | Focus on components/patterns |
| Never updating | Knowledge becomes liability | Follow triggers above |
| Over-documenting decisions | Dilutes important ones | Only non-obvious decisions |
| Keeping resolved issues | Hides actual problems | Remove when resolved |

---

## 🔗 Integration Points

### With AGENTS.md

```
AGENTS.md says: "How to work" (commands, style, rules)
.ai-context says: "What the project is" (architecture, decisions, issues)
```

Both should be read at session start. They serve different purposes and should not duplicate content.

### With Dynamic Code Exploration

```
.ai-context provides: Starting mental model
Code exploration provides: Current ground truth
```

Always verify knowledge against code. When they diverge, code is truth — update the knowledge.

---

## 📝 Changelog

| Date | Change |
|------|--------|
| 2026-04-16 | Initial maintenance guide created during .ai-context setup |

---

_This guide is itself maintained using these principles. Update it when you discover better maintenance patterns._