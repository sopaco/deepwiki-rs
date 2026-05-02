# AGENTS.md

> Project guide for coding agents. Keep this file focused on stable, high-level knowledge. Volatile details go in `.ai-context/`.

## .ai-context/ Directory

The `.ai-context/` folder contains supplementary documents that are **more likely to change** as the project evolves:

| File | Content |
|------|---------|
| [`references/ARCHITECTURE.md`](.ai-context/references/ARCHITECTURE.md) | Detailed component relationships, API dependencies, and design rationale |
| [`DYNAMICS.md`](.ai-context/DYNAMICS.md) | Current known issues, workarounds, and open TODOs |

### When to Update .ai-context/

Update files in `.ai-context/` when:

1. **Architecture changes** — New components removed, APIs added/removed, or data flow modified
2. **New known issues discovered** — During development or testing, if you find a bug or limitation that isn't obvious from the code
3. **API dependency changes** — New cortex-mem-service endpoints required, or existing ones deprecated
4. **Design decisions revisited** — Key tradeoffs re-evaluated (e.g., switching `ownsCompaction` from `false` to `true`)

**Do NOT update .ai-context/ for**:
- Minor refactors, variable renames, or code formatting
- Dependency version bumps (patch/minor)
- Documentation typos or wording improvements

Keep `.ai-context/` focused on information that would **meaningfully change how a coding agent understands or works with the project**.
