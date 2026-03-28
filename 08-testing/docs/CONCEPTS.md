# 测试与调试概念详解

## 1. Solana程序测试框架

### solana-program-test

`solana-program-test` 是Solana官方提供的测试框架，允许在本地环境中模拟完整的Solana运行时。

#### 核心组件

```rust
use solana_program_test::*;

// 创建测试环境
let program_test = ProgramTest::new(
    "program_name",      // 程序名称
    program_id,          // 程序ID
    processor!(entry),   // 程序入口点
);

// 启动测试环境
let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
```

- `banks_client`: 用于与测试环境交互的客户端
- `payer`: 默认的支付账户
- `recent_blockhash`: 最新的区块哈希

### 测试环境特点

1. **隔离性**：每个测试在独立的环境中运行
2. **快速**：无需连接真实网络
3. **可控**：可以精确控制账户状态和时间
4. **完整性**：模拟真实的Solana运行时行为

## 2. 测试类型

### 2.1 单元测试

测试单个函数或模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_instruction() {
        let data = vec![0, 10, 0, 0, 0, 20, 0, 0, 0];
        let instruction = CalculatorInstruction::try_from_slice(&data).unwrap();
        assert!(matches!(instruction, CalculatorInstruction::Add { a: 10, b: 20 }));
    }
}
```

### 2.2 集成测试

测试完整的程序交互：

```rust
#[tokio::test]
async fn test_program_flow() {
    // 1. 设置测试环境
    let program_test = ProgramTest::new(...);
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 2. 创建账户
    let result_account = Keypair::new();

    // 3. 构建交易
    let mut transaction = Transaction::new_with_payer(...);

    // 4. 执行交易
    banks_client.process_transaction(transaction).await.unwrap();

    // 5. 验证结果
    let account = banks_client.get_account(result_account.pubkey()).await.unwrap();
    // 断言...
}
```

## 3. 错误处理测试

### 3.1 预期错误测试

验证程序正确返回错误：

```rust
#[tokio::test]
async fn test_division_by_zero() {
    // ... 设置 ...

    // 构建会导致除零的指令
    let instruction_data = CalculatorInstruction::Divide { a: 10, b: 0 }
        .try_to_vec()
        .unwrap();

    // 执行交易
    let result = banks_client.process_transaction(transaction).await;

    // 验证返回错误
    assert!(result.is_err());
}
```

### 3.2 自定义错误类型

使用 `thiserror` 定义清晰的错误：

```rust
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum CalculatorError {
    #[error("除数不能为零")]
    DivisionByZero,

    #[error("算术运算溢出")]
    Overflow,
}
```

## 4. 状态验证

### 4.1 账户数据验证

```rust
// 获取账户
let account = banks_client
    .get_account(result_account.pubkey())
    .await
    .unwrap()
    .unwrap();

// 反序列化数据
let result_data = CalculatorResult::try_from_slice(&account.data).unwrap();

// 验证状态
assert_eq!(result_data.result, 30);
assert_eq!(result_data.operation_count, 1);
```

### 4.2 多步骤验证

```rust
#[tokio::test]
async fn test_multiple_operations() {
    // 操作1: 10 + 20 = 30
    // ... 执行 ...
    let data = CalculatorResult::try_from_slice(&account.data).unwrap();
    assert_eq!(data.result, 30);
    assert_eq!(data.operation_count, 1);

    // 操作2: 30 - 5 = 25
    // ... 执行 ...
    let data = CalculatorResult::try_from_slice(&account.data).unwrap();
    assert_eq!(data.result, 25);
    assert_eq!(data.operation_count, 2);
}
```

## 5. 安全性测试

### 5.1 溢出检测

使用 `checked_*` 方法：

```rust
// 不安全：可能溢出
let result = a + b;

// 安全：检测溢出
let result = a.checked_add(b)
    .ok_or(CalculatorError::Overflow)?;
```

### 5.2 边界条件测试

```rust
#[tokio::test]
async fn test_overflow() {
    // 测试 u64::MAX + 1
    let instruction_data = CalculatorInstruction::Add {
        a: u64::MAX,
        b: 1,
    }
    .try_to_vec()
    .unwrap();

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}
```

## 6. 测试最佳实践

### 6.1 测试命名

使用描述性名称：
- `test_add` - 测试加法功能
- `test_division_by_zero` - 测试除零错误
- `test_multiple_operations` - 测试多步操作

### 6.2 测试结构

遵循 AAA 模式：
1. **Arrange**（准备）：设置测试环境和数据
2. **Act**（执行）：执行被测试的操作
3. **Assert**（断言）：验证结果

```rust
#[tokio::test]
async fn test_example() {
    // Arrange
    let program_test = ProgramTest::new(...);
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Act
    banks_client.process_transaction(transaction).await.unwrap();

    // Assert
    let account = banks_client.get_account(...).await.unwrap();
    assert_eq!(result, expected);
}
```

### 6.3 测试覆盖率

确保测试覆盖：
- ✅ 正常执行路径
- ✅ 所有错误情况
- ✅ 边界条件
- ✅ 状态转换

### 6.4 测试独立性

每个测试应该：
- 独立运行
- 不依赖其他测试
- 不影响其他测试
- 可以任意顺序执行

## 7. 调试技巧

### 7.1 程序日志

使用 `msg!` 输出调试信息：

```rust
use solana_program::msg;

msg!("指令: 加法 {} + {}", a, b);
msg!("结果: {}", result);
```

### 7.2 测试日志

运行测试时查看日志：

```bash
# 显示所有日志
RUST_LOG=debug cargo test -- --nocapture

# 只显示程序日志
cargo test -- --nocapture
```

### 7.3 错误追踪

使用 `?` 操作符传播错误：

```rust
let result = operation()
    .map_err(|e| {
        msg!("操作失败: {:?}", e);
        e
    })?;
```

## 8. 性能测试

### 8.1 计算单元测试

监控程序的计算单元消耗：

```rust
#[tokio::test]
async fn test_compute_units() {
    // ... 执行交易 ...

    // 检查计算单元消耗
    let meta = banks_client
        .get_transaction_status(signature)
        .await
        .unwrap();

    println!("计算单元: {:?}", meta);
}
```

### 8.2 账户大小优化

验证账户大小：

```rust
assert!(account.data.len() <= MAX_ACCOUNT_SIZE);
```

## 总结

良好的测试实践包括：

1. **全面的测试覆盖**：正常路径 + 错误情况 + 边界条件
2. **清晰的测试结构**：AAA模式，描述性命名
3. **安全性优先**：使用checked操作，测试溢出
4. **独立性**：每个测试独立运行
5. **可维护性**：清晰的代码和注释
6. **调试友好**：适当的日志输出

通过系统的测试，可以确保Solana程序的正确性、安全性和可靠性。
