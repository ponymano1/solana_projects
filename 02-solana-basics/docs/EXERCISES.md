# 练习题

## 练习1：Lamports转换

实现以下功能：

1. 将2.5 SOL转换为lamports
2. 将1,500,000,000 lamports转换为SOL
3. 计算两个lamports值的和与差

```rust
// 提示：使用 Lamports::from_sol() 和 as_lamports()
```

## 练习2：账户操作

创建一个账户并执行以下操作：

1. 创建一个拥有5000 lamports的账户
2. 向账户添加数据：`[10, 20, 30, 40, 50]`
3. 读取并验证账户数据
4. 检查账户所有者

```rust
// 提示：使用 Account::new() 和 set_data()
```

## 练习3：交易创建

创建一个转账交易：

1. 创建两个账户地址（使用 `Pubkey::new_unique()`）
2. 创建一个从账户A到账户B转账1000 lamports的交易
3. 验证交易的付款人和指令数量

```rust
// 提示：使用 Transaction::new_transfer()
```

## 练习4：错误处理

实现错误处理逻辑：

1. 尝试从100 lamports中减去200 lamports
2. 捕获并处理 `InsufficientFunds` 错误
3. 打印错误信息

```rust
// 提示：使用 Result 和 match 语句
```

## 练习5：综合应用

实现一个简单的转账函数：

```rust
fn transfer(
    from: &mut Account,
    to: &mut Account,
    amount: u64,
) -> Result<(), SolanaError> {
    // 1. 检查from账户余额是否足够
    // 2. 从from账户减去amount
    // 3. 向to账户添加amount
    // 4. 返回结果
}
```

## 挑战题

### 挑战1：批量转账

实现一个函数，从一个账户向多个账户转账：

```rust
fn batch_transfer(
    from: &mut Account,
    recipients: Vec<(&mut Account, u64)>,
) -> Result<(), SolanaError> {
    // 实现批量转账逻辑
}
```

### 挑战2：账户数据序列化

使用borsh序列化库，实现账户数据的序列化和反序列化：

```rust
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
struct UserData {
    name: String,
    age: u8,
    balance: u64,
}

// 实现将UserData存储到账户的函数
```

## 测试你的答案

运行测试以验证你的实现：

```bash
cargo test
```

## 提示

- 仔细阅读错误信息
- 使用 `cargo clippy` 检查代码质量
- 参考 `src/lib.rs` 中的实现
- 查看 `tests/integration.rs` 中的测试用例