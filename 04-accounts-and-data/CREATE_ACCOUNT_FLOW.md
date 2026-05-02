# 第04节：`createAccount` 操作过程图解

这份文档专门解释：创建 `Profile Account` 时，本地客户端做了什么，交易发到链上后 Solana Runtime / System Program 又做了什么。

第04节里的创建配置文件不是一步完成的，而是一个交易里放了两条指令：

```text
Transaction
┌────────────────────────────────────────────┐
│ signer: payer                              │
│ signer: profile_account                    │
│                                            │
│ Instruction 1: SystemProgram.createAccount │
│ Instruction 2: ProfileProgram.CreateProfile│
└────────────────────────────────────────────┘
```

可以理解成：

```text
先创建一个空的数据账户
再把 UserProfile 数据写进去
```

---

## 1. 先明确几个角色

```text
┌──────────────────────────────┐
│ payer                        │
│ 用户钱包                      │
│                              │
│ 有私钥                        │
│ 有 SOL                        │
│ 负责支付交易费和租金           │
└──────────────────────────────┘

┌──────────────────────────────┐
│ profileAccount               │
│ 新生成的数据账户 Keypair       │
│                              │
│ 有公钥 publicKey              │
│ 有私钥 secretKey              │
│ 创建后用来保存 UserProfile     │
└──────────────────────────────┘

┌──────────────────────────────┐
│ System Program               │
│ Solana 内置程序               │
│                              │
│ 负责创建账户                  │
│ 负责分配空间                  │
│ 负责设置 Account.owner        │
└──────────────────────────────┘

┌──────────────────────────────┐
│ Profile Program              │
│ 你写的第04节程序              │
│                              │
│ 负责把 UserProfile 写入 data  │
└──────────────────────────────┘
```

注意：`profileAccount` 是一个 Keypair，所以它有私钥。但它的私钥主要用于“创建这个账户地址”时签名。账户创建后，后续更新配置文件通常不再需要它签名。

---

## 2. 本地客户端做了什么

客户端大致做 6 件事。

### 2.1 生成一个新的 profileAccount

```ts
const profileAccount = Keypair.generate();
```

本地生成：

```text
profileAccount
┌────────────────────────────────────┐
│ publicKey = 新账户地址              │
│ secretKey = 新账户私钥              │
└────────────────────────────────────┘
```

此时链上还没有这个账户。

```text
本地：有 profileAccount Keypair
链上：还没有 profileAccount 这个账户
```

---

### 2.2 计算账户需要多少空间

第04节的 `UserProfile` 固定占 132 字节：

```rust
pub fn space() -> usize {
    1 + 32 + 32 + 1 + 1 + 64 + 1
}
```

也就是：

```text
UserProfile.data
┌────────────────┬────────┬────────────┬──────────┬─────┬─────────────┬───────────┐
│ is_initialized │ owner  │ name       │ name_len │ age │ email       │ email_len │
│ 1 byte         │ 32     │ 32         │ 1        │ 1   │ 64          │ 1         │
└────────────────┴────────┴────────────┴──────────┴─────┴─────────────┴───────────┘
总计 132 bytes
```

客户端也要知道这个大小：

```ts
const PROFILE_SIZE = 132;
```

---

### 2.3 计算租金豁免需要多少 lamports

```ts
const rentExemptLamports = await connection.getMinimumBalanceForRentExemption(
  PROFILE_SIZE
);
```

意思是：

```text
我要创建一个 132 字节的账户。
它至少要存多少 lamports，才能保持 rent-exempt？
```

本地只是向 RPC 查询结果，还没有真正扣钱。

---

### 2.4 构造 SystemProgram.createAccount 指令

```ts
const createAccountIx = SystemProgram.createAccount({
  fromPubkey: payer.publicKey,
  newAccountPubkey: profileAccount.publicKey,
  lamports: rentExemptLamports,
  space: PROFILE_SIZE,
  programId: programId,
});
```

这条指令的含义：

```text
请 System Program 创建一个新账户：

付款人：payer
新账户地址：profileAccount.publicKey
存入金额：rentExemptLamports
分配空间：132 bytes
账户 owner：programId，也就是 Profile Program
```

对应图：

```text
SystemProgram.createAccount
┌──────────────────────────────────────────┐
│ fromPubkey: payer                        │  从谁那里扣 SOL
│ newAccountPubkey: profileAccount         │  创建哪个新账户
│ lamports: rentExemptLamports             │  给新账户存多少 lamports
│ space: 132                               │  给新账户分配多少 data 空间
│ programId: Profile Program ID            │  新账户归哪个程序管理
└──────────────────────────────────────────┘
```

重点是最后一项：

```text
programId: Profile Program ID
```

它会让新建出来的账户变成：

```text
profile_account.owner = Profile Program ID
```

也就是说，账户创建完成后，只有第04节这个程序可以修改 `profile_account.data`。

---

### 2.5 构造 ProfileProgram.CreateProfile 指令

`createAccount` 只负责创建一个空账户，不会自动写入 `UserProfile`。

