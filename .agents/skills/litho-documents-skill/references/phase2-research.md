# 阶段二：研究详细执行指南

## 研究策略总览

研究阶段模拟 deepwiki-rs 的 8 个专业 Agent，通过顺序 + 并行策略执行：

```
C1 系统上下文 → C2 领域模块 → [C2 架构 + C2 工作流 + C3 关键模块 + C3 边界 + C3 数据库(条件)]
```

高层研究为低层研究提供上下文，确保分析的一致性和深度。

---

## Step 2.1：系统上下文研究（SystemContextResearcher）

### 目标
建立 C4 Context 图所需的所有信息：系统边界、用户角色、外部依赖。

### 分析问题清单

**关于用户**：
- [ ] 谁是这个系统的直接用户？（开发者/最终用户/其他系统）
- [ ] 不同用户角色对系统有什么不同的使用方式？
- [ ] 用户通过什么接口使用系统（CLI/Web UI/SDK/API）？

**关于外部系统**：
- [ ] 系统依赖哪些外部服务？（数据库/第三方 API/消息队列/LLM 等）
- [ ] 系统是否被其他系统调用？（作为上游还是下游？）
- [ ] 有哪些外部文件系统或存储依赖？

**关于业务价值**：
- [ ] 系统解决的核心业务问题是什么？
- [ ] 不使用这个系统，用户需要手动做什么？
- [ ] 系统的主要输入和输出是什么？

### 搜索策略
```
codebase_search: "外部 API 调用" / "HTTP client" / "database connection"
grep_search: "reqwest::" / "axios" / "fetch(" / "http.Get" / "requests.get"
grep_search: "env::" / "dotenv" / "os.Getenv" / "process.env"  （找环境变量，推断外部依赖）
```

### 输出格式：C1 系统上下文报告

```markdown
## C1 系统上下文报告

### 系统定义
{系统名称}是一个{系统类型}，用于{核心功能描述，1-2 句话}。

### 用户角色
| 角色 | 描述 | 主要使用场景 |
|------|------|------------|
| {角色1} | {描述} | {场景} |

### 外部系统依赖
| 外部系统 | 交互类型 | 数据流向 | 关键性 |
|---------|---------|---------|-------|
| {系统1} | API/DB/文件 | 入/出/双向 | 高/中/低 |

### 业务价值
- **解决的核心问题**：{描述}
- **主要输入**：{描述}
- **主要输出**：{描述}

### 系统边界（范围内）
- {能力1}
- {能力2}

### 系统边界（范围外）
- {明确不做的事1}
- {明确不做的事2}
```

---

## Step 2.2：领域模块检测（DomainModulesDetector）

### 目标
运用 DDD 思想识别系统内的限界上下文和模块边界，建立模块关系图。

### 分析问题清单
- [ ] 系统中有哪些独立的业务能力域（领域）？
- [ ] 每个领域的核心职责是什么？
- [ ] 领域之间的依赖关系如何？（谁依赖谁？）
- [ ] 哪些领域是核心域（业务核心）？哪些是支撑域？哪些是通用域？

### 识别方法
1. **从目录结构识别**：`src/`（或等效主源码目录）下的**每个子目录**都必须被视为候选领域模块，不得选择性遗漏
2. **从代码组织识别**：`mod.rs` / `index.ts` / `__init__.py` 声明了模块的公共接口
3. **从依赖关系识别**：使用 `grep_search` 追踪 `use` / `import` 语句，画出依赖图
4. **从命名语义识别**：`cache`/`llm`/`memory`/`research`/`compose` 等命名往往直接揭示职责
5. **DDD 语义分组**：多个小目录可能归入同一领域（如 `cache/` + `performance_monitor/` → "缓存领域"）

### 全覆盖规则（⚠️ 关键！）

deepwiki-rs 的 DomainModulesDetector 通过 LLM 自动检测所有领域模块并逐一分析。Agent 必须达到同等覆盖度：

**步骤 A：枚举所有候选模块**
```
扫描 src/ 下所有一级子目录
→ 列出完整目录清单
→ 排除纯测试目录（*_test/、*_spec/）和纯辅助目录（*_generated/）
→ 剩余每个目录 = 一个候选领域模块
```

**步骤 B：DDD 语义分组**
```
将候选模块按 DDD 分类：
- 核心域（Core）：直接贡献业务价值的模块（如 workflow、research、compose、preprocess）
- 支撑域（Supporting）：为核心域提供技术支撑的模块（如 llm、integrations、cache）
- 通用域（Generic）：通用基础设施（如 types、utils、memory、i18n、config）
```

