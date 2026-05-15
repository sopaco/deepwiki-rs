# Mermaid 图表语法速查与类型选择指南

> 完整文档模板内嵌在 `phase3-composition.md` 中各 Step 的模板章节。

---

## 图表类型选择指南

| 场景 | 推荐图类型 | 语法关键词 |
|------|-----------|-----------|
| 系统边界+外部用户/系统 | `C4Context` | Person, System, System_Ext, Rel |
| 系统内部主要子系统 | `C4Container` | Container, Container_Boundary, ContainerDb |
| 模块内部组件 | `C4Component` | Component, Rel |
| 带条件分支的处理流程 | `flowchart TD` | -->, --\|label\|--> |
| 从左到右的数据流 | `flowchart LR` | --> |
| 多系统/组件时序交互 | `sequenceDiagram` | participant, ->>, -->> |
| 模块依赖关系 | `graph LR` | --> |
| 数据库表间关系 | `erDiagram` | }\|--\|\{, TABLE \{ |
| 系统状态机 | `stateDiagram-v2` | state, --> |

---

## 语法核心规范

### 节点 ID：仅字母数字+下划线
```
✅ A, B1, myNode, LLClient
❌ My Node, node-1, @user, node.1
```

### 节点标签：含特殊字符必加双引号
```
✅ A["LLM Client<br/>统一接口"]
❌ A[LLM Client<br/>统一接口]
```

### 标签换行：用 `<br/>`
```
✅ A["第一行<br/>第二行"]
❌ A["第一行\n第二行"]
```

### flowchart 箭头语法
```
✅ A --> B           # 无标签
✅ A --|"描述"|--> B  # 有标签
❌ A -> B             # flowchart 用 --> 不是 ->
```

### sequenceDiagram 消息语法
```
✅ A->>B: 同步消息
✅ A-->>B: 返回消息
✅ loop 描述 ... end
✅ alt 条件 ... else ... end
```

### erDiagram 关系符号
```
||--||   一对一
||--o{   一对多（零或多）
||--|{   一对多（一或多）
}|--|{   多对多
```

### C4 图结构
```
C4Context
    title 标题
    Person(id, "名", "描述")
    System(id, "名", "描述")
    System_Ext(id, "名", "描述")
    Rel(from, to, "关系", "技术")

C4Container
    title 标题
    System_Boundary(id, "名") { Container(...) }
    ContainerDb(id, "名", "技术", "描述")
```

---

## 快速修复常见错误

| 错误 | 修正 |
|------|------|
| 节点ID含空格或连字符 | 改用驼峰命名：`my-node` → `myNode` |
| 标签含括号/冒号未加引号 | 包裹双引号：`A[系统(核心)]` → `A["系统(核心)"]` |
| 子图名含空格 | 加引号别名：`subgraph 输入层` → `subgraph inputLayer["输入层"]` |
| C4图节点数过多 | 合并同类容器，保持 < 15 个节点 |
| erDiagram字段类型含特殊字符 | 用引号包裹类型名