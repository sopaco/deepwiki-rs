# Project Essence — Litho (deepwiki-rs)

> **Stability: HIGH** | Update: Quarterly or major version changes
>
> Last reviewed: 2026-04-16

---

## What Is This Project?

Litho is an AI-powered documentation generation engine that automatically analyzes source code and generates comprehensive C4 architecture documentation. It transforms raw codebases into professional documentation with context diagrams, container diagrams, component diagrams, and code-level documentation.

---

## Why Does It Exist?

**Problem:** Technical documentation is chronically outdated, incomplete, or missing. Manual documentation requires significant effort and quickly falls behind code changes, leading to:
- New developers spending weeks understanding codebases
- Architecture decisions being lost or forgotten
- Compliance and audit failures due to missing documentation

**Solution:** Automated documentation that stays in sync with code
- Eliminates manual documentation maintenance overhead
- Captures institutional knowledge before it's lost
- Reduces onboarding time from weeks to days
- Provides consistent C4 model structure across projects

---

## Who Is This For?

| User | Use Case |
|------|----------|
| Software Developers | Onboard to unfamiliar codebases, understand dependencies and architecture patterns |
| Solution Architects | Validate implementations against blueprints, detect architecture drift, generate C4 diagrams for reviews |
| Technical Writers | Auto-generate documentation drafts, create user guides from code analysis |
| Engineering Managers | Assess technical debt, communicate system scope to stakeholders |

---

## Core Value Proposition

```
Before Litho:
  Manual documentation that's outdated, inconsistent, and time-consuming to maintain
  New developers spend weeks reading code to understand architecture
  Architecture decisions are lost when team members leave

After Litho:
  Auto-generated C4 documentation that reflects actual codebase state
  Onboarding time reduced from weeks to days
  Architecture decisions captured and preserved automatically
```

---

## What Does It Provide?

### Key Components

| Component | Purpose | Value |
|-----------|---------|-------|
| Multi-language Analyzer | Parses 12+ programming languages | Works across heterogeneous tech stacks |
| AI Research Pipeline | 8 specialized agents analyze code | Produces comprehensive, contextual documentation |
| C4 Model Generator | Creates Context, Container, Component, Code diagrams | Professional architecture visualization |
| Knowledge Integration | Mounts external docs (PDF, Markdown, SQL) | Enriches analysis with business context |
| Database Documenter | Generates ERD diagrams from SQL | Complete system documentation |

### Key Features

1. **Multi-Language Support** — Rust, Python, Java, Go, C#, JavaScript/TypeScript, PHP, Swift, Kotlin, C++, and modern frontend frameworks
2. **Dual LLM Support** — Cloud APIs (OpenAI, Claude) and local inference (Ollama) for privacy-sensitive environments
3. **External Knowledge Integration** — RAG-style document chunking for incorporating existing ADRs, domain docs, and reference materials
4. **Intelligent Caching** — MD5-based response caching reduces API costs and enables offline replay
5. **8-Language i18n** — Generates documentation in English, Chinese, Japanese, Korean, Spanish, French, German, Portuguese

---

## Key Constraints

1. **Read-Only Analysis** — System never modifies source code; strict safety guarantee for production codebases
2. **LLM Dependency** — Core analysis requires LLM API connectivity or local Ollama server
3. **Command-Line Interface** — No GUI; designed for CI/CD pipeline integration
4. **Static Analysis Only** — Does not execute or interact with runtime systems

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Documentation coverage | All significant components documented |
| Onboarding time | < 1 day for new developers |
| Documentation freshness | Auto-updated on every generation |
| Cost efficiency | 90%+ cache hit rate for re-runs |

---

## Evolution Direction

- **Short-term:** Enhanced Mermaid diagram syntax, improved error messages
- **Mid-term:** Git history analysis for architecture evolution tracking
- **Long-term:** IDE plugins, real-time documentation sync, multi-repository analysis

---

*This file captures the stable essence of the project. For architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).*