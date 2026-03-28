# 第03节 - 第一个原生程序

## 学习目标

- 创建可以接收和处理数据的Solana程序
- 掌握Borsh序列化/反序列化的使用
- 实现简单的计数器程序（Initialize/Increment/Decrement/Reset）
- 理解账户数据的读写流程
- 学习程序所有权检查的重要性

## 前置知识

在开始本节之前，你需要完成：
- 第01节：环境搭建
- 第02节：Solana基础概念

确保你已经理解了账户模型、程序部署和交易的基本概念。

## 项目结构

```
03-first-program/
├── Cargo.toml          # 项目配置
├── README.md           # 本文件
├── src/
│   ├── lib.rs          # 主程序逻辑
│   ├── instruction.rs  # 指令枚举定义
│   └── state.rs        # 状态结构定义
├── tests/
│   └── integration.rs  # 集成测试
└── docs/
    ├── CONCEPTS.md     # 核心概念详解
    └── EXERCISES.md    # 练习题
```

## 运行步骤

### 1. 编译程序

```bash
cd 03-first-program
cargo build-bpf
```

### 2. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_initialize_counter
cargo test test_increment_counter

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

### 1. Borsh序列化

Borsh（Binary Object Representation Serializer for Hashing）是一种高效的二进制序列化格式，专为区块链设计。

```rust
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Counter {
    pub count: u64,
    pub is_initialized: bool,
}
```

### 2. 指令枚举

使用枚举定义程序支持的所有操作：

```rust
pub enum CounterInstruction {
    Initialize,
    Increment,
    Decrement,
    Reset,
}
```

### 3. 账户数据读写

```rust
// 读取账户数据
let counter = Counter::try_from_slice(&account.data.borrow())?;

// 修改数据
counter.count += 1;

// 写回账户
counter.serialize(&mut &mut account.data.borrow_mut()[..])?;
```

### 4. 所有权检查

确保账户属于当前程序：

```rust
if account.owner != program_id {
    return Err(ProgramError::IncorrectProgramId);
}
```

## 程序功能

本计数器程序支持以下操作：

1. **Initialize** - 初始化计数器，设置初始值为0
2. **Increment** - 增加计数值
3. **Decrement** - 减少计数值
4. **Reset** - 重置计数值为0

## 测试用例

项目包含6个完整的测试用例：

1. `test_initialize_counter` - 测试初始化功能
2. `test_increment_counter` - 测试增加计数
3. `test_decrement_counter` - 测试减少计数
4. `test_reset_counter` - 测试重置功能
5. `test_uninitialized_account_error` - 测试未初始化错误
6. `test_ownership_check` - 测试所有权检查

## 常见问题FAQ

### Q1: 为什么使用Borsh而不是其他序列化格式？

A: Borsh专为区块链设计，具有以下优势：
- 确定性：相同数据总是产生相同的字节序列
- 高效：比JSON等格式更节省空间
- 安全：严格的类型检查，防止反序列化攻击

### Q2: 为什么需要is_initialized字段？

A: 防止重复初始化和确保账户在使用前已正确设置。这是一种常见的安全模式。

### Q3: checked_add和checked_sub的作用是什么？

A: 防止整数溢出和下溢。在区块链程序中，必须处理所有可能的边界情况。

### Q4: 如何扩展这个程序？

A: 参考docs/EXERCISES.md中的练习题，可以添加：
- 新的指令（如SetValue）
- 计数器的上限和下限
- 权限控制
- 多个计数器支持

## 下一步

完成本节后，继续学习：
- 第04节：账户与数据存储
- 第05节：指令处理

## 参考资源

- [Solana程序开发文档](https://docs.solana.com/developing/on-chain-programs/overview)
- [Borsh规范](https://borsh.io/)
- [Solana Program Library](https://github.com/solana-labs/solana-program-library)
