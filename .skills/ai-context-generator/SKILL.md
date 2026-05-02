---
name: ai-context-generator
description: |
  Generates .ai-context knowledge base for coding agents. Activate when: (1) setting up a new project for AI-assisted development, (2) user asks to "create project knowledge" or "setup ai-context", (3) existing .ai-context needs regeneration. Creates tiered documentation structure optimized for agent comprehension and token efficiency.
---

# AI Context Generator

> A reusable skill for creating project knowledge bases that help coding agents work faster and smarter.

---

## 🎯 When to Use This Skill

**Activate when:**
- Setting up a new project for AI-assisted development
- User requests: "create ai-context", "setup project knowledge", "generate .ai-context"
- Existing `.ai-context` is outdated and needs regeneration
- After major project restructuring

**Do NOT activate when:**
- Project already has fresh `.ai-context` (check `SKILL.md` date)
- User asks for unrelated documentation
- Simple code tasks with clear existing context

---

## 📋 What This Skill Generates

Creates a `.ai-context/` directory with:

```
.ai-context/
├── SKILL.md                    # Entry point with activation rules
├── DYNAMICS.md                 # Active issues & constraints (Dynamic)
├── references/
│   ├── PROJECT-ESSENCE.md      # What & why (High stability)
│   ├── ARCHITECTURE.md         # Component relationships (Medium stability)
│   └── DECISIONS.md            # Design decisions (Update on change)
└── meta/
    ├── MAINTENANCE.md          # How to maintain this knowledge
    ├── templates/              # (Optional) Custom templates
    └── scripts/                # (Optional) Maintenance scripts
```

### Stability Tiers

| Tier | File | Update Frequency | Token Budget |
|------|------|-------------------|--------------|
| 0 | PROJECT-ESSENCE.md | Quarterly / Major version | ~500 tokens |
| 1 | ARCHITECTURE.md | Monthly / Sprint | ~1000 tokens |
| 2 | DECISIONS.md | Per decision change | ~800 tokens |
| 3 | DYNAMICS.md | As needed (issues) | ~600 tokens |

---

## 🔧 Generation Process

### Step 1: Gather Project Intelligence

Before generating, collect:

```
□ Read AGENTS.md (if exists) — operational rules
□ Read README.md — user-facing description
□ Read package.json — dependencies, scripts, entry points
□ Scan directory structure — identify components
□ Read docs/ or litho.docs/ — existing documentation
□ Identify key source files — main entry points
□ Note technology stack — frameworks, languages, platforms
```

### Step 2: Extract Knowledge

**For PROJECT-ESSENCE.md:**
- What is this project? (one sentence)
- Why does it exist? (problem/solution)
- Who is it for? (target users)
- What does it provide? (key features)
- Core constraints? (security, compatibility)

**For ARCHITECTURE.md:**
- System diagram (ASCII or Mermaid)
- Component responsibilities
- Data flow between components
- Key dependencies
- Important patterns

**For DECISIONS.md:**
- Non-obvious design choices
- Trade-offs made
- Constraints accepted
- Decisions that might be revisited

**For DYNAMICS.md:**
- Current blockers
- Known workarounds
- Temporary constraints
- Recently resolved issues (brief)

### Step 3: Generate Files

Use templates from `templates/` directory:

1. Start with `SKILL.md` — entry point with activation rules
2. Generate `references/PROJECT-ESSENCE.md` — core identity
3. Generate `references/ARCHITECTURE.md` — component map
4. Generate `references/DECISIONS.md` — design rationale
5. Generate `DYNAMICS.md` — active issues
6. Generate `meta/MAINTENANCE.md` — upkeep guide

### Step 4: Validate Quality

```
□ SKILL.md has clear activation triggers
□ PROJECT-ESSENCE.md readable in 2 minutes
□ ARCHITECTURE.md shows big picture (no code)
□ DECISIONS.md justified with rationale
□ DYNAMICS.md only contains current issues
□ All files dated at top
□ Total token budget < 4000 tokens
```

---

## 📝 Writing Principles

### Do:
- ✅ Write for someone who knows nothing about the project
- ✅ Use diagrams over paragraphs
- ✅ Focus on "why" not "how"
- ✅ Keep files under 150 lines each
- ✅ Link between related sections
- ✅ Include "Last updated" dates

### Don't:
- ❌ Copy-paste code snippets (link to files instead)
- ❌ Document every file/function
- ❌ Include details that change frequently
- ❌ Duplicate content across files
- ❌ Use jargon without context

---

## 🔄 Integration with AGENTS.md

```
AGENTS.md = "How to work" (commands, style, rules)
.ai-context = "What the project is" (architecture, decisions, issues)
```

Both should be read at session start. They serve different purposes and should not overlap.

---

## 📚 Template Reference

Templates are provided in `templates/`:

| Template | Purpose |
|----------|---------|
| `skill.md.tmpl` | SKILL.md with placeholder prompts |
| `essence.md.tmpl` | PROJECT-ESSENCE.md structure |
| `architecture.md.tmpl` | ARCHITECTURE.md with diagram prompts |
| `decisions.md.tmpl` | DECISIONS.md with ADR format |
| `dynamics.md.tmpl` | DYNAMICS.md with status tracking |
| `maintenance.md.tmpl` | MAINTENANCE.md guide |

---

## 🛠️ Automation Scripts

Scripts in `scripts/` can help with:

| Script | Purpose |
|--------|---------|
| `generate.ts` | Interactive generation from templates |
| `check-drift.ts` | Compare documented vs actual structure |
| `audit-dynamics.ts` | Flag stale issues (>30 days) |

---

## 💡 Example Usage

**User:** "Setup ai-context for my project"

**Agent:**
1. Activate this skill
2. Read AGENTS.md, README.md, package.json
3. Scan directory structure
4. Generate each file using templates
5. Ask clarifying questions if needed:
   - "What's the main problem this project solves?"
   - "Any non-obvious design decisions I should know about?"
   - "Current blockers or workarounds?"

---

## ⚠️ Important Notes

- Generated knowledge is a **starting point**, not final truth
- Agent should verify against actual code during first session
- User should review generated content for accuracy
- Schedule regular audits (monthly recommended)

---

## 📖 References

- [Agent Skills Specification](https://agentskills.io/specification)
- [Architecture Decision Records](https://adr.github.io/)
- [Writing Readable Docs](references/WRITING-GUIDE.md)

---

*This skill creates knowledge bases optimized for AI agents. For questions or improvements, see MAINTENANCE.md.*