**步骤 C：子模块识别**
```
对每个领域模块：
- 列出模块内的子目录和关键文件
- 提取每个子模块的职责、核心函数、代码路径
- 标注 importance (1-10) 和 complexity (1-10)
```

**步骤 D：领域间关系**
```
绘制模块依赖关系图（Mermaid graph）
识别业务流程（business_flows）：跨多个领域的核心数据流
```

### 搜索策略
```bash
# 找模块导出（识别每个模块的公共 API）
grep_search: "pub use" / "export { " / "__all__"

# 找模块间依赖（识别依赖方向）
grep_search: "use crate::{模块名}" / "from ..{模块名}" / "import * from './{模块名}'"

# 读取每个模块的 mod.rs 或 index.ts
read_file: "src/{module}/mod.rs" 或 "src/{module}/index.ts"
```

### 输出格式：领域模块报告（对齐 deepwiki-rs DomainModulesDetector）

```markdown
## 领域模块报告

### 识别到的领域模块（完整列表，不得遗漏）

| 模块名称 | 路径 | 核心职责 | 类型(核心/支撑/通用) | 重要性(1-10) | 复杂度(1-10) |
|---------|------|---------|------|------|------|
| {模块1} | `src/xxx/` | {描述} | 核心域 | 8 | 7 |
| {模块2} | `src/yyy/` | {描述} | 支撑域 | 5 | 4 |
| {所有模块...} | | | | | |

### 领域间关系
```mermaid
graph TD
  A["核心生成<br/>generator"] --> B["预处理<br/>preprocess"]
  A --> C["研究<br/>research"]
  A --> D["编排<br/>compose"]
  A --> E["输出<br/>outlet"]
  C --> F["LLM集成<br/>llm"]
  D --> F
  {所有关系...}
```

### 业务流程（Business Flows）
| 流程名称 | 描述 | 涉及领域 | 入口点 | 重要性 |
|---------|------|---------|-------|-------|
| {流程1} | {描述} | {领域1,领域2,...} | {入口函数} | {1-10} |

### 各模块详情（对齐 deepwiki-rs 的 sub_modules 结构）

#### {模块1名称}
- **路径**：`src/xxx/`
- **职责**：{描述}
- **核心抽象**：{接口/Trait 名称列表}
- **子模块**：
  - {子模块1} (`src/xxx/sub1.rs`)：{职责}，关键函数：{fn1, fn2}
  - {子模块2} (`src/xxx/sub2/`)：{职责}，关键函数：{fn3, fn4}
- **依赖的模块**：{列表}
- **被依赖的模块**：{列表}
- **重要性评分**：{1-10}

#### {模块2名称}
{同样的结构...}

#### ...（覆盖所有识别到的模块）
```

---

## Step 2.3：架构深度研究（ArchitectureResearcher）

### 目标
识别整体架构模式、核心技术选型、关键设计决策。

### 架构模式识别策略

**分层架构**迹象：
- 目录：`controller`/`service`/`repository`/`model`
- 严格的单向依赖

**微服务**迹象：
- 多个独立服务目录，各自有 main 入口
- 服务间通过 HTTP/gRPC/消息队列通信

**事件驱动**迹象：
- `event`/`handler`/`subscriber`/`publisher` 等命名
- 事件总线/消息队列集成

**管道-过滤器（Pipeline-Filter）**迹象：
- `pipeline`/`stage`/`phase`/`step`/`processor` 命名
- 链式处理模式

**CQRS**迹象：
- 命令和查询分离的代码组织
- `Command`/`Query`/`Handler` 命名模式

### 搜索策略
```
codebase_search: "核心数据流程" / "主要处理管道" / "核心工作流"
grep_search: "Arc<" / "RwLock" / "Mutex" / "Channel" / "async fn"  （并发模型）
grep_search: "impl.*Trait" / "interface.*implements"  （核心抽象实现）
```

### 读取策略
- 读取所有核心模块的入口文件
- 读取明显承担「编排」职责的文件（名含 orchestrator/workflow/pipeline）

### 输出格式：架构研究报告

