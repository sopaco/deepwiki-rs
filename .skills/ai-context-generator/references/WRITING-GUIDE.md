# Writing Guide for AI Context Knowledge Base

> Practical guidance for creating documentation that coding agents can effectively use. Last updated: 2025-01-11.

---

## Core Philosophy

**Goal:** Create knowledge that helps agents work faster, not knowledge that documents everything.

**Principle:** Agents have two sources of truth:
1. **Static Knowledge** (`.ai-context/`) — Mental model, context, rationale
2. **Dynamic Exploration** (code reading) — Current state, implementation details

Write static knowledge to complement dynamic exploration, not replace it.

---

## Writing for Agents vs Humans

| Aspect | Human Documentation | Agent Knowledge |
|--------|---------------------|-----------------|
| Detail level | Comprehensive | Minimal, strategic |
| Code examples | Plentiful | Link to files instead |
| Update frequency | Per feature | Per structural change |
| Audience | Varies | Technical, literal |
| Format flexibility | Prose-heavy | Structured, scannable |

---

## The Token Budget

Each file has an implicit token budget. Respect it.

| File | Budget | Why |
|------|--------|-----|
| PROJECT-ESSENCE.md | ~500 tokens | Must read every session |
| ARCHITECTURE.md | ~1000 tokens | Read when working across components |
| DECISIONS.md | ~800 tokens | Read when changing patterns |
| DYNAMICS.md | ~600 tokens | Read when debugging |
| SKILL.md | ~400 tokens | Loaded for discovery |

**Total: ~3300 tokens** — Less than one typical function's worth of context.

---

## Writing Techniques

### 1. Prefer Diagrams Over Paragraphs

❌ **Bad:**
```
The system has a gateway that loads plugins. The gateway talks to cortex-mem-service 
via HTTP. The service then connects to Qdrant for vector storage and the filesystem 
for markdown storage.
```

✅ **Good:**
```
┌─────────────┐
│   Gateway   │
└──────┬──────┘
       │ HTTP
       ▼
┌─────────────────┐
│ cortex-mem-svc  │
└────┬───────┬────┘
     │       │
     ▼       ▼
  Qdrant   Files
```

### 2. Link, Don't Copy

❌ **Bad:**
```markdown
The config format is:
```toml
[server]
port = 8085
host = "localhost"
```

✅ **Good:**
```markdown
Configuration format: see `config.example.toml` or `src/config.ts` for schema.
```

**Why:** Code copies become stale. Links stay valid.

### 3. State the Non-Obvious

❌ **Bad:**
```markdown
We use TypeScript for type safety.
```

✅ **Good:**
```markdown
We use TypeScript strict mode. Avoid `any` — we prefer runtime validation via Zod 
instead of type assertions.
```

**Why:** The first is obvious to any TypeScript user. The second captures project-specific practices.

### 4. Use Tables for Comparisons

❌ **Bad:**
```markdown
The memory plugin is for explicit calls while the context engine handles automatic 
lifecycle hooks. They can both be installed together.
```

✅ **Good:**
```markdown
| Aspect | Memory Plugin | Context Engine |
|--------|---------------|----------------|
| Trigger | Explicit tool call | Automatic lifecycle |
| Control | Full | None |
| Co-install | Yes | Yes |
```

### 5. Date Everything

Every file should have a "Last updated" or "Last reviewed" date at the top.

**Why:** Agents need to know if knowledge might be stale.

---

## Anti-Patterns

### Anti-Pattern 1: The Encyclopedia

```markdown
## File Structure

/src/
  /components/
    /Button/
      Button.tsx        # Button component
      Button.test.tsx  # Tests
      styles.css       # Styles
    /Input/
      Input.tsx
      ...
```

**Problem:** Becomes wrong immediately. Use `find_path` instead.

**Fix:**
```markdown
Components live in `src/components/`. Each is self-contained with its own directory.
```

---

### Anti-Pattern 2: The Tutorial

```markdown
## How to Add a New Tool

