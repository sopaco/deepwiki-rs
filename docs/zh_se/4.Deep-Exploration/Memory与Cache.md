# Memory 与 Cache 模块

## Memory 模块 (memory)

Memory 是 Litho 流水线中的"快递站"——它用作用域(scope)机制管理 13 个 Agent 之间的数据共享。每个 Agent 把自己的分析结果写入专属的 scope（如 `"system_context"`、`"domain_modules"`），后续 Agent 通过 scope 精准提取数据。这就像快递站给每个收件人分配独立储物柜——互不干扰，精准投递。

### 核心功能

- **Scoped KV 存储**：`Memory::store(scope, key, value)` 和 `Memory::get(scope, key)` 提供作用域化的读写操作。scope 对应 Agent 类型（如 SystemContextResearcher 使用 `"system_context"` scope），key 对应数据类型（如 `"result"`）。
- **访问统计**：`MemoryMetadata` 记录每个 scope 的创建时间、更新时间、访问次数、数据大小——这些信息帮助评估 Memory 的使用效率。
- **线程安全**：Memory 通过 `Arc<RwLock<Memory>>` 在 GeneratorContext 中共享，RwLock 允许多个 Agent 同时读取，只在写入时独占锁。

### 关键数据结构

| 类型 | 路径 | 用途 |
|------|------|------|
| `Memory` | `src/memory/mod.rs` | 作用域化KV存储——Agent间数据共享中枢 |
| `MemoryMetadata` | `src/memory/mod.rs` | 元数据——记录访问统计和数据大小 |

### Memory Scope 映射

| Agent 类型 | Scope Key | 存储的数据 |
|-----------|-----------|----------|
| PreProcessAgent | `preprocess` | ProjectStructure, DirectoryDossiers, CodeInsights |
| SystemContextResearcher | `system_context` | 系统上下文分析结果 |
| DomainModulesDetector | `domain_modules` | 领域模块列表、importance评分 |
| ArchitectureResearcher | `architecture` | 架构模式、设计原则 |
| WorkflowResearcher | `workflow` | 工作流分析结果 |
| BoundaryAnalyzer | `boundary` | CLI接口、配置结构 |
| KeyModulesInsight | `key_modules` | 各模块深度洞察 |
| DatabaseOverviewAnalyzer | `database_overview` | 数据库架构分析 |

---

## Cache 模块 (cache)

CacheManager 是 Litho 的"记账本"——它记录哪些 LLM 调用已经做过，避免重复花钱。核心机制是用 MD5(Prompt + Model) 作为缓存键，相同请求直接返回缓存结果，跳过 API 调用。

### 核心功能

- **Prompt 哈希缓存**：`CacheManager::hash_prompt()` 用 MD5 对 Prompt 内容和模型名称计算哈希，作为缓存唯一标识——确保"同样的输入一定得到同样的缓存结果"。
- **持久化缓存**：缓存数据存储在 `.litho/cache/` 目录下的 JSON 文件中，跨运行周期持久化。过期策略基于 `expire_hours`（默认 168 小时 = 7 天）。
- **性能监控**：`CachePerformanceMonitor` 实时记录缓存命中率、按 category 分类的统计信息——这些数据出现在执行摘要报告中。
- **Token 使用追踪**：`CacheEntry` 记录每次 LLM 调用的 token 消耗和模型名称——帮助用户评估 LLM 使用成本。
- **Prompt 压缩缓存**：除了 LLM 调用结果缓存，还有独立的压缩缓存——相同内容的压缩结果也会被缓存，避免重复压缩。

### 关键数据结构

| 类型 | 路径 | 用途 |
|------|------|------|
| `CacheManager` | `src/cache/mod.rs` | 缓存管理——MD5哈希缓存+性能监控 |
| `CacheEntry<T>` | `src/cache/mod.rs` | 缓存条目——数据+时间戳+哈希+token使用 |
| `CachePerformanceMonitor` | `src/cache/performance_monitor.rs` | 性能监控——命中率+分类统计 |
| `CachePerformanceReport` | `src/cache/performance_monitor.rs` | 性能报告——用于执行摘要 |

---

> **置信度评分**：8/10 — Memory 和 Cache 的 scope 映射和功能描述基于代码的直接分析。