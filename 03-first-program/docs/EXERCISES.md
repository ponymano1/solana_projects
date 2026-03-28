# 练习题

通过以下练习巩固你对Solana程序开发的理解。每个练习都会逐步增加难度。

## 练习1：添加SetValue指令

### 目标
添加一个新的指令，允许直接设置计数器的值。

### 要求

1. 在`CounterInstruction`枚举中添加新的变体：
```rust
SetValue { value: u64 }
```

2. 实现`process_set_value`函数

3. 添加测试用例验证功能

### 提示

- 记得检查账户是否已初始化
- 考虑是否需要权限控制
- 添加日志输出

### 测试用例

```rust
#[tokio::test]
async fn test_set_value() {
    // 初始化计数器
    // 设置值为100
    // 验证值是否正确
}
```

### 预期结果

```bash
cargo test test_set_value
# 测试应该通过
```

---

## 练习2：实现计数器的上限和下限

### 目标
为计数器添加上限和下限，防止值超出范围。

### 要求

1. 修改`Counter`结构体：
```rust
pub struct Counter {
    pub count: u64,
    pub is_initialized: bool,
    pub min_value: u64,
    pub max_value: u64,
}
```

2. 修改`Initialize`指令接受参数：
```rust
Initialize { min_value: u64, max_value: u64 }
```

3. 在`Increment`和`Decrement`中检查边界

4. 添加自定义错误类型：
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CounterError {
    #[error("计数器已达到最大值")]
    MaxValueReached,
    
    #[error("计数器已达到最小值")]
    MinValueReached,
}
```

### 提示

- 更新`Counter::LEN`常量（现在是25字节）
- 在增加前检查是否会超过max_value
- 在减少前检查是否会低于min_value
- 更新所有测试用例

### 测试用例

```rust
#[tokio::test]
async fn test_max_value_limit() {
    // 初始化计数器，max_value=10
    // 增加到10
    // 尝试再增加，应该失败
}

#[tokio::test]
async fn test_min_value_limit() {
    // 初始化计数器，min_value=5
    // 设置为5
    // 尝试减少，应该失败
}
```

### 预期结果

计数器值始终在[min_value, max_value]范围内。

---

## 练习3：添加权限控制

### 目标
只允许特定的账户（管理员）修改计数器。

### 要求

1. 修改`Counter`结构体添加管理员字段：
```rust
pub struct Counter {
    pub count: u64,
    pub is_initialized: bool,
    pub authority: Pubkey,  // 管理员公钥
}
```

2. 修改`Initialize`指令接受管理员参数

3. 在所有修改操作中验证签名者：
```rust
fn process_increment(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;  // 新增
    
    // 检查签名
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 检查权限
    let counter = Counter::try_from_slice(&counter_account.data.borrow())?;
    if counter.authority != *authority_account.key {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // ... 继续处理
}
```

4. 添加转移管理员权限的指令：
```rust
TransferAuthority { new_authority: Pubkey }
```

### 提示

- 更新指令创建函数，添加authority账户
- 所有测试需要提供authority签名
- 考虑添加多签支持

### 测试用例

```rust
#[tokio::test]
async fn test_unauthorized_access() {
    // 使用authority1初始化
    // 尝试用authority2增加计数
    // 应该失败
}

#[tokio::test]
async fn test_transfer_authority() {
    // 初始化计数器
    // 转移权限给新管理员
    // 验证新管理员可以操作
    // 验证旧管理员不能操作
}
```

### 预期结果

只有授权的管理员可以修改计数器。

---

## 练习4：实现多个计数器

### 目标
支持为不同用户创建独立的计数器。

### 要求

1. 使用PDA（Program Derived Address）为每个用户创建计数器：
```rust
// 在客户端代码中
let (counter_pda, bump) = Pubkey::find_program_address(
    &[b"counter", user.key().as_ref()],
    &program_id,
);
```

2. 修改初始化逻辑，使用PDA创建账户

3. 添加查询指令，获取用户的计数器值

4. 实现计数器列表功能

### 提示

- 学习PDA的概念（这是第06节的内容）
- 使用`invoke_signed`创建PDA账户
- 考虑如何存储多个计数器的索引

### 测试用例

```rust
#[tokio::test]
async fn test_multiple_counters() {
    // 为user1创建计数器
    // 为user2创建计数器
    // 分别操作两个计数器
    // 验证它们互不影响
}
```

### 预期结果

每个用户都有自己独立的计数器。

---

## 进阶挑战

### 挑战1：添加历史记录

实现一个功能，记录计数器的所有变更历史。

要求：
- 使用额外的账户存储历史记录
- 每次修改时添加时间戳和操作类型
- 实现分页查询历史记录

### 挑战2：实现计数器快照

允许用户创建计数器的快照，并可以恢复到任意快照。

要求：
- 添加CreateSnapshot指令
- 添加RestoreSnapshot指令
- 限制快照数量（如最多10个）

### 挑战3：添加事件通知

使用Solana的日志功能实现事件系统。

要求：
- 定义事件结构
- 在关键操作时发出事件
- 编写客户端代码监听事件

示例：
```rust
#[derive(BorshSerialize)]
pub struct CounterIncrementedEvent {
    pub counter: Pubkey,
    pub old_value: u64,
    pub new_value: u64,
    pub timestamp: i64,
}

// 发出事件
msg!("EVENT:{}", base64::encode(event.try_to_vec()?));
```

---

## 学习资源

完成这些练习后，你应该：

1. 熟练使用Borsh序列化
2. 理解指令设计模式
3. 掌握账户数据管理
4. 了解权限控制机制
5. 初步了解PDA概念

### 推荐阅读

- [Solana Cookbook](https://solanacookbook.com/)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Solana Program Library源码](https://github.com/solana-labs/solana-program-library)

### 下一步

完成练习后，继续学习：
- 第04节：账户与数据存储
- 第05节：指令处理
- 第06节：PDA基础

---

## 提交你的解决方案

如果你完成了练习，可以：

1. 创建新的分支
```bash
git checkout -b exercise-03-solutions
```

2. 提交你的代码
```bash
git add .
git commit -m "完成第03节练习题"
```

3. 运行所有测试确保通过
```bash
cargo test
cargo clippy
```

祝你学习愉快！