```markdown
## 架构研究报告

### 架构模式
采用的主要架构模式：{模式名称}
理由：{从代码中观察到的证据}

### 核心设计原则
1. {原则1}：{说明和代码证据}
2. {原则2}：{说明和代码证据}

### 技术栈详情
| 层次/领域 | 技术选型 | 选择理由（如可推断） |
|---------|---------|----------------|
| 语言/运行时 | {xxx} | {理由} |
| 异步模型 | {xxx} | {理由} |
| 序列化 | {xxx} | {理由} |

### 关键数据结构
| 类型名 | 文件路径 | 用途 |
|-------|---------|------|
| {Type1} | `path/to/file.rs` | {描述} |

### 核心接口/Trait/协议
| 名称 | 实现数量 | 核心职责 |
|-----|---------|---------|
| {Interface1} | {N} 个实现 | {描述} |

### 架构决策记录（推断）
- **决策1**：{描述}，观察依据：{代码证据}
- **决策2**：{描述}，观察依据：{代码证据}
```

---

## Step 2.4：工作流研究（WorkflowResearcher）

### 目标
识别系统的核心业务执行流程，理解从输入到输出的完整路径。

### 搜索策略
```
# 找主流程函数（入口点）
grep_search: "pub fn launch" / "fn main" / "async fn run" / "func main" / "def main"

# 找流程协调（谁驱动执行顺序？）
codebase_search: "工作流编排" / "流程控制" / "阶段执行"
grep_search: "workflow" / "orchestrat" / "pipeline" / "sequence"

# 找错误处理流程
grep_search: "Result<" / "try {" / "catch {" / ".unwrap_or" / "? "
```

### 读取策略
- 重点读取 `workflow.rs`/`main.rs`/`app.rs`/`server.rs` 等总入口文件
- 追踪主流程函数的调用链（需要跨文件追踪 2-3 层）
- 读取负责流程调度的 orchestrator/scheduler 文件

### 输出格式：工作流研究报告

```markdown
## 工作流研究报告

### 主要工作流

#### 工作流1：{名称}
**触发方式**：{用户行为/事件/定时}
**入口**：{函数名} in `{文件路径}`
**执行步骤**：
1. {步骤1}（调用：`{函数}`）
2. {步骤2}（调用：`{函数}`）
3. ...
**输出/结果**：{描述}

**流程图数据**：
- 节点：[{节点1}, {节点2}, ...]
- 边：[{节点1}→{节点2}, ...]

### 并发/异步模型
{描述系统如何处理并发：线程池/async/await/Actor 模型等}

### 错误处理策略
{描述系统的错误处理和优雅降级策略}

### 关键时序交互
{描述跨模块的关键调用序列，用于生成时序图}
参与者：[{系统1}, {系统2}, ...]
关键交互：
- {系统1} → {系统2}：{操作描述}
- {系统2} → {系统3}：{操作描述}
```

---

## Step 2.5：关键模块深度洞察（KeyModulesInsight）

### 目标
对每个核心领域模块进行深度分析，为 `4.Deep-Exploration/` 文档提供素材。

### 对每个模块按 importance 评分分级分析（⚠️ 渐进式深度控制）

| importance | 分析深度 | 读取文件数 | Mermaid 图 | 最低产出 |
|-----------|---------|----------|-----------|---------|
| ≥7（核心域） | 深度分析 | 5+ | 完整 flowchart + 交互表格 | 概述+功能点+组件表+数据流图+接口+交互 |
| 4-6（支撑域） | 标准分析 | 3 | 精简流程图 | 概述+功能点+组件表+交互表格 |
| ≤3（通用域） | 简要描述 | 1-2 | 无图 | 概述+核心职责+文件列表 |

**核心域模块示例**：workflow、research、compose、preprocess、llm（直接贡献文档生成能力）
**支撑域模块示例**：cache、integrations、memory（为核心域提供技术支撑）
**通用域模块示例**：types、utils、i18n、config（通用基础设施）

**执行节奏**（逐模块独立分析并写入）：
```
核心域模块: 深读 → 深度分析 → write_to_file → 释放上下文
支撑域模块: 中度分析 → write_to_file → 释放上下文
通用域模块: 简要描述 → write_to_file → 释放上下文
每批3-4个模块，按 importance 从高到低排序
```

**并发搜索标注**：Step 2.3(架构)、2.4(工作流)、2.6(边界) 的搜索操作可以并发调用工具；Step 2.5 必须在 Step 2.2 之后执行。

**读取策略**（按 importance 分级）：
1. 模块入口文件（`mod.rs` / `index.ts` / `__init__.py`）
2. 名称含关键词的文件：`core`、`main`、`service`、`handler`、`processor`
3. 最大的几个文件（通常包含核心逻辑）

