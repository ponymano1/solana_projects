# 第03节账户图解：第一个计数器程序

这份文档只关注一个问题：客户端发起一次操作时，链上到底看到了哪些账户、哪些数据，以及程序如何判断这次操作能不能执行。

第03节的程序很简单：每次操作只传入 1 个业务数据账户，也就是 `Counter Account`。但这不代表全链上只能有 1 个计数器账户；客户端可以创建很多个 `Counter Account`，每个账户都是一个独立计数器。

---

## 1. 本章有哪些账户

```
┌──────────────────────────────┐
│ Program Account              │
│ 也就是部署后的程序本身         │
│                              │
│ key: program_id              │
│ executable: true             │
│ 作用: 执行计数器程序代码       │
└──────────────────────────────┘

┌──────────────────────────────┐
│ Counter Account              │
│ 也就是保存计数器数据的账户     │
│                              │
│ key: counter_account_pubkey  │
│ owner: program_id            │
│ data: Counter 序列化后的字节   │
│ 作用: 保存 count 和 initialized│
└──────────────────────────────┘

┌──────────────────────────────┐
│ Payer / User Wallet          │
│ 用户钱包                      │
│                              │
│ key: payer_pubkey            │
│ signer: true                 │
│ 作用: 支付交易费；创建账户时支付租金 │
└──────────────────────────────┘

┌──────────────────────────────┐
│ System Program               │
│ Solana 内置系统程序           │
│                              │
│ key: 111111111111...         │
│ 作用: 创建普通账户、分配空间、指定 owner │
└──────────────────────────────┘
```

最容易混淆的是：`Counter Account` 不是程序本身，它只是一个被程序管理的数据容器。

```
程序代码在哪里？
  Program Account

计数器数据在哪里？
  某一个 Counter Account.data

谁能改 Counter Account.data？
  只有 Counter Account.owner 指向的程序，也就是 program_id
```

### 1.1 到底有几个 Counter Account

本章测试和示例里，一次只创建并操作 1 个 `Counter Account`：

```
program_id = Counter Program

counter_account_A
┌──────────────────────────────┐
│ owner: program_id            │
│ data: Counter { count: 0 }   │
└──────────────────────────────┘
```

但在真实链上，同一个程序可以管理很多个 `Counter Account`。每创建一个新的账户，就多一个独立计数器：

```
同一个 Counter Program
┌─────────────────────────────────────────────────────┐
│ program_id                                           │
│ 只保存程序代码，不保存某一个固定的 count              │
└─────────────────────────────────────────────────────┘
        │
        ├── 管理 counter_account_A
        │   ┌──────────────────────────────┐
        │   │ owner: program_id            │
        │   │ data: Counter { count: 3 }   │
        │   └──────────────────────────────┘
        │
        ├── 管理 counter_account_B
        │   ┌──────────────────────────────┐
        │   │ owner: program_id            │
        │   │ data: Counter { count: 100 } │
        │   └──────────────────────────────┘
        │
        └── 管理 counter_account_C
            ┌──────────────────────────────┐
            │ owner: program_id            │
            │ data: Counter { count: 0 }   │
            └──────────────────────────────┘
```

所以要区分两个问题：

| 问题 | 答案 |
|---|---|
| 本章一条 `Increment` 指令传几个 `Counter Account`？ | 1 个，放在 `accounts[0]` |
| 整个程序最多能有几个 `Counter Account`？ | 可以有很多个，每个账户地址对应一个独立计数器 |
| 程序怎么知道要改哪一个计数器？ | 客户端把目标 `counter_account` 放进本次指令的 `accounts[0]` |

举例：

```
想给 A 加 1：
  accounts = [counter_account_A]
  data = [1]

想给 B 加 1：
  accounts = [counter_account_B]
  data = [1]

链上代码完全一样，区别只在客户端这次传进来的是哪个账户。
```

---

## 2. Counter Account 里面存了什么

`Counter` 结构：