所以客户端还要再构造一条调用自己程序的指令：

```ts
const createProfileIx = new TransactionInstruction({
  keys: [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: profileAccount.publicKey, isSigner: true, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: programId,
  data: createProfileInstructionData,
});
```

这条指令的含义：

```text
请 Profile Program 初始化刚创建的 profile_account：

accounts[0] = payer
accounts[1] = profile_account
accounts[2] = system_program

data = CreateProfile { name, age, email }
```

注意：这一步写入的是业务数据：

```text
UserProfile {
  is_initialized: true,
  owner: payer.publicKey,
  name,
  age,
  email,
}
```

---

### 2.6 把两条指令放进一个交易并签名发送

```ts
const transaction = new Transaction().add(
  createAccountIx,
  createProfileIx
);

await sendAndConfirmTransaction(
  connection,
  transaction,
  [payer, profileAccount]
);
```

为什么签名者是两个？

```text
payer 签名
  表示：我同意付款。
  包括交易费，以及给 profile_account 存入租金。

profileAccount 签名
  表示：我同意创建这个新地址对应的账户。
  证明客户端确实控制 profileAccount.publicKey 这个地址。
```

所以 create 阶段需要：

```text
signers = [payer, profileAccount]
```

---

## 3. 链上执行时发生了什么

交易发送到链上后，不是你的程序先执行，而是 Solana Runtime 先处理交易。

整体流程：

```text
客户端发送 Transaction
        │
        ▼
Solana Runtime
        │
        ├─ 检查交易签名
        ├─ 加载交易里声明的账户
        ├─ 执行 Instruction 1: SystemProgram.createAccount
        └─ 执行 Instruction 2: ProfileProgram.CreateProfile
```

---

## 4. 链上第一步：System Program 创建账户

执行第一条指令前，链上状态类似：

```text
payer account
┌──────────────────────────────┐
│ lamports: 例如 10 SOL         │
│ owner: System Program         │
└──────────────────────────────┘

profile_account
┌──────────────────────────────┐
│ 不存在                        │
└──────────────────────────────┘
```

System Program 执行 `createAccount` 时做这些事：

```text
1. 检查 payer 是否签名
2. 检查 profile_account 是否签名
3. 从 payer 扣除 lamports
4. 创建 profile_account
5. 给 profile_account 分配 132 字节 data 空间
6. 给 profile_account 存入 rentExemptLamports
7. 设置 profile_account.owner = Profile Program ID
```

执行后：

```text
payer account
┌──────────────────────────────────────────┐
│ lamports: 原余额 - rentExemptLamports - fee│
│ owner: System Program                     │
└──────────────────────────────────────────┘

profile_account
┌──────────────────────────────────────────┐
│ lamports: rentExemptLamports             │
│ owner: Profile Program ID                │
│ data: 132 个 0                           │
│ executable: false                        │
└──────────────────────────────────────────┘
```

此时账户已经存在，但里面还没有有效的 `UserProfile` 数据。

```text
profile_account.data = [0, 0, 0, 0, ...]
```

---

## 5. 链上第二步：Profile Program 初始化数据

第二条指令开始执行：

```text
ProfileProgram.CreateProfile
```

链上程序收到三个参数：

```rust
process_instruction(
    program_id,
    accounts,
    instruction_data,
)
```

它们分别是：

```text
program_id:
  当前 Profile Program 的程序 ID

accounts:
  [
    payer_info,
    profile_info,
    system_program_info,
  ]

instruction_data:
  Borsh 序列化后的 CreateProfile { name, age, email }
```

链上代码按顺序取账户：

```rust
let payer_info = next_account_info(account_info_iter)?;
let profile_info = next_account_info(account_info_iter)?;
let _system_program_info = next_account_info(account_info_iter)?;
```

所以客户端账户顺序必须是：

```text
accounts[0] = payer
accounts[1] = profile_account
accounts[2] = system_program
```

然后程序做检查：

```text
1. payer_info.is_signer == true ?
   确认付款人签名了

2. profile_info.is_signer == true ?
   确认新账户也签名了

3. profile_info.owner == program_id ?
   确认 profile_account 已经被 System Program 设置为当前程序管理

4. name/email 长度是否合法？
   name 非空且 <= 32 字节
   email 非空且 <= 64 字节
```

检查通过后，程序构造业务数据：

```rust
let profile = UserProfile::new(
    *payer_info.key,
    name,
    age,
    email,
)?;
```

也就是：

```text
UserProfile {
  is_initialized: true,
  owner: payer.publicKey,
  name,
  age,
  email,
}
```

最后序列化写入账户：

```rust
profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;
```

写入后：

```text
profile_account
┌──────────────────────────────────────────┐
│ lamports: rentExemptLamports             │
│ owner: Profile Program ID                │  ← Solana 账户层面的 owner
│ data:                                    │
│   UserProfile {                          │
│     is_initialized: true                 │
│     owner: payer.publicKey               │  ← 业务数据层面的 owner
│     name: "张三"                         │
│     age: 25                              │
│     email: "zhangsan@example.com"        │
│   }                                      │
└──────────────────────────────────────────┘
```

