# Design Decisions — Litho (deepwiki-rs)

> Key architectural and design decisions that shape this project. Update when decisions are made or revisited.
>
> Last reviewed: 2026-04-16

---

## Decision Index

| ID | Decision | Status | Date |
|----|----------|--------|------|
| ADR-001 | Multi-Agent AI Pipeline Architecture | Active | 2024-01 |
| ADR-002 | Read-Only Analysis Constraint | Active | 2024-01 |
| ADR-003 | Dual LLM Provider Support | Active | 2024-01 |
| ADR-004 | Language-Agnostic Preprocessing | Active | 2024-02 |
| ADR-005 | File-Based Response Caching | Active | 2024-02 |
| ADR-006 | Four-Stage Processing Pipeline | Active | 2024-01 |

---

## ADR-001: Multi-Agent AI Pipeline Architecture

**Context**: How should we structure AI analysis to produce comprehensive C4 documentation? A single LLM call cannot effectively analyze complex codebases while maintaining consistency across abstraction levels.

**Decision**: Implement 8 specialized research agents that operate at different C4 abstraction levels, with staged dependency ordering.

**Rationale**: 
- Single-agent approaches produce inconsistent results across abstraction levels
- Specialized agents can focus on specific aspects (system context, domain modules, workflows, boundaries)
- Staged dependency pattern ensures high-level context (C1) guides detailed analysis (C3-C4)
- Reduces hallucination through structured, focused analysis

**Trade-offs**:
- (+) Higher quality, more consistent documentation
- (+) Parallelizable execution for performance
- (+) Easier to debug and improve individual agents
- (-) More complex orchestration logic
- (-) Higher token usage from multiple LLM calls

**Implications**: Changes to agent interfaces require careful coordination. Adding new agents must respect dependency ordering.

**Revisit if**: Single-agent models become capable enough to handle all C4 levels accurately.

---

## ADR-002: Read-Only Analysis Constraint

**Context**: Documentation tools often create fear of unintended modifications to production codebases. How can we ensure users trust the tool?

**Decision**: Enforce strict read-only filesystem access to target projects. Never modify, create, or delete source code files.

**Rationale**:
- Production safety is paramount for adoption
- Enables "run anywhere" deployment model without risk
- Simplifies security reviews and compliance
- User trust is essential for CI/CD integration

**Trade-offs**:
- (+) Complete safety guarantee for production codebases
- (+) Simplifies security and compliance discussions
- (-) Cannot implement auto-fix suggestions
- (-) Cannot write inline documentation back to source

**Implications**: All output is generated documentation only. Any future "fix" features must use separate tooling.

**Revisit if**: Users strongly request automated refactoring capabilities (would require explicit opt-in mode).

---

## ADR-003: Dual LLM Provider Support

**Context**: Different users have different constraints on data handling. Some require cloud APIs for power, others need local inference for privacy.

**Decision**: Abstract LLM integration through a unified interface supporting both cloud APIs (OpenAI, Claude) and local inference (Ollama).

**Rationale**:
- Enterprise users often have data sovereignty requirements
- Air-gapped environments need fully local solutions
- Cloud APIs provide best quality; Ollama provides best privacy
- Unified interface enables seamless switching

**Trade-offs**:
- (+) Broadest possible user coverage
- (+) Enables cost optimization (local for drafts, cloud for final)
- (-) Added abstraction complexity
- (-) Ollama feature parity requires maintenance

**Implications**: New features must consider both provider types. Testing requires both environments.

**Revisit if**: A single provider becomes clearly dominant or a new provider emerges as essential.

---

## ADR-004: Language-Agnostic Preprocessing

**Context**: Supporting 12+ programming languages with unique syntax rules. How to avoid duplicating core analysis logic?

**Decision**: Feature-based language processor architecture where language-specific modules plug into a shared core pipeline.

**Rationale**:
- Core AI pipeline is language-agnostic
- Only parsing layer differs between languages
- Maximizes code reuse and maintainability
- Easy to add new language support

**Trade-offs**:
- (+) Shared logic reduces maintenance burden
- (+) New languages require minimal code
- (-) Some language-specific optimizations are sacrificed
- (-) Abstract syntax handling can miss language-specific patterns

**Implications**: Adding languages requires implementing the `LanguageProcessor` trait. Language-specific heuristics belong in processors, not core.

**Revisit if**: Performance analysis shows per-language optimization critical.

---

## ADR-005: File-Based Response Caching

**Context**: LLM API calls are expensive and rate-limited. How can we reduce costs for iterative documentation generation?

**Decision**: Implement persistent MD5-keyed file cache for LLM responses with TTL expiration.

**Rationale**:
- Iterative runs often repeat identical analysis
- Same code + same prompt = same response (deterministic)
- Enables offline replay of previous analyses
- Provides audit trail of AI reasoning

**Trade-offs**:
- (+) Significant cost reduction (90%+ hit rate on re-runs)
- (+) Faster iteration during development
- (+) Enables offline analysis replay
- (-) Cache invalidation complexity
- (-) Disk space usage over time

**Implications**: Cache directory must be managed. Prompt changes may require cache clearing.

**Revisit if**: LLM APIs become significantly cheaper or non-deterministic.

---

## ADR-006: Four-Stage Processing Pipeline

**Context**: Documentation generation involves distinct concerns: code analysis, AI reasoning, content creation, and output validation.

**Decision**: Organize processing into four explicit stages: Preprocess → Research → Compose → Output.

**Rationale**:
- Clear separation of concerns
- Each stage has well-defined inputs and outputs
- Failures can be isolated to specific stages
- Enables parallel processing where stages permit

**Trade-offs**:
- (+) Testable individual stages
- (+) Clear error boundaries
- (+) Observable progress reporting
- (-) Latency from sequential stages
- (-) Some coupling between stages

**Implications**: Adding new processing steps requires fitting into existing stage boundaries or creating new stage.

**Revisit if**: Performance profiling suggests merging stages for speed.

---

## Guidance for Filling This Template

### What warrants a decision record?

- Choices between multiple viable alternatives
- Decisions that affect multiple components
- Non-obvious trade-offs
- Decisions that might be questioned later
- Architectural constraints

### What does NOT need a record?

- Trivial naming conventions
- Standard patterns in the tech stack
- Decisions with only one reasonable option
- Temporary choices with clear expiration

### Status Values

| Status | Meaning |
|--------|---------|
| Active | Currently in effect |
| Superseded | Replaced by ADR-XXX |
| Deprecated | No longer recommended |
| Under Review | Being reconsidered |

---

_This file captures decisions that aren't obvious from code. For implementation details, see ARCHITECTURE.md._