```rust
pub struct Counter {
    pub count: u64,
    pub is_initialized: bool,
}
```

序列化后占 9 字节：

```
Counter Account
┌──────────────────────────────────────────┐
│ lamports: 租金余额                        │
│ owner: program_id                        │  ← 账户层面的 owner
│ executable: false                        │
│ data:                                    │
│   ┌────┬────┬────┬────┬────┬────┬────┬────┬────┐
│   │ count u64, 小端序，占 8 字节           │ init│
│   └────┴────┴────┴────┴────┴────┴────┴────┴────┘
└──────────────────────────────────────────┘
```

示例：`count = 5`，`is_initialized = true`

```
字节偏移:   0    1    2    3    4    5    6    7    8
          ┌────┬────┬────┬────┬────┬────┬────┬────┬────┐
data:     │ 05 │ 00 │ 00 │ 00 │ 00 │ 00 │ 00 │ 00 │ 01 │
          └────┴────┴────┴────┴────┴────┴────┴────┴────┘
           └────────────── count = 5 ──────────────┘  true
```

---

## 3. 客户端发起操作时传了什么

一次 Solana 指令主要包含三部分：

```
Instruction
┌────────────────────────────────────┐
│ program_id                         │  要调用哪个链上程序
│ accounts                           │  这次调用允许程序访问哪些账户
│ data                               │  这次调用的业务参数
└────────────────────────────────────┘
```

在本章中：

```
program_id = 计数器程序 ID
accounts   = [Counter Account]
data       = 指令编号，比如 [0] / [1] / [2] / [3]
```

指令数据对应关系：

| 客户端 `data` | 链上指令 | 含义 |
|---|---|---|
| `[0]` | `CounterInstruction::Initialize` | 初始化计数器 |
| `[1]` | `CounterInstruction::Increment` | 加 1 |
| `[2]` | `CounterInstruction::Decrement` | 减 1 |
| `[3]` | `CounterInstruction::Reset` | 重置为 0 |

账户数组对应关系：

| 客户端 `keys[i]` | 链上读取方式 | 权限 | 作用 |
|---|---|---|---|
| `keys[0] = counterAccount` | 第一次 `next_account_info` | writable | 读取并修改计数器数据 |

链上代码只会按顺序取账户：

```rust
let accounts_iter = &mut accounts.iter();
let counter_account = next_account_info(accounts_iter)?;
```

所以客户端传账户时，顺序非常重要。第一个账户必须是计数器账户。

---

## 4. 初始化：创建账户 + 写入初始数据

初始化通常分两步：

1. 调用 `System Program` 创建 `Counter Account`
2. 调用自己的计数器程序，把 `Counter { count: 0, is_initialized: true }` 写进去

### 4.1 客户端交易结构

```
Transaction
┌────────────────────────────────────────────┐
│ signer: payer                              │
│ signer: counter_account                    │  创建新账户时需要新账户签名
│                                            │
│ Instruction 1: SystemProgram.createAccount │
│ Instruction 2: CounterProgram.Initialize   │
└────────────────────────────────────────────┘
```

### 4.2 第一步：System Program 创建 Counter Account

客户端传给 System Program 的核心数据：

```
SystemProgram.createAccount
┌──────────────────────────────────────────┐
│ fromPubkey: payer                        │  谁付款
│ newAccountPubkey: counter_account        │  要创建哪个账户
│ lamports: rent_exempt_lamports           │  存入多少租金
│ space: 9                                 │  分配多少 data 空间
│ programId: counter_program_id            │  创建后账户归哪个程序管理
└──────────────────────────────────────────┘
```

账户变化：

```
创建前
payer:           lamports = 10 SOL
counter_account: 不存在

        │ System Program 执行 createAccount
        ▼

创建后
payer:           lamports = 10 SOL - rent - fee
counter_account:
  lamports: rent
  owner: counter_program_id
  data: [00 00 00 00 00 00 00 00 00]
```

这里的关键点是：`owner: counter_program_id`。这表示从现在开始，只有计数器程序能修改这个账户的 `data`。

