# Architecture — Litho (deepwiki-rs)

> How components fit together. Last updated: 2026-04-16.
>
> **Update this when:** New component added, responsibilities shift, data flow changes.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Litho CLI Entry Point                            │
│                                  (src/main.rs)                                │
│                                                                               │
│  ┌─────────────┐    ┌─────────────────┐    ┌──────────────────────────────┐  │
│  │   CLI.rs    │───▶│   Config.rs     │───▶│        i18n.rs               │  │
│  │ (clap)      │    │ (TOML parsing)  │    │ (8 languages)                │  │
│  └─────────────┘    └─────────────────┘    └──────────────────────────────┘  │
│           │                                                                   │
│           ▼                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │                        Generator Workflow                                 │ │
│ │                      (src/generator/workflow.rs)                         │ │
│ │                                                                           │ │
│ │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐  │ │
│ │   │  Preprocess │──▶│   Research   │──▶│   Compose   │──▶│   Outlet    │  │ │
│ │   │   Stage     │   │    Stage     │   │    Stage    │   │   Stage     │  │ │
│ │   └─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘  │ │
│ │          │                  │                 │                 │        │ │
│ │          ▼                  ▼                 ▼                 ▼        │ │
│ │   ┌─────────────────────────────────────────────────────────────────┐   │ │
│ │   │                    Memory & Cache Layer                         │   │ │
│ │   │              (src/memory + src/cache)                           │   │ │
│ │   └─────────────────────────────────────────────────────────────────┘   │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                    │                                    │
                    ▼                                    ▼
        ┌──────────────────────┐            ┌──────────────────────┐
        │    LLM Integration   │            │  Knowledge Sync      │
        │    (src/llm/)        │            │ (src/integrations/)  │
        │                      │            │                      │
        │  ┌────────────────┐ │            │  • local_docs.rs     │
        │  │ Cloud Client   │ │            │  • knowledge_sync.rs │
        │  │ (OpenAI/Claude)│ │            │                      │
        │  └────────────────┘ │            └──────────────────────┘
        │  ┌────────────────┐ │
        │  │ Ollama Client  │ │
        │  │ (Local)        │ │
        │  └────────────────┘ │
        └──────────────────────┘
