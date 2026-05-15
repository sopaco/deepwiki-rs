---
name: litho-document-skill
description: This skill should be used when the user asks to "generate project documentation", "analyze codebase architecture", "create C4 architecture diagrams", "document a repository", "generate technical docs", "使用 Litho 生成文档", "分析代码库架构", "生成架构文档", "为项目生成技术文档", "生成 C4 模型文档", "为这个项目写文档", "自动生成文档", "帮我分析这个代码库", or any request involving automated documentation generation for a software project. This skill enables the AI agent to autonomously analyze any codebase and produce high-quality C4 architecture documentation (Overview, Architecture, Workflow, Deep-Exploration modules, Boundary Interfaces, Database Overview) — equivalent to what deepwiki-rs produces — purely through agent reasoning and tool usage, without depending on any external binary.
version: 3.0.0
---

# Litho Document Skill（纯 Agent 版）

本 Skill 是 Litho（deepwiki-rs）的**纯 Agent 平行实现**。不依赖任何外部二进制，完全通过 Agent 的工具调用能力自主完成四阶段文档生成流水线。

**目标产出**：
- `1.概述.md` — C4 Context 图 + 项目概述 + 业务价值
- `2.架构.md` — C4 Container/Component 图 + 架构模式 + 模块职责
- `3.工作流.md` — 时序图 + 流程图 + 并发模型 + 错误处理
- `4.Deep-Exploration/` — 每个领域模块的深度研究文档
- `5.边界接口.md` — CLI/API/配置等对外接口清单
- `6.数据库概览.md` — ER 图 + 表结构（条件触发）

---

## 四阶段流水线总览

```
预处理 → 研究 → 编排 → 输出
 ↓        ↓       ↓       ↓
结构洞察   C1-C4   Markdown   文件持久化
```

每个阶段的详细执行指南在 `references/` 中，Agent 按需加载。下面只给出决策级指导。

---

## 阶段一：预处理 → 了解项目

**决策要点**：
- 根据**项目规模选择扫描策略**（见下方快速路径）
- 建立预处理报告：项目名、语言、框架、核心模块列表、README摘要
- 预处理报告是后续所有阶段的**基础上下文**，务必准确

**快速路径**（按项目规模）：

| 规模 | 判断标准 | 扫描策略 |
|------|---------|---------|
| 小 | <100 源文件 | `list_files` 递归 + `read_file` 全部核心文件 |
| 中 | 100-500 源文件 | `list_files` 仅一级目录 + `read_file` 入口+配置+README + `codebase_search` 语义搜索 |
| 大 | >500 源文件 | 仅读 README + 主配置 + 入口文件 + `view_file_outline` 核心模块 + `grep_search` 精确搜索 |

> 详细步骤见 `references/phase1-preprocessing.md`

---

## 阶段二：研究 → C4 多层级分析

**决策要点**：
- 执行顺序：**C1 → C2 → [C3 并行]**（与 deepwiki-rs 一致）
- **领域模块必须全覆盖**：`src/` 下每个子目录都识别为候选模块，用 DDD 分组（核心域/支撑域/通用域），不得遗漏
- **渐进式深度控制**：按 importance 评分分级分析
- 研究产出写入 `.litho-agent/` 临时目录持久化（见下方中间产物策略）

**并发搜索**：Step 2.3(架构) + 2.4(工作流) + 2.6(边界) 的搜索可并发调用，Step 2.5(模块深度) 必须在 2.2(领域模块) 之后

**渐进式深度**：

| importance | 分析深度 | 读取文件数 | Mermaid 图 |
|-----------|---------|----------|-----------|
| ≥7（核心域） | 深度分析 | 5+ | 完整 flowchart + 交互表格 |
| 4-6（支撑域） | 标准分析 | 3 | 精简流程图 |
| ≤3（通用域） | 简要描述 | 1-2 | 无图 |

> 详细步骤见 `references/phase2-research.md`

---

## 阶段三：编排 → 生成 Markdown 文档

**决策要点**：
- **生成顺序**：边界接口 → 概述 → 模块深度(逐个) → 架构 → 工作流 → 数据库（依赖少的先写入）
- **分章节写入大型文档**：架构和工作流分 2-3 次写入（框架 → 补充章节）
- **逐模块独立写入**：每个 Deep-Exploration 文档独立 write_to_file，写完即释放上下文
- **代码引用密度**：每模块 ≥3 文件路径、≥2 类型名、组件表每行有路径列

> 详细模板见 `references/phase3-composition.md`

---

## 阶段四：输出 → 验证与交付

**决策要点**：
- Mermaid 图表语法验证（节点 ID 仅字母数字、标签用双引号、换行用 `<br/>`）
- 每份文档末尾标注置信度评分（1-10）
- 生成执行摘要报告（文档清单、模块覆盖数、置信度表、需人工审查项）

**数据库文档触发**（满足任一即触发，否则写极简声明文件）：
- `.sql`/`.sqlproj` 文件 | `migrations/`/`sql/`/`db/`/`database/` 目录 | ORM 依赖 | DB 配置文件

> 详细验证清单见 `references/phase4-output.md`

---

## ⚠️ 中间产物持久化策略（核心！）

### 问题
Agent 单次对话上下文窗口有限。随着分析深入，早期的研究结果可能因上下文压力被「遗忘」。

### 解决方案
每完成一个研究步骤，将关键发现**持久化到 `.litho-agent/` 临时目录**，而非仅依赖对话上下文：

```
.litho-agent/
├── preprocessing.md        ← 预处理报告（阶段一产出）
├── c1-system-context.md    ← 系统上下文报告
├── c2-domain-modules.md    ← 领域模块报告
├── architecture.md         ← 架构研究报告
├── workflow.md             ← 工作流研究报告
├── boundary.md             ← 边界接口报告
├── database.md             ← 数据库报告（条件）
└── modules/                ← 各模块深度报告
    ├── llm.md
    ├── cache.md
    └── ...
```

**操作方法**：
- 每完成一个研究 Step → `write_to_file` 写入 `.litho-agent/` 对应文件
- 编排阶段需要某报告 → `read_file` 从 `.litho-agent/` 读取
- 最终输出完成后 → 删除 `.litho-agent/` 临时目录（可选保留供复查）

**关键优势**：
- 上下文压力时可以释放早期研究数据，需要时再读取
- 研究结果不丢失，即使对话很长也能保证编排阶段的数据完整性
- 与 deepwiki-rs 的 Memory 作用域机制等效

---

## 工具使用优先级

1. `codebase_search` — 语义搜索（找「做什么事」的代码）
2. `grep_search` — 精确搜索（找特定符号/类名/函数名）
3. `view_file_outline` — 快速获取文件结构（不读全量）
4. `read_file` — 深读关键文件（入口、核心模块）
5. `list_files` — 扫描目录结构

---

## 参考文档（按需加载）

- `references/phase1-preprocessing.md` — 预处理详细步骤 + 搜索策略
- `references/phase2-research.md` — 研究各 Agent 详细指南 + 输出格式
- `references/phase3-composition.md` — 文档模板 + 分章节策略 + 代码引用规范
- `references/phase4-output.md` — Mermaid 验证清单 + 置信度评分模板
- `references/doc-templates.md` — Mermaid 图表语法速查 + 类型选择指南