### 4.3 第二步：调用 Initialize 指令

客户端发给计数器程序的指令：

```
CounterProgram.Initialize
┌──────────────────────────────────────────┐
│ program_id: counter_program_id           │
│ accounts:                                │
│   [0] counter_account                    │ writable, signer=false
│ data: [0]                                │ Initialize
└──────────────────────────────────────────┘
```

链上处理流程：

```
process_instruction
        │
        │ data = [0]
        ▼
反序列化为 CounterInstruction::Initialize
        │
        ▼
process_initialize
        │
        ├─ 读取 accounts[0] 作为 counter_account
        │
        ├─ 鉴权 1: counter_account.owner == program_id ?
        │       是：说明这个数据账户归当前程序管理，可以写 data
        │       否：返回 IncorrectProgramId
        │
        ├─ 读取 counter_account.data
        │
        ├─ 鉴权 2: counter.is_initialized == false ?
        │       是：允许初始化
        │       否：返回 AccountAlreadyInitialized
        │
        └─ 写入 data:
            count = 0
            is_initialized = true
```

初始化后的账户：

```
Counter Account
┌──────────────────────────────────────────┐
│ owner: counter_program_id                │
│ data:                                    │
│   count = 0                              │
│   is_initialized = true                  │
└──────────────────────────────────────────┘
```

---

## 5. Increment：计数器加 1

客户端发给计数器程序的指令：

```
CounterProgram.Increment
┌──────────────────────────────────────────┐
│ program_id: counter_program_id           │
│ accounts:                                │
│   [0] counter_account                    │ writable, signer=false
│ data: [1]                                │ Increment
└──────────────────────────────────────────┘
```

链上操作账户：

```
只操作 accounts[0]: Counter Account

读取:
  counter_account.data -> Counter { count, is_initialized }

修改:
  count = count + 1

写回:
  Counter -> counter_account.data
```

完整流程：

```
客户端
  │
  │ program_id = counter_program_id
  │ accounts = [counter_account writable]
  │ data = [1]
  ▼
链上程序
  │
  ├─ 鉴权 1: counter_account.owner == program_id
  │
  ├─ 鉴权 2: counter.is_initialized == true
  │
  ├─ 安全检查: checked_add，防止 u64 溢出
  │
  └─ 写回 counter_account.data
```

这里没有检查用户签名，所以任何人都可以对这个计数器加 1。第03节的重点是理解数据账户，不是权限控制。

---

## 6. Decrement：计数器减 1

客户端指令：

```
CounterProgram.Decrement
┌──────────────────────────────────────────┐
│ program_id: counter_program_id           │
│ accounts:                                │
│   [0] counter_account                    │ writable, signer=false
│ data: [2]                                │ Decrement
└──────────────────────────────────────────┘
```

链上流程：

```
Counter Account.data
  │
  ▼
反序列化 Counter
  │
  ├─ 检查 Account.owner 是不是当前 program_id
  ├─ 检查 is_initialized 是不是 true
  ├─ checked_sub(1)，防止 0 - 1 下溢
  ▼
序列化 Counter 并写回 data
```

账户变化示例：

```
执行前:
  count = 5
  is_initialized = true

执行后:
  count = 4
  is_initialized = true
```

---

## 7. Reset：重置为 0

客户端指令：

```
CounterProgram.Reset
┌──────────────────────────────────────────┐
│ program_id: counter_program_id           │
│ accounts:                                │
│   [0] counter_account                    │ writable, signer=false
│ data: [3]                                │ Reset
└──────────────────────────────────────────┘
```

链上流程：

```
读取 Counter Account
  │
  ├─ 检查 counter_account.owner == program_id
  ├─ 检查 counter.is_initialized == true
  └─ 设置 counter.count = 0
        │
        ▼
写回 Counter Account.data
```

注意：Reset 不会把 `is_initialized` 改回 `false`。这个账户仍然是一个已经初始化过的计数器，只是数值变回 0。

