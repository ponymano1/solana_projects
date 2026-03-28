# 第05节 - 指令处理

## 学习目标

- 设计复杂的指令结构
- 实现指令路由和分发
- 处理指令参数和验证
- 掌握自定义错误类型的使用
- 理解权限控制和访问验证

## 前置知识

在开始本节之前，你需要完成：
- 第01节：环境搭建
- 第02节：Solana基础概念
- 第03节：第一个原生程序
- 第04节：账户与数据存储

确保你已经理解了账户模型、Borsh序列化和基本的程序结构。

## 项目结构

```
05-instructions/
├── Cargo.toml          # 项目配置
├── README.md           # 本文件
├── src/
│   ├── lib.rs          # 主程序逻辑
│   ├── instruction.rs  # 指令枚举定义
│   ├── state.rs        # 状态结构定义
│   └── error.rs        # 自定义错误类型
├── tests/
│   └── integration.rs  # 集成测试
└── docs/
    ├── CONCEPTS.md     # 核心概念详解
    └── EXERCISES.md    # 练习题
```

## 运行步骤

### 1. 编译程序

```bash
cd 05-instructions
cargo build-bpf
```

### 2. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_initialize_todo_list
cargo test test_create_todo

# 查看测试日志
cargo test -- --nocapture
```

### 3. 代码检查

```bash
# 格式化代码
cargo fmt

# 运行Clippy检查
cargo clippy
```

## 核心概念

### 1. 复杂指令结构

本节实现了一个Todo应用，支持多种操作：

```rust
pub enum TodoInstruction {
    Initialize,
    CreateTodo { title: String, description: String },
    UpdateTodo { id: u32, completed: bool },
    DeleteTodo { id: u32 },
}
```

### 2. 指令路由

使用match表达式将指令分发到对应的处理函数：

```rust
match instruction {
    TodoInstruction::Initialize => process_initialize(program_id, accounts),
    TodoInstruction::CreateTodo { title, description } =>
        process_create_todo(program_id, accounts, title, description),
    // ...
}
```

### 3. 自定义错误类型

使用thiserror库定义清晰的错误信息：

```rust
#[derive(Error, Debug, Copy, Clone)]
pub enum TodoError {
    #[error("无效的指令")]
    InvalidInstruction,
    #[error("账户未初始化")]
    UninitializedAccount,
    // ...
}
```

### 4. 参数验证

在处理指令前验证所有参数：

```rust
if title.len() > MAX_TITLE_LEN {
    return Err(TodoError::TitleTooLong.into());
}
```

### 5. 权限控制

确保只有所有者可以操作Todo列表：

```rust
if todo_list.owner != *owner_account.key {
    return Err(TodoError::Unauthorized.into());
}
```

## 程序功能

本Todo应用支持以下操作：

1. **Initialize** - 初始化Todo列表，设置所有者
2. **CreateTodo** - 创建新的Todo项
3. **UpdateTodo** - 更新Todo的完成状态
4. **DeleteTodo** - 删除指定的Todo项

## 测试用例

项目包含5个完整的测试用例：

1. `test_initialize_todo_list` - 测试初始化功能
2. `test_create_todo` - 测试创建Todo
3. `test_update_todo` - 测试更新Todo状态
4. `test_delete_todo` - 测试删除Todo
5. `test_unauthorized_access` - 测试权限控制

## 常见问题FAQ

### Q1: 为什么需要自定义错误类型？

A: 自定义错误类型提供了：
- 清晰的错误信息，便于调试
- 类型安全，编译时检查
- 更好的用户体验
- 便于错误处理和日志记录

### Q2: 如何设计指令结构？

A: 设计指令时应考虑：
- 每个指令应该有明确的单一职责
- 使用枚举的变体来携带不同的参数
- 参数应该是必需的，避免使用Option
- 考虑指令的组合和顺序依赖

### Q3: 为什么要限制Todo列表的大小？

A: 限制大小是因为：
- 账户空间是有限的，需要预先分配
- 防止无限增长导致的性能问题
- 简化租金计算
- 这是Solana程序的常见模式

### Q4: 如何处理Vec的序列化？

A: Borsh会自动处理Vec的序列化：
- 先序列化Vec的长度（4字节）
- 然后序列化每个元素
- 反序列化时按相同顺序读取

### Q5: 权限检查应该放在哪里？

A: 权限检查应该：
- 在每个需要权限的操作开始时进行
- 在修改数据之前完成
- 使用is_signer检查签名
- 使用owner字段检查所有权

## 下一步

完成本节后，继续学习：
- 第06节：PDA（程序派生地址）
- 第07节：跨程序调用（CPI）

## 参考资源

- [Solana程序开发文档](https://docs.solana.com/developing/on-chain-programs/overview)
- [Borsh序列化](https://borsh.io/)
- [thiserror库文档](https://docs.rs/thiserror/)