1. Create a new file in `src/tools/`
2. Import the tool interface
3. Implement the execute method
4. Register in tool-registry.ts
5. Add tests
...
```

**Problem:** This is a procedure, not knowledge. Procedures belong in AGENTS.md.

**Fix:**
```markdown
Tools are registered in `tool-registry.ts`. Each tool implements `ToolInterface`.
```

---

### Anti-Pattern 3: The Decision Dump

```markdown
## ADR-042: Use tabs for indentation

We decided to use tabs because...

## ADR-043: Use semicolons

We decided to use semicolons because...
```

**Problem:** Not all decisions matter equally.

**Fix:** Only document decisions that:
- Have significant trade-offs
- Affect multiple components
- Might be questioned later
- Have non-obvious rationale

---

### Anti-Pattern 4: The Issue Graveyard

```markdown
## Fixed in 2024-01

The config parser was broken...

## Fixed in 2024-02

The API endpoint was wrong...
```

**Problem:** Resolved issues hide active ones.

**Fix:** DYNAMICS.md should only contain:
- Active blockers
- Known constraints
- Items under consideration
- Recently resolved (last 2 weeks, brief)

---

## What to Put Where

### PROJECT-ESSENCE.md — "What and Why"

- What is this? (one sentence)
- Why does it exist?
- Who uses it?
- Key value proposition
- Core constraints
- Success metrics

**Not:** Technical details, architecture, decisions

---

### ARCHITECTURE.md — "How Things Connect"

- System diagram
- Component responsibilities
- Data flow
- Key patterns
- Dependencies
- Configuration layers

**Not:** Implementation details, every file, API specs

---

### DECISIONS.md — "Why We Chose This"

- Non-obvious choices
- Trade-offs accepted
- Constraints embraced
- Things we might revisit

**Not:** Naming conventions, style choices, one-off decisions

---

### DYNAMICS.md — "What's Happening Now"

- Active blockers
- Workarounds in use
- Temporary constraints
- Items under review

**Not:** History, resolved issues, wishlists

---

## Review Checklist

Before finalizing any file:

```
□ Is this knowledge that code exploration can't easily reveal?
□ Would this still be accurate in 3 months?
□ Is the token budget respected?
□ Is there a "Last updated" date?
□ Did I link instead of copy?
□ Did I state the non-obvious?
□ Is this scannable (tables, lists, diagrams)?
```

---

## Example Transformations

### Before (Human-Style):

```markdown
# Architecture

This document describes the architecture of our system. The system is built using 
TypeScript and runs on Node.js. We use Express for the HTTP server and PostgreSQL 
for the database. The frontend is built with React.

The main components are:
- API Layer: Handles HTTP requests
- Business Logic: Contains the core functionality
- Data Layer: Manages database access

We chose PostgreSQL because it's reliable and has good JSON support.
```

### After (Agent-Optimized):

```markdown
# Architecture

> Last updated: 2025-01-11

## System Diagram

```
┌─────────────┐     ┌─────────────┐     ┌───────────┐
│   Express   │────▶│  Business   │────▶│ PostgreSQL│
│   :3000     │     │   Logic     │     │   :5432   │
└─────────────┘     └─────────────┘     └───────────┘
       │
       ▼
┌─────────────┐
│   React     │
│   Client    │
└─────────────┘
```

## Components

| Component | Entry Point | Responsibility |
|-----------|-------------|----------------|
| API Layer | `src/api/` | HTTP routing, auth |
| Business Logic | `src/core/` | Domain operations |
| Data Layer | `src/db/` | Queries, migrations |

## Key Decisions

See [DECISIONS.md](DECISIONS.md) for rationale on PostgreSQL, Express, etc.
```

---

## Summary

1. **Minimize tokens** — Every token costs attention
2. **Link, don't copy** — Code changes, links stay valid
3. **State the non-obvious** — Obvious things don't need documentation
4. **Structure for scanning** — Tables, lists, diagrams
5. **Date everything** — Agents need to assess staleness
6. **Separate concerns** — Each file has a purpose

---

_Writing for agents is writing for a literal, token-constrained, but technically competent reader who prefers structure over prose._