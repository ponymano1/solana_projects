# 第03节 - 第一个Solana程序：计数器

本节实现一个简单的计数器程序，学习Solana程序的基本结构、指令处理和状态管理。

## 目录

- [学习目标](#学习目标)
- [快速开始](#快速开始)
- [核心概念](#核心概念)
- [数据存储详解](#数据存储详解)
- [程序结构](#程序结构)
- [客户端调用](#客户端调用)
- [常见问题](#常见问题)

---

## 学习目标

- ✅ 理解Solana程序的基本结构
- ✅ 掌握Borsh序列化/反序列化
- ✅ 学会指令枚举的定义和使用
- ✅ 理解程序入口点和指令分发
- ✅ 掌握客户端与链上程序的数据对应关系

## 前置知识

- 完成第01节：环境搭建
- 完成第02节：Solana基础概念
- 理解账户模型、程序部署和交易的基本概念

## 快速开始

### 编译和测试

```bash
cd 03-first-program
cargo build-bpf
cargo test
```

### 测试包括

- `test_initialize_counter` - 初始化计数器
- `test_increment_counter` - 增加计数
- `test_decrement_counter` - 减少计数
- `test_reset_counter` - 重置计数
- `test_uninitialized_account_error` - 测试未初始化错误
- `test_ownership_check` - 测试所有权检查

---

## 核心概念

### 1. 数据存储位置

**关键问题：Counter数据存在哪里？**

答案：序列化后存储在账户的 `data` 字段中。

```
┌─────────────────────────────────────┐
│    Counter Account（计数器账户）     │
│                                     │
│  owner: program_id                  │
│  data: [0, 0, 0, 0, 0, 0, 0, 0, 1]  │ ← 9字节
│         └────────┬────────┘  └─┘    │
│              count(u64)    initialized(bool)
└─────────────────────────────────────┘
         ↓ 反序列化
┌─────────────────────────────────────┐
│      Counter（数据结构）             │
│                                     │
│  count: 0                           │
│  is_initialized: true               │
└─────────────────────────────────────┘
```

**存储过程：**
1. 创建Counter结构体
2. 使用Borsh序列化为9字节数组
3. 写入账户的data字段
4. 读取时反序列化回Counter结构

### 2. Borsh序列化

Borsh（Binary Object Representation Serializer for Hashing）是Solana使用的序列化格式。

**优势：**
- **确定性**：相同数据总是产生相同字节序列
- **高效**：紧凑的二进制格式
- **安全**：严格的类型检查

**示例：**
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Counter {
    pub count: u64,           // 8字节
    pub is_initialized: bool, // 1字节
}
// 总计：9字节
```

**数据布局：**
```
字节偏移:  0  1  2  3  4  5  6  7  8
         ┌──┬──┬──┬──┬──┬──┬──┬──┬──┐
         │     count (u64)      │ i │
         └──┴──┴──┴──┴──┴──┴──┴──┴──┘
                                  └─ is_initialized (bool)

示例：count = 5, is_initialized = true
         ┌──┬──┬──┬──┬──┬──┬──┬──┬──┐
         │05│00│00│00│00│00│00│00│01│
         └──┴──┴──┴──┴──┴──┴──┴──┴──┘
         └─────── 小端序 ────────┘
```

### 3. 指令枚举

```rust
pub enum CounterInstruction {
    Initialize,  // 指令索引: 0
    Increment,   // 指令索引: 1
    Decrement,   // 指令索引: 2
    Reset,       // 指令索引: 3
}
```

**Borsh序列化后：**
- `Initialize` → `[0]`
- `Increment` → `[1]`
- `Decrement` → `[2]`
- `Reset` → `[3]`

---

## 数据存储详解

### Counter结构的内存布局

```
字节偏移:  0  1  2  3  4  5  6  7  8
         ┌──┬──┬──┬──┬──┬──┬──┬──┬──┐
字段:     │     count (u64)      │ i │
         └──┴──┴──┴──┴──┴──┴──┴──┴──┘
                                  └─ is_initialized (bool)
```

**小端序（Little-Endian）：**
- 低位字节存储在低地址
- 5 (u64) = 0x0000000000000005
- 存储为：[05, 00, 00, 00, 00, 00, 00, 00]

**为什么需要is_initialized？**
- 防止重复初始化
- 区分"未初始化"和"初始化为0"
- 确保账户在使用前已正确设置

---

## 程序结构

### 入口点（lib.rs）

```rust
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,      // 程序ID
    accounts: &[AccountInfo], // 账户列表
    instruction_data: &[u8],  // 指令数据
) -> ProgramResult {
    // 1. 反序列化指令
    let instruction = CounterInstruction::try_from_slice(instruction_data)?;

    // 2. 根据指令类型分发处理
    match instruction {
        CounterInstruction::Initialize => process_initialize(...),
        CounterInstruction::Increment => process_increment(...),
        CounterInstruction::Decrement => process_decrement(...),
        CounterInstruction::Reset => process_reset(...),
    }
}
```

### 处理流程图

```
客户端发送交易
    ↓
[指令数据: [1]]  ← Increment指令
    ↓
process_instruction接收
    ↓
反序列化为CounterInstruction::Increment
    ↓
match分发到process_increment
    ↓
读取账户data → 反序列化Counter
    ↓
count += 1
    ↓
序列化Counter → 写入账户data
    ↓
返回成功
```

### 初始化流程

```
┌─────────────┐
│ 客户端       │
└──────┬──────┘
       │ 1. 创建账户（System Program）
       │    - 分配9字节空间
       │    - 设置owner为program_id
       ↓
┌──────────────────────────────┐
│ Counter Account              │
│ data: [0, 0, 0, 0, 0, 0, 0, 0, 0] │ ← 未初始化
└──────┬───────────────────────┘
       │ 2. 发送Initialize指令
       │    instruction_data: [0]
       ↓
┌──────────────────────────────┐
│ process_initialize           │
│ - 检查未初始化               │
│ - 设置count = 0              │
│ - 设置is_initialized = true  │
└──────┬───────────────────────┘
       │ 3. 序列化并写入
       ↓
┌──────────────────────────────┐
│ Counter Account              │
│ data: [0, 0, 0, 0, 0, 0, 0, 0, 1] │ ← 已初始化
└──────────────────────────────┘
```

---

## 客户端调用

### 完整流程示例

#### 步骤1：创建账户

```typescript
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

const PROGRAM_ID = new PublicKey('你的程序ID');
const connection = new Connection('http://localhost:8899', 'confirmed');
const payer = Keypair.fromSecretKey(/* 你的密钥 */);

// 1. 生成计数器账户
const counterAccount = Keypair.generate();

// 2. 计算租金
const space = 9; // Counter的大小
const rentExemptLamports = await connection.getMinimumBalanceForRentExemption(space);

// 3. 创建账户
const createAccountIx = SystemProgram.createAccount({
  fromPubkey: payer.publicKey,
  newAccountPubkey: counterAccount.publicKey,
  lamports: rentExemptLamports,
  space: space,
  programId: PROGRAM_ID,
});
```

#### 步骤2：初始化计数器

```typescript
// 构建Initialize指令
const initializeIx = new TransactionInstruction({
  keys: [
    { pubkey: counterAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: Buffer.from([0]), // Initialize指令索引
});

// 发送交易
const transaction = new Transaction().add(createAccountIx, initializeIx);
await sendAndConfirmTransaction(
  connection,
  transaction,
  [payer, counterAccount] // 签名者：payer和counterAccount
);

console.log('计数器已初始化');
```

#### 步骤3：增加计数

```typescript
const incrementIx = new TransactionInstruction({
  keys: [
    { pubkey: counterAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: Buffer.from([1]), // Increment指令索引
});

const tx = new Transaction().add(incrementIx);
await sendAndConfirmTransaction(connection, tx, [payer]);

console.log('计数已增加');
```

#### 步骤4：读取计数器值

```typescript
// 1. 获取账户信息
const accountInfo = await connection.getAccountInfo(counterAccount.publicKey);

// 2. 手动反序列化（Borsh格式）
const data = accountInfo.data;
const count = data.readBigUInt64LE(0);        // 前8字节：count
const isInitialized = data[8] !== 0;          // 第9字节：is_initialized

console.log('Count:', count.toString());
console.log('Initialized:', isInitialized);
```

### 客户端与链上对应关系

#### 指令数据对应

| 客户端 | 链上 | 说明 |
|--------|------|------|
| `Buffer.from([0])` | `CounterInstruction::Initialize` | 初始化 |
| `Buffer.from([1])` | `CounterInstruction::Increment` | 增加 |
| `Buffer.from([2])` | `CounterInstruction::Decrement` | 减少 |
| `Buffer.from([3])` | `CounterInstruction::Reset` | 重置 |

#### 账户数组对应

| 客户端 keys[i] | 链上 accounts[i] | 说明 |
|----------------|------------------|------|
| `keys[0]` | `counter_account` | 计数器账户，需要写入 |

**注意：**
- 这个程序只需要一个账户
- `isSigner: false` 因为不需要计数器账户签名
- `isWritable: true` 因为需要修改账户data

### 完整客户端代码

```typescript
async function main() {
  const connection = new Connection('http://localhost:8899', 'confirmed');
  const payer = Keypair.fromSecretKey(/* 你的密钥 */);
  const counterAccount = Keypair.generate();

  // 1. 创建账户
  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: counterAccount.publicKey,
    lamports: await connection.getMinimumBalanceForRentExemption(9),
    space: 9,
    programId: PROGRAM_ID,
  });

  // 2. 初始化
  const initializeIx = new TransactionInstruction({
    keys: [{ pubkey: counterAccount.publicKey, isSigner: false, isWritable: true }],
    programId: PROGRAM_ID,
    data: Buffer.from([0]),
  });

  const createTx = new Transaction().add(createAccountIx, initializeIx);
  await sendAndConfirmTransaction(connection, createTx, [payer, counterAccount]);
  console.log('✓ 计数器已初始化');

  // 3. 增加计数
  const incrementIx = new TransactionInstruction({
    keys: [{ pubkey: counterAccount.publicKey, isSigner: false, isWritable: true }],
    programId: PROGRAM_ID,
    data: Buffer.from([1]),
  });

  await sendAndConfirmTransaction(connection, new Transaction().add(incrementIx), [payer]);
  console.log('✓ 计数已增加');

  // 4. 读取计数
  const accountInfo = await connection.getAccountInfo(counterAccount.publicKey);
  const count = accountInfo.data.readBigUInt64LE(0);
  console.log('当前计数:', count.toString());
}

main();
```

---

## 常见问题

### Q: 为什么需要is_initialized字段？
A:
- 防止重复初始化
- 区分"未初始化"和"初始化为0"
- 确保账户在使用前已正确设置

### Q: 为什么使用checked_add而不是直接+？
A:
- 防止整数溢出
- Solana程序必须处理所有可能的错误
- checked_add在溢出时返回None，可以安全处理

### Q: 客户端如何知道指令索引？
A:
- 查看链上程序的instruction.rs
- 枚举的顺序决定了索引：第一个是0，第二个是1...
- 或者查看程序文档

### Q: 为什么计数器账户不需要签名？
A:
- 计数器账户只是数据存储
- 不需要证明拥有私钥
- 只有payer需要签名（支付交易费）

### Q: 数据如何在链上存储？
A:
- Counter结构序列化为9字节
- 存储在账户的data字段
- 读取时反序列化回Counter结构

### Q: 如何防止未授权修改计数器？
A:
- 这个简单示例没有权限控制
- 实际应用中应该添加owner字段
- 验证签名者是否是owner（参考第04节）

---

## 最佳实践

### 1. 状态检查

```rust
// ✅ 正确：检查初始化状态
if !counter.is_initialized {
    return Err(ProgramError::UninitializedAccount);
}

// ❌ 错误：不检查状态
let mut counter = Counter::try_from_slice(&account.data)?;
counter.count += 1; // 可能操作未初始化的账户
```

### 2. 溢出保护

```rust
// ✅ 正确：使用checked_add
counter.count = counter.count
    .checked_add(1)
    .ok_or(ProgramError::InvalidAccountData)?;

// ❌ 错误：直接加法
counter.count += 1; // 可能溢出
```

### 3. 防止重复初始化

```rust
// ✅ 正确：检查是否已初始化
if counter.is_initialized {
    return Err(ProgramError::AccountAlreadyInitialized);
}

// ❌ 错误：不检查
counter.is_initialized = true; // 可能覆盖已有数据
```

---

## 项目结构

```
03-first-program/
├── src/
│   ├── lib.rs          # 程序入口点，指令分发
│   ├── instruction.rs  # 指令枚举定义
│   └── state.rs        # 状态数据结构
└── tests/
    └── integration.rs  # 集成测试
```

---

## 下一步

完成本节后，继续学习：
- **第04节：账户与数据存储** - 学习账户创建和权限控制
- **第05节：指令处理** - 学习复杂指令和错误处理
- **第06节：PDA基础** - 学习程序派生地址

---

## 参考资料

- [Solana程序入口点](https://docs.solana.com/developing/on-chain-programs/overview#entry-point)
- [Borsh序列化](https://borsh.io/)
- [Solana程序测试](https://docs.solana.com/developing/test-validator)