```

---

## Four-Stage Processing Pipeline

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                         Phase 1: Preprocessing                                 │
│                        (src/generator/preprocess/)                             │
│                                                                               │
│  Input: Source Code Files                                                     │
│     │                                                                         │
│     ▼                                                                         │
│  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐                │
│  │    Structure   │──▶│    Original    │──▶│     Code       │                │
│  │    Extractor   │   │   Document     │   │    Analysis    │                │
│  │                │   │   Extractor    │   │     Agent      │                │
│  └────────────────┘   └────────────────┘   └────────────────┘                │
│         │                    │                     │                        │
│         ▼                    ▼                     ▼                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                     CodeInsight Objects                                  │ │
│  │              (file structures, dependencies, annotations)               │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                          Phase 2: Research                                    │
│                         (src/generator/research/)                             │
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                    Research Orchestrator                                  │ │
│  │              (step_forward_agent.rs + agent_executor.rs)                │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                          │                                                    │
│         ┌────────────────┼────────────────┬────────────────┐                │
│         ▼                ▼                ▼                ▼                │
│  ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐          │
│  │  System    │   │   Domain   │   │ Workflow   │   │  Boundary  │          │
│  │  Context   │   │  Module    │   │ Researcher │   │  Analyzer  │          │
│  │ Researcher │   │  Detector  │   │            │   │            │          │
│  └────────────┘   └────────────┘   └────────────┘   └────────────┘          │
│         │                │                │                │                │
│         └────────────────┴────────────────┴────────────────┘                │
│                                   │                                          │
│                                   ▼                                          │
│                         ┌─────────────────┐                                 │
│                         │  Key Module      │                                 │
│                         │  Insight Officer │                                 │
│                         └─────────────────┘                                 │
│                                   │                                          │
│                                   ▼                                          │
│                    Structured Research Reports (JSON)                        │
└───────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                         Phase 3: Composition                                   │
│                         (src/generator/compose/)                              │
│                                                                               │
│  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐                │
│  │   Overview     │   │  Architecture  │   │   Workflow      │                │
│  │   Editor       │   │   Editor      │   │   Editor        │                │
│  └────────────────┘   └────────────────┘   └────────────────┘                │
│         │                    │                     │                        │
│         └────────────────────┼─────────────────────┘                        │
│                              ▼                                               │
│                    ┌────────────────────┐                                   │
│                    │  Mermaid Diagram    │                                   │
│                    │  Synthesis          │                                   │
│                    └────────────────────┘                                   │
│                              │                                               │
│                              ▼                                               │
│                  Markdown Documents + Mermaid Diagrams                      │
└───────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                           Phase 4: Output                                      │
│                          (src/generator/outlet/)                              │
│                                                                               │
│  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐                │
│  │   Mermaid      │──▶│   Document      │──▶│   File         │                │
│  │   Fixer        │   │   Integrity     │   │   Persistence  │                │
│  │                │   │   Check         │   │                │                │
│  └────────────────┘   └────────────────┘   └────────────────┘                │
│                                                    │                          │
│                                                    ▼                          │
│                                    ┌───────────────────────────┐              │
│                                    │   Output Directory        │              │
│                                    │   (docs/ structure)       │              │
│                                    └───────────────────────────┘              │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## Component Responsibilities

### CLI Layer (`src/cli.rs`)
- **Entry:** `src/cli.rs`
- **Purpose:** Command-line argument parsing and routing
- **Key behaviors:**
  - Parses 20+ CLI options using clap
  - Routes to appropriate workflow based on subcommand
  - Handles help, version, and error output

### Configuration (`src/config.rs`)
- **Entry:** `src/config.rs`
- **Purpose:** Load and merge configuration from multiple sources
- **Key behaviors:**
  - Reads `litho.toml` configuration file
  - Supports environment variable overrides
  - Validates LLM provider settings

### Generator Core (`src/generator/`)
- **Entry:** `src/generator/workflow.rs`
- **Purpose:** Orchestrates the four-stage documentation pipeline
- **Key behaviors:**
  - Manages pipeline state transitions
  - Coordinates between stages
  - Handles error recovery and retries

### Preprocess Stage (`src/generator/preprocess/`)
- **Purpose:** Extract code structure and insights from source files
- **Key behaviors:**
  - Multi-language parsing (12+ languages)
  - Dependency extraction
  - Code insight generation via LLM
  - Initializes agent memory chunks

### Research Stage (`src/generator/research/`)
- **Purpose:** AI-powered architecture analysis
- **Key behaviors:**
  - 8 specialized research agents
  - ReAct reasoning loop
  - Reads/writes to agent memory
  - Produces structured JSON reports

### Compose Stage (`src/generator/compose/`)
- **Purpose:** Generate documentation from research data
- **Key behaviors:**
  - Template-based markdown generation
  - Mermaid diagram synthesis
  - i18n localization

### Outlet Stage (`src/generator/outlet/`)
- **Purpose:** Finalize and persist documentation
- **Key behaviors:**
  - Mermaid syntax validation and repair
  - Document integrity checking
  - File system persistence

### LLM Integration (`src/llm/`)
- **Entry:** `src/llm/mod.rs`
- **Purpose:** Abstract LLM provider communication
- **Key behaviors:**
  - Cloud API clients (OpenAI, Claude)
  - Local Ollama client
  - Token management and retry logic
  - Tool execution framework

### Cache System (`src/cache/`)
- **Entry:** `src/cache/mod.rs`
- **Purpose:** Reduce API costs through response caching
- **Key behaviors:**
  - MD5-keyed file-based cache
  - TTL expiration
  - Performance monitoring
  - Cache hit/miss statistics

### Knowledge Integration (`src/integrations/`)
- **Entry:** `src/integrations/knowledge_sync.rs`
- **Purpose:** Incorporate external documentation into analysis
- **Key behaviors:**
  - PDF, Markdown, SQL file ingestion
  - Semantic chunking
  - Knowledge base synchronization
  - RAG-style retrieval

---

## Data Flow

### Main Generation Flow
```
CLI Command
    → Load Configuration (litho.toml)
    → Initialize GeneratorContext
    → Phase 1: Preprocess (scan codebase)
    → Phase 2: Research (AI agents analyze)
    → Phase 3: Compose (generate markdown)
    → Phase 4: Output (validate & persist)
    → Return output path
```

### LLM Request Flow
```
Agent needs analysis
    → Check cache (MD5 lookup)
    → If miss: Send to LLM provider
    → Parse structured JSON response
    → Cache response for future
    → Return to agent
```

### Knowledge Sync Flow
```
CLI: litho sync-knowledge
    → Scan configured paths
    → Detect changed files (mtime)
    → Chunk documents (semantic)
    → Index in memory store
    → Available for RAG retrieval
```

---

## Key Design Patterns

### 1. Step-Forward Agent Pattern
- **Where:** `src/generator/step_forward_agent.rs`
- **Why:** Enables staged, dependency-aware analysis
- **How it works:** Each agent declares data sources and output schema; orchestrator runs agents in dependency order

### 2. Pipeline Architecture
- **Where:** `src/generator/workflow.rs`
- **Why:** Clear separation of concerns; testable stages
- **How it works:** Four distinct phases with well-defined inputs/outputs; each phase can fail independently

### 3. Repository Pattern
- **Where:** `src/cache/mod.rs`, `src/integrations/local_docs.rs`
- **Why:** Abstract storage implementation; testable without real filesystem
- **How it works:** Traits define storage interface; implementations handle specifics

---

## Dependencies

| Package | Purpose | Version Constraint |
|---------|---------|-------------------|
| rig-core | LLM abstraction | 0.34 |
| tokio | Async runtime | 1.47 |
| clap | CLI parsing | 4.5 |
| serde | Serialization | 1.0 |
| anyhow | Error handling | 1.0 |
| reqwest | HTTP client | 0.12 |
| walkdir | File traversal | 2.5 |

---

## Configuration Layers

```
CLI Arguments (highest priority)
    ↓
Environment Variables (LITHO_*)
    ↓
litho.toml (project config)
    ↓
Default values (lowest priority)
```

---

*This file describes component relationships. For detailed implementation, explore the source code or use `grep`.*