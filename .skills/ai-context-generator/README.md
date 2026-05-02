# AI Context Generator

> A reusable Agent Skill for creating project knowledge bases optimized for coding agents.

---

## What Is This?

This skill helps you generate a `.ai-context/` directory structure that provides coding agents (like Claude Code, Cursor, etc.) with pre-generated project knowledge. The knowledge is organized by stability tiers, making it easy for agents to understand:

- **What** the project is (PROJECT-ESSENCE.md)
- **How** components fit together (ARCHITECTURE.md)
- **Why** design decisions were made (DECISIONS.md)
- **What issues** are currently active (DYNAMICS.md)

---

## Why Use This?

**Problem:** Coding agents spend valuable tokens and time exploring code to understand project context. This exploration is repeated in every session.

**Solution:** Pre-generated knowledge that:
- Reduces onboarding time from ~10 minutes to ~30 seconds
- Preserves institutional knowledge (decisions, constraints)
- Uses ~4000 tokens vs ~50,000+ for full code exploration
- Guides agents to relevant code locations faster

---

## Quick Start

### Option 1: Use with an AI Agent

Simply tell your coding agent:

```
Use the ai-context-generator skill to setup .ai-context for this project
```

The agent will:
1. Read project files (AGENTS.md, README.md, package.json, etc.)
2. Generate the `.ai-context/` structure
3. Ask clarifying questions if needed
4. Create all necessary files

### Option 2: Manual Generation

1. Copy the `templates/` directory contents
2. Replace `{{PLACEHOLDERS}}` with your project details
3. Create the `.ai-context/` directory structure

Or tell the agent: 'Use the ai-context-generator skill to setup .ai-context for my project'
---

## Generated Structure

```
.ai-context/
├── SKILL.md                    # Entry point with activation rules
├── DYNAMICS.md                 # Active issues & constraints
├── references/
│   ├── PROJECT-ESSENCE.md      # What & why (stable)
│   ├── ARCHITECTURE.md          # Component relationships
│   └── DECISIONS.md             # Design decisions & rationale
└── meta/
    ├── MAINTENANCE.md           # How to maintain this knowledge
    ├── templates/               # (Optional) Custom templates
    └── scripts/                # (Optional) Maintenance scripts
```

---

## Stability Tiers

| Tier | File | Stability | Tokens | Update Frequency |
|------|------|-----------|--------|------------------|
| 0 | PROJECT-ESSENCE.md | High | ~500 | Quarterly |
| 1 | ARCHITECTURE.md | Medium | ~1000 | Monthly |
| 2 | DECISIONS.md | Low | ~800 | Per decision |
| 3 | DYNAMICS.md | Dynamic | ~600 | As needed |

Total budget: ~4000 tokens (within typical context limits)

---

## Files Overview

### SKILL.md
The entry point that tells agents:
- When to activate this knowledge
- Which file to read for specific needs
- How to keep knowledge updated

### references/PROJECT-ESSENCE.md
One-page summary answering:
- What is this project?
- Why does it exist?
- Who is it for?
- What does it provide?

### references/ARCHITECTURE.md
Component-level overview with:
- System diagram (ASCII or Mermaid)
- Component responsibilities
- Data flow descriptions
- Key dependencies

### references/DECISIONS.md
Architecture Decision Records (ADRs) format:
- Context (the problem)
- Decision (the choice)
- Rationale (why)
- Trade-offs
- Implications

### DYNAMICS.md
Living document tracking:
- Active issues (blockers)
- Known constraints
- Workarounds
- Recently resolved issues

### meta/MAINTENANCE.md
Guide for keeping knowledge accurate:
- What triggers updates
- How to make updates
- Quality standards
- Anti-patterns to avoid

---

## Integration with AGENTS.md

This skill complements (not replaces) `AGENTS.md`:

| File | Purpose | Author |
|------|---------|--------|
| `AGENTS.md` | How to work (commands, style, rules) | Project maintainers |
| `.ai-context/` | What the project is (architecture, decisions) | Generated |

**Both should be read at session start for optimal agent performance.**

---

## Customization

### Custom Templates

Copy templates to your project and modify:

```
your-project/
├── .ai-context/
│   └── meta/
│       └── templates/     # Override defaults here
```

The skill will use local templates if they exist.

### Custom Scripts

Add automation scripts to `meta/scripts/`:

| Script | Purpose |
|--------|---------|
| `check-drift.ts` | Detect documentation drift from code |
| `audit-dynamics.ts` | Flag stale issues |
| `generate-from-code.ts` | Auto-generate from code analysis |

---

## Best Practices

### Do:
- ✅ Generate once, maintain regularly
- ✅ Keep each file under 150 lines
- ✅ Use diagrams over paragraphs
- ✅ Update "Last updated" dates
- ✅ Remove resolved issues from DYNAMICS.md

### Don't:
- ❌ Copy-paste code snippets (link to files instead)
- ❌ Document every file/function
- ❌ Include details that change frequently
- ❌ Let knowledge go stale

---

## Example Usage

### Scenario 1: New Project Setup

```
User: Setup ai-context for my new Express.js API project

Agent:
1. Reads package.json, identifies Express.js + TypeScript
2. Scans src/ directory structure
3. Generates PROJECT-ESSENCE.md describing the API
4. Creates ARCHITECTURE.md with component diagram
5. Initializes empty DECISIONS.md and DYNAMICS.md
6. Asks about any non-obvious design choices
```

### Scenario 2: Existing Project

```
User: My project has 50k lines of code, help agents understand it faster

Agent:
1. Reads AGENTS.md, README.md, existing docs/
2. Analyzes directory structure for components
3. Extracts key architecture patterns
4. Generates concise knowledge base
5. Highlights areas needing clarification
```

---

## Compatibility

- **Agent Skills Spec**: Fully compliant with [agentskills.io](https://agentskills.io/specification)
- **Claude Code**: Works with Claude Code's skill system
- **Cursor**: Compatible with Cursor's context system
- **Other Agents**: Portable to any agent supporting Agent Skills

---

## Contributing

To improve this skill:

1. Fork and modify templates
2. Test with your projects
3. Submit improvements via PR

---

## License

MIT License — Use freely in any project.

---

## References

- [Agent Skills Specification](https://agentskills.io/specification)
- [Architecture Decision Records](https://adr.github.io/)
- [Project README Template](https://github.com/LinusBorg/project-readme-template)

---

*Generate better project knowledge. Help agents work smarter.*
