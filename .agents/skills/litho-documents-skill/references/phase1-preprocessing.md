# 阶段一：预处理详细执行指南

## 目标
从零开始理解项目，建立完整的项目基础洞察，为研究阶段提供高质量的上下文。

---

## Step 1.1：项目根目录扫描

使用 `list_files(path="{project_path}", recursive=false)` 扫描根目录。

**识别项目类型的关键文件**：

| 文件 | 项目类型 |
|------|---------|
| `Cargo.toml` | Rust 项目 |
| `package.json` | Node.js/JavaScript/TypeScript |
| `pom.xml` / `build.gradle` | Java/Kotlin |
| `go.mod` | Go |
| `requirements.txt` / `pyproject.toml` | Python |
| `*.sln` / `*.csproj` | C#/.NET |
| `pubspec.yaml` | Flutter/Dart |
| `CMakeLists.txt` | C/C++ |

**读取策略**：
1. 先读取识别到的项目配置文件（了解依赖和元数据）
2. 读取 `README.md`（如存在多个，优先读目标语言版本）
3. 识别主源码目录（`src/`、`lib/`、`app/`、`cmd/`、`core/` 等）

---

## Step 1.2：源码目录结构扫描

使用 `list_files(path="{src_dir}", recursive=true)` 扫描源码目录。

**分析维度**：

### 目录组织模式识别
- **按功能分层**（如 `controllers/`、`services/`、`models/`）→ 分层架构
- **按领域划分**（如 `payment/`、`user/`、`order/`）→ 模块化/DDD 架构
- **按类型划分**（如 `handlers/`、`middleware/`、`utils/`）→ MVC 架构
- **混合模式**→ 记录具体结构

### 文件数量统计
```
总文件数：X
按语言分布：
  - Rust (.rs)：X 个
  - TypeScript (.ts/.tsx)：X 个
  - ... 其他语言
核心源码文件数：X（排除 test/spec 文件后）
```

### 跳过规则（以下目录不递归分析）
```
.git/  node_modules/  target/  build/  dist/  .cache/
__pycache__/  vendor/  .svelte-kit/  out/  bin/  obj/
*.test.*  *.spec.*  __tests__/  test/  tests/
```

---

## Step 1.3：核心文件深度读取

### 优先级 1（必读）
- `README.md` — 项目功能描述
- 主配置文件（Cargo.toml/package.json 等）— 依赖和版本
- 主入口文件（main.rs/index.ts/main.py/Main.java/main.go 等）

### 优先级 2（重要）
- 模块声明文件（`src/lib.rs`、`src/mod.rs`、各模块的 `mod.rs`）
- 核心抽象定义文件（往往在 `types/`、`models/`、`interfaces/` 目录）
- 配置结构定义文件

### 优先级 3（按需）
- 核心业务逻辑文件（名称包含：`workflow`、`orchestrator`、`service`、`handler`、`processor`）
- 工具类文件（往往体现系统的横切关注点）

### 读取技巧
- 使用 `view_file_outline` 先看文件结构，再决定是否全量读取
- 对超过 500 行的文件，先读头部 100 行和大纲，选择性读关键段落
- 使用 `read_file(start_line_one_indexed, end_line_one_indexed)` 精准读取

---

## Step 1.4：代码分析搜索策略

### 识别核心抽象（接口/Trait/协议）
```
grep_search: "pub trait" / "interface " / "abstract class" / "Protocol"
目的：识别系统的核心抽象层
```

### 识别主要数据类型
```
grep_search: "pub struct" / "data class" / "type.*{" / "class.*:"
目的：识别核心数据模型
```

### 识别关键函数/方法
```
codebase_search: "主要处理流程" / "核心业务逻辑" / "入口函数"
grep_search: "pub fn launch" / "async fn main" / "func Run" / "def run"
目的：找到系统的执行入口和核心流程函数
```

### 识别模块依赖
```
grep_search: "use crate::" / "import {" / "from . import" / "require("
目的：理解模块间的依赖关系
```

### 识别配置和初始化
```
codebase_search: "配置初始化" / "context creation" / "dependency injection"
目的：理解系统的初始化流程和依赖管理方式
```

---

## Step 1.5：建立预处理报告

将上述分析整理为结构化报告，作为全程的基础上下文：

```markdown
# 预处理报告

## 项目基本信息
- **项目名称**：{从配置文件/README提取}
- **版本**：{从配置文件提取}
- **项目类型**：CLI工具 / Web服务 / 库 / 桌面应用 / ...
- **主要编程语言**：{语言1（主要）, 语言2（次要）}
- **核心框架/运行时**：{框架列表}

## 技术栈
- **运行时**：{Rust/Node.js/JVM/Python Runtime/...}
- **Web框架**：{如适用}
- **数据库**：{如适用}
- **LLM/AI集成**：{如适用}
- **主要依赖库**：{top 5-10 个关键依赖}

## 目录结构摘要
```
{项目名}/
├── src/           # {描述}
│   ├── module1/   # {描述}
│   └── module2/   # {描述}
├── docs/          # {描述}
└── ...
```

## 识别到的核心模块
- **{模块1名称}** (`src/module1/`)：{一句话职责描述}
- **{模块2名称}** (`src/module2/`)：{一句话职责描述}
- ...

## 关键文件清单
- 入口文件：`{path}`
- 核心抽象：`{path}` （包含 {trait/interface 列表}）
- 数据类型：`{path}` （包含 {主要类型列表}）

## 依赖关系摘要
{识别到的主要模块依赖关系，用文字或简单列表描述}

## README 核心内容
{提取 README 中的关键信息：项目功能、使用方式、架构说明等}

## 注意事项
{任何特别的代码组织方式、非常规架构选择、需要特别关注的内容}
```

---

## 常见挑战和处理策略

### 无 README 的项目
从入口文件注释、配置文件元数据、测试文件描述中推断项目功能。

### 大型 Monorepo
先扫描顶层的 `packages/`、`apps/`、`services/` 结构，识别各子项目，选择性分析核心子项目。

### 多语言混合项目
按文件数量分布确定主语言，然后识别语言间的集成方式（FFI/gRPC/HTTP API 等）。

### 代码注释稀少的项目
依赖函数/类命名、目录结构、测试用例来推断功能意图。