**分析问题清单**：
- [ ] 这个模块的核心职责是什么（用一句话概括）？
- [ ] 模块暴露的主要公共接口有哪些？
- [ ] 模块内部的主要数据流程是什么？
- [ ] 有哪些值得关注的算法或设计模式？
- [ ] 这个模块有哪些扩展点（可插拔设计）？
- [ ] 模块的性能特点（同步/异步、缓存策略等）？

### 输出格式：模块深度报告（每模块一份）

```markdown
## {模块名称}模块深度报告

### 模块概述
{职责描述，2-3 句话}

### 核心功能点
1. {功能1}：{简述}
2. {功能2}：{简述}

### 关键组件

| 组件/类型 | 文件路径 | 职责 |
|---------|---------|------|
| {Component1} | `path/to/file` | {描述} |

### 内部数据流
{文字描述模块内的数据处理流程}

关键步骤：
1. {步骤1}（函数：`{func_name}` in `{file}`）
2. {步骤2}

### 扩展点
{如有可插拔设计，描述扩展机制}

### 性能考量
{并发策略、缓存使用、资源管理等}
```

---

## Step 2.6：边界接口分析（BoundaryAnalyzer）

### 目标
全面枚举系统对外暴露的接口，为 `5.边界接口.md` 提供素材。

### 接口类型和搜索策略

**CLI 接口**：
```
grep_search: "clap" / "argparse" / "cobra.Command" / "click" / "urfave/cli"
grep_search: "#\[derive(Parser\)" / "Arg::new(" / "subcommand"
read_file: 包含 CLI 定义的文件（通常是 cli.rs/cmd/root.go/cli.py）
```

**REST API**：
```
grep_search: "Router" / "route" / "handler" / ".get(" / ".post(" / "@GetMapping"
codebase_search: "API 路由定义" / "HTTP 端点"
```

**配置结构**：
```
grep_search: "#\[derive(Deserialize\)" / "interface.*Config" / "dataclass" / "@ConfigurationProperties"
read_file: 配置结构定义文件（config.rs/config.ts/settings.py 等）
```

**公共库 API（如果是库项目）**：
```
grep_search: "pub fn" / "export function" / "public static"（收集所有公共函数）
read_file: lib.rs / index.ts / __init__.py（查看导出清单）
```

### 输出格式：边界接口报告

```markdown
## 边界接口报告

### CLI 接口（如适用）

#### 命令：{命令名}
- **描述**：{功能描述}
- **参数**：
  | 参数 | 类型 | 是否必须 | 说明 | 默认值 |
  |------|------|---------|------|-------|
  | {param1} | {type} | 是/否 | {描述} | {default} |
- **使用示例**：
  ```bash
  {命令} {示例参数}
  ```

### REST API 端点（如适用）
| 方法 | 路径 | 描述 | 请求体 | 响应 |
|------|------|------|-------|------|
| GET | `/api/xxx` | {描述} | - | {格式} |

### 配置结构
| 配置项 | 类型 | 描述 | 默认值 |
|-------|------|------|-------|
| {key} | {type} | {描述} | {default} |

### 集成建议
{如何集成和使用这个系统的建议}
```

---

## Step 2.7：数据库分析（DatabaseOverviewAnalyzer）[条件执行]

### 触发条件
检测到以下任一情况时执行：
- 存在 `.sql` 文件
- 存在 `migrations/` 目录
- `Cargo.toml`/`package.json` 中有 ORM 依赖（diesel/sqlx/prisma/typeorm/sequelize/hibernate 等）
- 存在 `.sqlproj` 文件

### 搜索策略
```
list_files: "migrations/" / "sql/" / "db/" / "database/"
grep_search: "CREATE TABLE" / "ALTER TABLE" / "CREATE INDEX"
grep_search: "struct.*Schema" / "entity:" / "@Entity" / "@Table"（ORM 模型定义）
```

### 读取策略
- 读取所有 `.sql` 迁移文件（按时间顺序，了解 schema 演进）
- 读取 ORM 实体定义文件

### 输出格式：数据库概览报告

```markdown
## 数据库概览报告

### 统计摘要
| 类型 | 数量 |
|------|------|
| 数据表 | X |
| 视图 | X |
| 存储过程 | X |
| 索引 | X |

### 核心数据表
#### {表名}
| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| {col1} | {type} | PK/FK/NOT NULL | {描述} |

### 表关系
{文字描述主要表间关系，用于生成 ER 图}
关系列表：
- {表A} 1:N {表B}（通过 {外键字段}）
- {表C} N:N {表D}（通过 {关联表}）
```