---

## 8. 本章的鉴权到底是什么

第03节只有两类检查：

```
┌──────────────────────────────────────────┐
│ 1. 账户 owner 检查                        │
│                                          │
│ if counter_account.owner != program_id { │
│     return IncorrectProgramId;           │
│ }                                        │
└──────────────────────────────────────────┘
```

含义：

```
这个 Counter Account 是不是归当前程序管理？

是：当前程序可以修改它的 data
否：当前程序不能把别的程序的账户当成自己的账户来改
```

以及：

```
┌──────────────────────────────────────────┐
│ 2. 初始化状态检查                         │
│                                          │
│ Initialize 时要求 is_initialized=false   │
│ Increment/Decrement/Reset 时要求 true    │
└──────────────────────────────────────────┘
```

含义：

```
Initialize:
  防止重复初始化覆盖已有状态

Increment / Decrement / Reset:
  防止操作一个还没有被正确初始化的数据账户
```

第03节没有做的鉴权：

```
没有 user signer 检查
没有记录谁创建了计数器
没有限制只有某个用户能加减或重置
```

所以这个计数器是“公开可操作”的：只要知道 `counter_account` 地址，任何交易都可以传入这个账户并调用加减或重置。

---

## 9. `isSigner` 和 `isWritable` 怎么理解

客户端代码里会写：

```typescript
keys: [
  { pubkey: counterAccount.publicKey, isSigner: false, isWritable: true },
]
```

意思是：

```
isSigner: false
  本次调用不要求 counterAccount 的私钥签名。
  程序只是把它当作数据账户读写，不需要证明“我是这个账户地址的私钥持有人”。

isWritable: true
  本次调用会修改 counterAccount.data。
  如果不标记 writable，运行时不允许程序写这个账户。
```

为什么创建账户时 `counterAccount` 又需要签名？

```
创建账户阶段:
  System Program 要创建一个新地址对应的账户。
  新账户 Keypair 需要签名，证明客户端确实控制这个新地址。

后续业务操作阶段:
  Counter Account 已经存在，并且 owner 是 counter_program_id。
  修改 data 靠的是“程序拥有这个账户”，不是 Counter Account 私钥签名。
```

---

## 10. 一张总图

```
客户端 Transaction
┌────────────────────────────────────────────────────────────┐
│ payer 签名，支付交易费                                      │
│                                                            │
│ 调用哪个程序: counter_program_id                            │
│ 传入哪些账户:                                               │
│   [0] counter_account, writable                             │
│ 传入什么数据:                                               │
│   [0] Initialize / [1] Increment / [2] Decrement / [3] Reset │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
链上 Counter Program
┌────────────────────────────────────────────────────────────┐
│ process_instruction(program_id, accounts, instruction_data) │
│                                                            │
│ 1. instruction_data 反序列化成具体指令                       │
│ 2. accounts[0] 当作 Counter Account                         │
│ 3. 检查 Counter Account.owner == program_id                  │
│ 4. 反序列化 Counter Account.data                             │
│ 5. 检查初始化状态                                            │
│ 6. 修改 count                                                │
│ 7. 序列化写回 Counter Account.data                           │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
Counter Account
┌────────────────────────────────────────────────────────────┐
│ owner: counter_program_id                                  │
│ data: Counter { count, is_initialized }                    │
└────────────────────────────────────────────────────────────┘
```

---

## 11. 本章应该形成的账户模型直觉

记住这几句话：

1. Solana 程序本身不直接保存业务状态。
2. 业务状态保存在独立的数据账户里，也就是 `Account.data`。
3. 客户端必须显式告诉程序这次可以访问哪些账户。
4. 链上程序按 `accounts` 数组顺序读取账户。
5. 只有账户的 `owner` 程序可以修改这个账户的 `data`。
6. `isSigner` 是交易签名要求，`owner` 是账户数据写权限，它们不是一回事。
7. 第03节的计数器没有用户级权限控制，所以任何人都能操作它。
