# 第08节：测试与调试

本节通过一个计算器程序演示Solana程序的测试最佳实践。

## 项目结构

```
08-testing/
├── src/
│   ├── lib.rs           # 主程序逻辑
│   ├── error.rs         # 自定义错误类型
│   ├── state.rs         # 状态结构
│   └── instruction.rs   # 指令定义
└── tests/
    └── integration.rs   # 集成测试
```

## 核心概念

### 1. 测试框架

使用 `solana-program-test` 进行集成测试：

```rust
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_add() {
    let program_test = ProgramTest::new(
        "testing",
        program_id,
        processor!(process_instruction),
    );
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    // 测试逻辑...
}
```

### 2. 测试类型

#### 正常路径测试
- `test_add`: 测试基本加法操作
- `test_multiple_operations`: 测试多个操作的链式调用

#### 错误处理测试
- `test_division_by_zero`: 验证除零错误
- `test_overflow`: 验证溢出检测

### 3. 错误处理

使用 `checked_*` 方法防止溢出：

```rust
let result = a.checked_add(b)
    .ok_or(CalculatorError::Overflow)?;
```

### 4. 测试最佳实践

1. **测试覆盖率**：包含正常路径和错误情况
2. **独立性**：每个测试独立运行，不依赖其他测试
3. **可读性**：清晰的测试名称和注释
4. **断言**：验证状态变化和错误条件

## 运行测试

```bash
cargo test
```

## 学习要点

1. 如何使用 `solana-program-test` 编写集成测试
2. 如何测试错误处理逻辑
3. 如何验证程序状态变化
4. 使用 `checked_*` 方法防止算术溢出
