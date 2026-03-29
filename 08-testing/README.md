# 第08节 - 测试与调试：计算器程序

本节学习Solana程序的测试方法和调试技巧，实现一个计算器程序作为测试示例。

## 学习目标

- ✅ 掌握solana-program-test框架
- ✅ 学会编写集成测试
- ✅ 理解测试环境的搭建
- ✅ 掌握错误测试和边界测试
- ✅ 学会使用日志调试

## 快速开始

```bash
cd 08-testing
cargo build-bpf
cargo test
cargo test -- --nocapture  # 查看日志
```

---

## 核心概念

### 1. solana-program-test框架

提供本地测试环境，无需部署到测试网。

**优势：**
- 快速执行
- 完全控制
- 易于调试
- 支持并行测试

### 2. 测试结构

```rust
#[tokio::test]
async fn test_add() {
    // 1. 创建测试环境
    let program_test = ProgramTest::new(...);
    
    // 2. 启动测试环境
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // 3. 创建账户
    // 4. 构建指令
    // 5. 发送交易
    // 6. 验证结果
}
```

### 3. 计算器功能

```
Add:      a + b
Subtract: a - b
Multiply: a * b
Divide:   a / b (检查除零)
```

---

## 测试示例

### 基本测试

```rust
#[tokio::test]
async fn test_add() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "calculator",
        program_id,
        processor!(process_instruction),
    );

    // 添加结果账户
    let result_account = Keypair::new();
    program_test.add_account(
        result_account.pubkey(),
        Account {
            lamports: 1_000_000,
            data: vec![0; 16],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 构建Add指令
    let instruction = create_add_instruction(&program_id, &result_account.pubkey(), 5, 3);
    
    // 发送交易
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证结果
    let account = banks_client.get_account(result_account.pubkey()).await.unwrap().unwrap();
    let result: CalculatorResult = borsh::BorshDeserialize::try_from_slice(&account.data).unwrap();
    assert_eq!(result.result, 8);
}
```

### 错误测试

```rust
#[tokio::test]
async fn test_divide_by_zero() {
    // ... 设置测试环境 ...

    let instruction = create_divide_instruction(&program_id, &result_account.pubkey(), 10, 0);
    
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    
    // 应该失败
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}
```

---

## 客户端调用

### 计算器指令

```typescript
// 序列化Add指令
function serializeAdd(a: bigint, b: bigint): Buffer {
  const buffer = Buffer.alloc(17);
  buffer[0] = 0; // Add指令索引
  buffer.writeBigInt64LE(a, 1);
  buffer.writeBigInt64LE(b, 9);
  return buffer;
}

const addIx = new TransactionInstruction({
  keys: [
    { pubkey: resultAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: serializeAdd(BigInt(5), BigInt(3)),
});
```

### 指令对应

| 客户端 | 链上 | 数据格式 |
|--------|------|----------|
| `serializeAdd(a, b)` | `CalculatorInstruction::Add{a, b}` | `[0, a(8字节), b(8字节)]` |
| `serializeSubtract(a, b)` | `CalculatorInstruction::Subtract{a, b}` | `[1, a(8字节), b(8字节)]` |
| `serializeMultiply(a, b)` | `CalculatorInstruction::Multiply{a, b}` | `[2, a(8字节), b(8字节)]` |
| `serializeDivide(a, b)` | `CalculatorInstruction::Divide{a, b}` | `[3, a(8字节), b(8字节)]` |

---

## 调试技巧

### 1. 使用msg!打印日志

```rust
msg!("计算: {} + {} = {}", a, b, result);
```

### 2. 查看测试日志

```bash
cargo test -- --nocapture
```

### 3. 单独运行测试

```bash
cargo test test_add
```

### 4. 检查账户状态

```rust
let account = banks_client.get_account(account_pubkey).await.unwrap();
msg!("账户余额: {}", account.lamports);
msg!("账户数据长度: {}", account.data.len());
```

---

## 常见问题

### Q: 如何测试错误情况？
A: 使用`assert!(result.is_err())`验证交易失败。

### Q: 如何查看程序日志？
A: 使用`cargo test -- --nocapture`查看`msg!`输出。

### Q: 测试环境和真实环境有什么区别？
A: 测试环境是本地模拟，速度快但可能与真实环境有细微差异。

### Q: 如何测试并发？
A: solana-program-test支持并行测试，每个测试独立运行。

---

## 最佳实践

1. **测试覆盖**：正常情况 + 边界情况 + 错误情况
2. **独立测试**：每个测试独立，不依赖其他测试
3. **清晰断言**：使用有意义的断言消息
4. **日志调试**：合理使用`msg!`打印关键信息

---

## 下一步

恭喜完成阶段一（原生开发基础）！

继续学习：
- 阶段二：Anchor框架
- 阶段三：实战应用
- 阶段四：进阶主题

## 参考资料

- [solana-program-test文档](https://docs.rs/solana-program-test/)
- [Solana测试指南](https://docs.solana.com/developing/test-validator)