---

## 6. 本地和链上的职责分工

| 阶段 | 本地客户端做什么 | 链上做什么 |
|---|---|---|
| 生成账户 | `Keypair.generate()` 生成 `profileAccount` | 什么都没发生，链上还没有这个账户 |
| 算空间 | 确定 `PROFILE_SIZE = 132` | 什么都没发生 |
| 算租金 | RPC 查询 rent-exempt 余额 | 返回所需 lamports |
| 构造 createAccount | 填 `fromPubkey/newAccountPubkey/lamports/space/programId` | 还没执行，只是构造指令 |
| 构造 CreateProfile | 填 accounts 和 Borsh 数据 | 还没执行，只是构造指令 |
| 签名发送 | `payer` 和 `profileAccount` 签名 | Runtime 验证签名 |
| 执行 createAccount | 无 | System Program 扣款、创建账户、分配空间、设置 owner |
| 执行 CreateProfile | 无 | Profile Program 校验账户、写入 UserProfile 数据 |

---

## 7. 为什么 create 要 profileAccount 签名，update 不需要

### Create 时

```text
profile_account 还不存在。
你正在创建这个地址对应的新账户。
```

所以需要：

```text
profileAccount 签名
  证明：我控制这个新账户地址
  表示：我同意创建这个账户
```

### Update 时

```text
profile_account 已经存在。
它只是一个数据账户。
```

更新时真正需要证明的是：

```text
谁有权修改这份 UserProfile 数据？
```

第04节的答案是：

```text
UserProfile.owner 对应的钱包有权修改
```

所以 update 检查：

```rust
if !owner_info.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
}

if profile.owner != *owner_info.key {
    return Err(ProgramError::IllegalOwner);
}
```

也就是：

```text
签名的钱包 == UserProfile.owner
```

而不是：

```text
profileAccount 本身签名
```

总结：

| 操作 | profileAccount 是否签名 | 原因 |
|---|---|---|
| `createAccount` | 需要 | 创建这个新地址对应的账户，需要证明控制该地址 |
| `CreateProfile` | 本章代码也要求需要 | 因为它紧跟创建流程，确认新账户签名参与了初始化 |
| `UpdateProfile` | 不需要 | 它已经是数据账户，业务权限看 `UserProfile.owner` 钱包签名 |
| `CloseProfile` | 不需要 | 业务权限仍然看 `UserProfile.owner` 钱包签名 |

---

## 8. 一张完整流程图

```text
本地客户端
┌────────────────────────────────────────────────────────────┐
│ 1. payer 已有钱包                                           │
│ 2. profileAccount = Keypair.generate()                      │
│ 3. PROFILE_SIZE = 132                                       │
│ 4. 查询 rentExemptLamports                                  │
│ 5. 构造 SystemProgram.createAccount                         │
│ 6. 构造 ProfileProgram.CreateProfile                        │
│ 7. payer + profileAccount 签名并发送交易                     │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
Solana Runtime
┌────────────────────────────────────────────────────────────┐
│ 1. 验证 payer 签名                                          │
│ 2. 验证 profileAccount 签名                                 │
│ 3. 按顺序执行交易里的指令                                    │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
System Program
┌────────────────────────────────────────────────────────────┐
│ createAccount                                               │
│                                                            │
│ payer 扣 lamports                                           │
│ 创建 profile_account                                        │
│ 分配 132 bytes data                                         │
│ 设置 profile_account.owner = Profile Program ID             │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
Profile Program
┌────────────────────────────────────────────────────────────┐
│ CreateProfile                                               │
│                                                            │
│ 检查 payer signer                                           │
│ 检查 profile_account signer                                 │
│ 检查 profile_account.owner == program_id                    │
│ 构造 UserProfile                                            │
│ 写入 profile_account.data                                   │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
最终账户状态
┌────────────────────────────────────────────────────────────┐
│ profile_account                                             │
│                                                            │
│ lamports: rentExemptLamports                                │
│ Account.owner: Profile Program ID                           │
│ data: UserProfile {                                         │
│   is_initialized: true                                      │
│   owner: payer.publicKey                                    │
│   name, age, email                                          │
│ }                                                          │
└────────────────────────────────────────────────────────────┘
```

---

## 9. 最重要的直觉

记住这几句话：

1. `Keypair.generate()` 只是在本地生成密钥对，不会自动在链上创建账户。
2. 链上账户必须通过 `SystemProgram.createAccount` 创建。
3. `createAccount` 会做三件核心事：扣钱、分配空间、设置 `Account.owner`。
4. `createAccount` 创建的是空账户，不会写入业务数据。
5. 业务数据要由你的程序在第二条指令里写入 `Account.data`。
6. 创建账户时需要 `payer` 和 `profileAccount` 两个签名。
7. 后续 update/close 不需要 `profileAccount` 签名，因为权限看的是 `UserProfile.owner` 钱包签名。
