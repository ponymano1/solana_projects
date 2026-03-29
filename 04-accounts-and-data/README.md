# 第04节 - 账户与数据存储

本节深入学习Solana的账户模型、数据存储机制和账户生命周期管理。

## 目录

- [学习目标](#学习目标)
- [快速开始](#快速开始)
- [核心概念](#核心概念)
  - [1. 数据到底存在哪里？](#1-数据到底存在哪里)
  - [2. 两层Owner概念](#2-两层owner概念重要)
  - [3. 账户创建流程](#3-账户创建流程)
  - [4. 租金机制](#4-租金机制)
- [客户端调用](#客户端调用)
- [常见问题](#常见问题)
- [最佳实践](#最佳实践)

---

## 学习目标

- ✅ 理解数据在链上的实际存储位置
- ✅ 掌握两层owner概念（账户owner vs 数据owner）
- ✅ 学会创建账户和计算租金
- ✅ 实现完整的数据CRUD操作
- ✅ 理解客户端如何与链上程序交互

## 快速开始

### 编译和测试

```bash
cd 04-accounts-and-data
cargo build-bpf
cargo test
```

### 测试包括

- `test_create_profile` - 创建用户配置文件
- `test_update_profile` - 更新配置文件
- `test_close_profile` - 关闭配置文件并返还租金
- `test_rent_calculation` - 租金计算
- `test_ownership_check` - 所有权验证
- `test_data_validation` - 数据验证

---

## 核心概念

### 1. 数据到底存在哪里？

**答案：数据序列化后存储在账户的 `data` 字段中。**

#### Solana账户结构

每个账户包含以下字段：

```
Account {
    lamports: u64,        // 账户余额
    owner: Pubkey,        // 拥有此账户的程序ID
    data: Vec<u8>,        // 数据存储区域 ← 你的数据存在这里！
    executable: bool,     // 是否可执行
    rent_epoch: u64,      // 租金纪元
}
```

#### 数据存储可视化

```
┌─────────────────────────────────────┐
│         Account（账户）              │
│                                     │
│  lamports: 1000000                  │
│  owner: program_id                  │
│  data: [1, 0, 0, 0, 32, ...]  ← 这里！│
│  executable: false                  │
│  rent_epoch: 123                    │
└─────────────────────────────────────┘
         ↓ 反序列化
┌─────────────────────────────────────┐
│      UserProfile（数据结构）         │
│                                     │
│  is_initialized: true               │
│  owner: user_pubkey                 │
│  name: "张三"                        │
│  age: 25                            │
│  email: "zhangsan@example.com"      │
└─────────────────────────────────────┘
```

#### 存储过程

1. 创建UserProfile结构体
2. 使用Borsh序列化为字节数组
3. 写入账户的data字段：`profile.serialize(&mut account.data)?`
4. 读取时反序列化回结构体

---

### 2. 两层Owner概念（重要！）

这是Solana最容易混淆的概念。有**两个不同的"owner"**：

#### 账户层面的Owner (Account.owner)

```rust
// 账户结构中的owner字段
pub struct Account {
    pub owner: Pubkey,  // ← 这是账户层面的owner
    pub data: Vec<u8>,
}
```

**作用：**
- 表示哪个程序拥有这个账户的控制权
- 只有owner程序才能修改账户的data字段
- 只有owner程序才能减少账户的lamports

**验证时机：创建账户时**

```rust
// 在create_profile中验证
if profile_info.owner != program_id {
    return Err(ProgramError::IncorrectProgramId);
}
```

#### 数据层面的Owner (UserProfile.owner)

```rust
// UserProfile结构中的owner字段
pub struct UserProfile {
    pub owner: Pubkey,  // ← 这是数据层面的owner
    pub name: [u8; 32],
}
```

**作用：**
- 表示这个配置文件属于哪个用户
- 业务逻辑层面的权限控制
- 决定谁可以更新/删除这个配置文件

**验证时机：更新/删除操作时**

```rust
// 在update_profile中验证
if profile.owner != *owner_info.key {
    return Err(ProgramError::IllegalOwner);
}
```

#### 两层Owner关系图

```
┌────────────────────────────────────────────────────┐
│                  Solana账户                         │
│                                                     │
│  ┌──────────────────────────────────────────┐     │
│  │ Account.owner = program_id               │     │ ← 账户层面
│  │ (表示：这个账户被program_id程序管理)       │     │
│  └──────────────────────────────────────────┘     │
│                                                     │
│  ┌──────────────────────────────────────────┐     │
│  │ Account.data = [序列化的UserProfile]      │     │
│  │                                           │     │
│  │  反序列化后：                              │     │
│  │  ┌─────────────────────────────────┐     │     │
│  │  │ UserProfile.owner = user_pubkey │     │     │ ← 数据层面
│  │  │ (表示：这个配置文件属于user)      │     │     │
│  │  │ name: "张三"                     │     │     │
│  │  │ age: 25                         │     │     │
│  │  └─────────────────────────────────┘     │     │
│  └──────────────────────────────────────────┘     │
└────────────────────────────────────────────────────┘

权限模型：
- program_id 控制账户 → 可以读写Account.data
- user_pubkey 控制数据 → 可以更新UserProfile内容
```

#### 为什么需要两层Owner？

| 场景 | 验证的Owner | 原因 |
|------|------------|------|
| **创建配置文件** | Account.owner | 确保账户已分配给程序，程序才能写入data |
| **更新配置文件** | UserProfile.owner | 业务逻辑：只有配置文件创建者才能修改 |

**对比表：**

| 层面 | Owner值 | 作用 | 验证时机 | 验证代码 |
|------|---------|------|----------|----------|
| 账户层面 | `program_id` | 控制账户访问 | 创建时 | `profile_info.owner != program_id` |
| 数据层面 | `user_pubkey` | 控制数据权限 | 更新/删除时 | `profile.owner != *owner_info.key` |

---

### 3. 账户创建流程

#### 完整流程图

```
┌──────────────┐
│   用户钱包    │ (payer)
└──────┬───────┘
       │ 1. 支付租金
       │ 2. 签名交易
       ↓
┌──────────────────────────────────────────┐
│         System Program                    │
│  (Solana内置程序，负责创建账户)            │
└──────┬───────────────────────────────────┘
       │ 创建账户，设置owner=program_id
       ↓
┌──────────────────────────────────────────┐
│      Profile Account (配置文件账户)        │
│  owner: program_id                       │
│  data: [空]                              │
└──────┬───────────────────────────────────┘
       │
       ↓
┌──────────────────────────────────────────┐
│      Your Program (你的程序)              │
│  初始化data，写入UserProfile数据          │
└──────────────────────────────────────────┘
```

#### 代码实现

```rust
// 1. 计算所需空间
let space = UserProfile::space(); // 132字节

// 2. 计算租金
let rent = Rent::default();
let lamports = rent.minimum_balance(space);

// 3. 创建账户（通过System Program）
let create_account_ix = system_instruction::create_account(
    &payer.pubkey(),        // 付款人
    &new_account.pubkey(),  // 新账户
    lamports,               // 租金
    space as u64,           // 空间
    &program_id,            // 设置owner为你的程序
);

// 4. 初始化数据（你的程序）
let profile = UserProfile::new(...);
profile.serialize(&mut account.data)?;
```

#### 为什么需要两步？

1. **创建账户**：System Program负责分配空间和转移所有权
2. **初始化数据**：你的程序负责写入业务数据

这种分离确保了清晰的职责划分。

---

### 4. 租金机制

#### 租金的目的

防止区块链状态无限增长：
- 存储数据需要成本
- 验证节点需要存储所有账户
- 租金激励用户清理不需要的账户

#### 租金豁免

账户可以通过保持足够余额来豁免租金：

```rust
let rent = Rent::default();
let min_balance = rent.minimum_balance(account_size);

// 检查是否豁免
if rent.is_exempt(account.lamports, account_size) {
    // 账户豁免租金
}
```

#### 租金计算

```
最小余额 = 每字节每年租金 × 账户大小 × 2年
```

示例：
- 132字节账户
- 最小余额：约 0.00091872 SOL

#### 租金回收

关闭账户时返还租金：

```rust
// 转移lamports到接收者
**receiver.lamports.borrow_mut() += account.lamports();

// 清空账户
**account.lamports.borrow_mut() = 0;
account.data.borrow_mut().fill(0);
```

---

## 客户端调用

### 创建配置文件

客户端需要发送两个指令：

```typescript
// 1. 创建账户（调用System Program）
const createAccountIx = SystemProgram.createAccount({
  fromPubkey: payer.publicKey,
  newAccountPubkey: profileAccount.publicKey,
  lamports: rentExemptLamports,     // 租金
  space: 132,                       // UserProfile的大小
  programId: yourProgramId,         // 设置owner为你的程序
});

// 2. 初始化配置文件（调用你的程序）
const createProfileIx = new TransactionInstruction({
  keys: [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: profileAccount.publicKey, isSigner: true, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: yourProgramId,
  data: serializeCreateProfileData("张三", 25, "zhangsan@example.com"),
});

// 3. 发送交易
const transaction = new Transaction().add(createAccountIx, createProfileIx);
await sendAndConfirmTransaction(connection, transaction, [payer, profileAccount]);
```

### 数据对应关系

**客户端传递的accounts对应链上：**

| 客户端 keys[i] | 链上 accounts[i] | 说明 |
|----------------|------------------|------|
| `keys[0]` | `payer_info` | 付款人，需要签名 |
| `keys[1]` | `profile_info` | 配置文件账户，需要签名 |
| `keys[2]` | `system_program_info` | System Program，只读 |

### 更新配置文件

```typescript
const updateProfileIx = new TransactionInstruction({
  keys: [
    { pubkey: owner.publicKey, isSigner: true, isWritable: true },  // 数据owner
    { pubkey: profileAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: yourProgramId,
  data: serializeUpdateProfileData("李四", 26, null),
});
```

**注意差异：**
- 创建时：profile账户需要签名（`isSigner: true`）
- 更新时：profile账户不需要签名（`isSigner: false`）
- 原因：更新时验证的是数据owner，不是账户owner

**详细的客户端调用指南请查看：[docs/CLIENT_GUIDE.md](docs/CLIENT_GUIDE.md)**

---

## 常见问题

### Q: 数据到底存在哪里？
A: 数据序列化后存储在账户的`data`字段中。账户是Solana的基本存储单元。

### Q: 为什么有两个owner？
A:
- `Account.owner`（账户owner）：Solana系统层面，控制谁能修改账户
- `UserProfile.owner`（数据owner）：业务逻辑层面，控制谁能操作数据

### Q: 创建时为什么验证Account.owner？
A: 确保账户已经正确分配给你的程序。只有owner程序才能写入账户data。

### Q: 更新时为什么验证UserProfile.owner？
A: 这是业务权限控制。账户owner肯定是program_id（否则读不到数据），需要验证的是谁有权修改这个配置文件。

### Q: 客户端如何知道传哪些账户？
A: 查看程序代码中的`process_xxx`函数，看它调用了几次`next_account_info`，顺序对应客户端的`keys`数组。

### Q: 为什么需要租金？
A: 防止区块链状态无限增长。账户必须保持足够余额才能存活。

### Q: 如何计算租金？
A: 使用 `Rent::default().minimum_balance(space)` 计算租金豁免所需的最小余额。

### Q: 账户关闭后租金去哪了？
A: 返还给指定的接收者账户（通常是数据owner）。

---

## 最佳实践

### 空间设计

- ✅ 使用固定大小数据结构
- ✅ 预留扩展空间（如果需要）
- ❌ 避免过度分配空间

### 安全检查

```rust
// 1. 验证签名者
if !owner.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
}

// 2. 验证账户所有者
if profile.owner != *owner.key {
    return Err(ProgramError::IllegalOwner);
}

// 3. 检查初始化状态
if !profile.is_initialized {
    return Err(ProgramError::UninitializedAccount);
}

// 4. 验证输入数据
if name.len() > MAX_NAME_LEN {
    return Err(ProgramError::InvalidInstructionData);
}
```

### 数据布局

UserProfile使用固定大小布局：

```rust
pub struct UserProfile {
    is_initialized: bool,    // 1字节
    owner: Pubkey,           // 32字节
    name: [u8; 32],         // 32字节（固定数组）
    name_len: u8,           // 1字节
    age: u8,                // 1字节
    email: [u8; 64],        // 64字节（固定数组）
    email_len: u8,          // 1字节
}
// 总计：132字节
```

**为什么使用固定大小数组？**
- 空间可预测，便于计算租金
- 序列化/反序列化高效
- 避免动态分配的复杂性

### 常见陷阱

#### ❌ 忘记验证所有权

```rust
// 错误：没有验证所有者
let mut profile = UserProfile::try_from_slice(&account.data)?;
profile.name = new_name;
```

#### ✅ 正确做法

```rust
// 正确：验证所有者
if profile.owner != *signer.key {
    return Err(ProgramError::IllegalOwner);
}
```

#### ❌ 空间计算错误

```rust
// 错误：使用String的实际长度
let space = 1 + 32 + name.len() + 1 + email.len();
```

#### ✅ 正确做法

```rust
// 正确：使用最大长度
let space = 1 + 32 + MAX_NAME_LEN + 1 + MAX_EMAIL_LEN;
```

---

## 与以太坊的对比

| 特性 | Solana | 以太坊 |
|------|--------|--------|
| 存储模型 | 独立账户，程序和数据分离 | 合约内部存储 |
| 成本模型 | 租金（可回收） | Gas（一次性） |
| 数据访问 | 显式传递账户 | 隐式访问storage |
| 所有权 | 程序拥有账户 | 合约拥有存储 |
| 账户创建 | 需要System Program | 自动创建 |

---

## 项目结构

```
04-accounts-and-data/
├── src/
│   ├── lib.rs           # 入口点
│   ├── instruction.rs   # 指令定义
│   ├── state.rs         # 数据结构（增强注释）
│   └── processor.rs     # 业务逻辑（增强注释）
├── tests/
│   └── integration.rs   # 集成测试
├── docs/
│   └── CLIENT_GUIDE.md  # 客户端调用详细指南
└── README.md            # 本文档
```

---

## 下一步

完成本节后，继续学习：
- **第05节：指令处理** - 复杂指令结构和错误处理
- **第06节：PDA基础** - 程序派生地址
- **第07节：CPI基础** - 跨程序调用

---

## 参考资料

- [Solana账户模型](https://docs.solana.com/developing/programming-model/accounts)
- [租金机制](https://docs.solana.com/developing/programming-model/accounts#rent)
- [System Program](https://docs.solana.com/developing/runtime-facilities/programs#system-program)
- [Borsh序列化](https://borsh.io/)
