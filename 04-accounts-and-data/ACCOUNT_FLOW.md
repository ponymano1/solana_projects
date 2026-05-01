# 第04节账户图解：账户与数据存储

这份文档只关注账户模型：每个操作涉及哪些账户、客户端传了什么数据、链上操作了哪些账户，以及程序如何鉴权。

第04节比第03节多了一个关键概念：同一个配置文件账户里，同时存在“Solana 账户层面的 owner”和“业务数据层面的 owner”。

---

## 1. 本章有哪些账户

```
┌──────────────────────────────┐
│ Program Account              │
│ 部署后的 UserProfile 程序     │
│                              │
│ key: program_id              │
│ executable: true             │
│ 作用: 执行 create/update/close │
└──────────────────────────────┘

┌──────────────────────────────┐
│ Payer / Owner Wallet         │
│ 用户钱包                      │
│                              │
│ key: payer_pubkey            │
│ signer: true                 │
│ 作用:                         │
│ - 创建时支付租金和交易费       │
│ - 后续作为数据 owner 鉴权      │
│ - 关闭时接收退回的租金         │
└──────────────────────────────┘

┌──────────────────────────────┐
│ Profile Account              │
│ 保存用户配置文件的数据账户     │
│                              │
│ key: profile_account_pubkey  │
│ owner: program_id            │
│ data: UserProfile 序列化数据  │
│ 作用: 保存 name/age/email/owner│
└──────────────────────────────┘

┌──────────────────────────────┐
│ System Program               │
│ Solana 内置系统程序           │
│                              │
│ key: 111111111111...         │
│ 作用: 创建 Profile Account    │
└──────────────────────────────┘
```

一句话版本：

```
Program Account 负责执行代码
Profile Account 负责保存数据
Payer / Owner Wallet 负责签名和权限证明
System Program 负责创建账户
```

---

## 2. 两层 owner：本章最重要的概念

第04节里有两个叫 `owner` 的东西，但它们完全不是一回事。

### 2.1 Solana 账户层面的 owner

这是每个 Solana Account 自带的字段：

```
Profile Account
┌──────────────────────────────────────────┐
│ lamports: rent_exempt_lamports           │
│ owner: program_id                        │  ← Account.owner
│ executable: false                        │
│ data: [序列化后的 UserProfile]            │
└──────────────────────────────────────────┘
```

它表示：哪个程序拥有这个账户的数据写权限。

```
Profile Account.owner = program_id

含义：
  只有 program_id 这个程序可以修改 Profile Account.data
```

### 2.2 业务数据层面的 owner

这是 `UserProfile` 结构体里的字段：

```rust
pub struct UserProfile {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub name: [u8; 32],
    pub name_len: u8,
    pub age: u8,
    pub email: [u8; 64],
    pub email_len: u8,
}
```

它被序列化后存进 `Profile Account.data`：

```
Profile Account.data
┌──────────────────────────────────────────┐
│ UserProfile {                            │
│   is_initialized: true                   │
│   owner: payer_pubkey                    │  ← UserProfile.owner
│   name: "张三"                           │
│   age: 25                                │
│   email: "zhangsan@example.com"          │
│ }                                        │
└──────────────────────────────────────────┘
```

它表示：这个配置文件属于哪个用户。

```
UserProfile.owner = payer_pubkey

含义：
  只有 payer_pubkey 对应的钱包签名，才能更新或关闭这个配置文件
```

### 2.3 两层 owner 总图

```
Profile Account
┌────────────────────────────────────────────────────────┐
│ Account.owner = program_id                             │
│                                                        │
│   账户层权限：谁能写这个账户的 data？                   │
│   答案：program_id 这个程序                             │
│                                                        │
│ Account.data                                           │
│ ┌────────────────────────────────────────────────────┐ │
│ │ UserProfile.owner = payer_pubkey                   │ │
│ │                                                    │ │
│ │   业务层权限：谁能更新/关闭这个配置文件？            │ │
│ │   答案：payer_pubkey 这个用户签名后可以              │ │
│ └────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
```

对比表：

| owner | 在哪里 | 值是什么 | 控制什么 | 什么时候检查 |
|---|---|---|---|---|
| `Account.owner` | Solana 账户字段 | `program_id` | 当前程序能不能写 `Account.data` | 创建配置文件时显式检查 |
| `UserProfile.owner` | 业务数据字段 | `payer_pubkey` | 谁有权更新/关闭配置文件 | 更新、关闭时检查 |

---

## 3. UserProfile 数据布局

`UserProfile::space()` 是 132 字节：

```
Profile Account.data，长度 132 字节
┌──────────────────┬──────────────┬────────────┬──────────┬─────┬─────────────┬───────────┐
│ is_initialized   │ owner        │ name       │ name_len │ age │ email       │ email_len │
│ bool             │ Pubkey       │ [u8; 32]   │ u8       │ u8  │ [u8; 64]    │ u8        │
│ 1 byte           │ 32 bytes     │ 32 bytes   │ 1 byte   │ 1   │ 64 bytes    │ 1 byte    │
└──────────────────┴──────────────┴────────────┴──────────┴─────┴─────────────┴───────────┘
总计: 1 + 32 + 32 + 1 + 1 + 64 + 1 = 132 bytes
```

为什么 `name` 和 `email` 用固定数组？

```
因为 Solana 创建账户时必须提前确定 data 空间大小。

如果 name/email 长度动态变化，账户空间就不好提前固定。
本章用固定最大长度：
  name: 最多 32 字节
  email: 最多 64 字节
```

字符串实际长度单独保存在 `name_len` 和 `email_len` 中：

```
name = "张三"
name bytes = UTF-8 字节
name_len = 实际字节长度
name [u8; 32] = 前面放真实字节，后面补 0
```

---

## 4. 客户端指令的通用结构

客户端发给链上程序的每个指令都是：

```
Instruction
┌────────────────────────────────────┐
│ program_id                         │  调用哪个程序
│ accounts                           │  允许程序访问哪些账户
│ data                               │  业务指令和参数，Borsh 序列化
└────────────────────────────────────┘
```

第04节的指令枚举：

```rust
pub enum ProfileInstruction {
    CreateProfile { name: String, age: u8, email: String },
    UpdateProfile { name: Option<String>, age: Option<u8>, email: Option<String> },
    CloseProfile,
}
```

Borsh 序列化后，`data` 的第一个字节是枚举编号：

| 客户端 `data` 开头 | 链上指令 | 后面跟的数据 |
|---|---|---|
| `[0]` | `CreateProfile` | `name: String, age: u8, email: String` |
| `[1]` | `UpdateProfile` | `name: Option<String>, age: Option<u8>, email: Option<String>` |
| `[2]` | `CloseProfile` | 无 |

Borsh 常见格式：

```
String:
┌──────────────┬──────────────────┐
│ u32 长度小端  │ UTF-8 字节        │
└──────────────┴──────────────────┘

Option<T>:
None:         ┌────┐
              │ 00 │
              └────┘

Some(value):  ┌────┬────────────────┐
              │ 01 │ value 的序列化  │
              └────┴────────────────┘
```

---

## 5. CreateProfile：创建配置文件

创建配置文件分成两个动作：

```
Transaction
┌────────────────────────────────────────────┐
│ signer: payer                              │
│ signer: profile_account                    │
│                                            │
│ Instruction 1: SystemProgram.createAccount │
│ Instruction 2: YourProgram.CreateProfile   │
└────────────────────────────────────────────┘
```

### 5.1 第一条指令：System Program 创建账户

客户端传给 System Program 的数据：

```
SystemProgram.createAccount
┌──────────────────────────────────────────┐
│ fromPubkey: payer                        │  付款人
│ newAccountPubkey: profile_account        │  新建的数据账户
│ lamports: rent_exempt_lamports           │  租金豁免余额
│ space: 132                               │  UserProfile 所需空间
│ programId: program_id                    │  账户 owner 设置成你的程序
└──────────────────────────────────────────┘
```

账户变化：

```
创建前
┌────────────────────┐
│ payer              │ 有 SOL
└────────────────────┘

profile_account 不存在

        │ System Program 执行 createAccount
        ▼

创建后
┌──────────────────────────────────────────┐
│ payer                                    │ SOL 减少：租金 + 交易费
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│ profile_account                          │
│ lamports: rent_exempt_lamports           │
│ owner: program_id                        │
│ data: 132 个 0                           │
└──────────────────────────────────────────┘
```

### 5.2 第二条指令：调用 CreateProfile 写入数据

客户端传给本程序的指令：

```
ProfileProgram.CreateProfile
┌──────────────────────────────────────────┐
│ program_id: program_id                   │
│ accounts:                                │
│   [0] payer                              │ signer, writable
│   [1] profile_account                    │ signer, writable
│   [2] system_program                     │ readonly
│ data:                                    │
│   [0]                                    │ CreateProfile
│   name: String                           │
│   age: u8                                │
│   email: String                          │
└──────────────────────────────────────────┘
```

账户数组顺序和链上代码一一对应：

```rust
let payer_info = next_account_info(account_info_iter)?;
let profile_info = next_account_info(account_info_iter)?;
let _system_program_info = next_account_info(account_info_iter)?;
```

| 客户端 `keys[i]` | 链上变量 | 权限 | 作用 |
|---|---|---|---|
| `keys[0] = payer` | `payer_info` | signer, writable | 支付者；会被写入为 `UserProfile.owner` |
| `keys[1] = profile_account` | `profile_info` | signer, writable | 要初始化的数据账户 |
| `keys[2] = system_program` | `_system_program_info` | readonly | 系统程序账户，本章代码里只取出但未 CPI 调用 |

链上处理流程：

```
process_create_profile
        │
        ├─ 读取 accounts[0] -> payer_info
        ├─ 读取 accounts[1] -> profile_info
        ├─ 读取 accounts[2] -> system_program_info
        │
        ├─ 鉴权 1: payer_info.is_signer == true ?
        │       是：付款人确实签名了
        │       否：MissingRequiredSignature
        │
        ├─ 鉴权 2: profile_info.is_signer == true ?
        │       是：新账户地址的 Keypair 参与了签名
        │       否：MissingRequiredSignature
        │
        ├─ 鉴权 3: profile_info.owner == program_id ?
        │       是：这个账户已经归当前程序管理，可以写 data
        │       否：IncorrectProgramId
        │
        ├─ 数据验证:
        │       name 非空，且 <= 32 字节
        │       email 非空，且 <= 64 字节
        │
        └─ 写入 profile_info.data:
                UserProfile {
                  is_initialized: true,
                  owner: payer_info.key,
                  name,
                  age,
                  email
                }
```

创建完成后的账户：

```
Profile Account
┌──────────────────────────────────────────┐
│ Account.owner: program_id                │
│ data:                                    │
│   UserProfile {                          │
│     is_initialized: true                 │
│     owner: payer_pubkey                  │
│     name: "张三"                         │
│     age: 25                              │
│     email: "zhangsan@example.com"        │
│   }                                      │
└──────────────────────────────────────────┘
```

---

## 6. UpdateProfile：更新配置文件

更新时不再创建账户，只修改已有 `Profile Account.data`。

客户端指令：

```
ProfileProgram.UpdateProfile
┌──────────────────────────────────────────┐
│ program_id: program_id                   │
│ accounts:                                │
│   [0] owner                              │ signer, writable
│   [1] profile_account                    │ writable
│ data:                                    │
│   [1]                                    │ UpdateProfile
│   name: Option<String>                   │ Some 表示更新，None 表示不改
│   age: Option<u8>                        │ Some 表示更新，None 表示不改
│   email: Option<String>                  │ Some 表示更新，None 表示不改
└──────────────────────────────────────────┘
```

账户数组对应链上代码：

```rust
let owner_info = next_account_info(account_info_iter)?;
let profile_info = next_account_info(account_info_iter)?;
```

| 客户端 `keys[i]` | 链上变量 | 权限 | 作用 |
|---|---|---|---|
| `keys[0] = owner` | `owner_info` | signer | 证明“我是这个配置文件的业务 owner” |
| `keys[1] = profile_account` | `profile_info` | writable | 保存并更新 UserProfile 数据 |

链上处理流程：

```
process_update_profile
        │
        ├─ 读取 accounts[0] -> owner_info
        ├─ 读取 accounts[1] -> profile_info
        │
        ├─ 鉴权 1: owner_info.is_signer == true ?
        │       是：调用者确实控制这个钱包
        │       否：MissingRequiredSignature
        │
        ├─ 读取 profile_info.data
        ├─ 反序列化为 UserProfile
        │
        ├─ 鉴权 2: profile.is_initialized == true ?
        │       是：这是一个已初始化的配置文件
        │       否：UninitializedAccount
        │
        ├─ 鉴权 3: profile.owner == owner_info.key ?
        │       是：签名者就是这个配置文件的业务 owner
        │       否：IllegalOwner
        │
        ├─ 根据 Option 字段更新:
        │       name = Some(x)  -> 更新 name
        │       name = None     -> 不改 name
        │       age = Some(x)   -> 更新 age
        │       age = None      -> 不改 age
        │       email 同理
        │
        └─ 序列化并写回 profile_info.data
```

示例：只更新名字和年龄，不更新邮箱。

```
客户端 data:
┌────┬──────────────┬────────────┬──────────────┐
│ 01 │ Some("李四") │ Some(26)   │ None         │
└────┴──────────────┴────────────┴──────────────┘
  ↑        ↑             ↑             ↑
Update    name          age           email 不变
```

账户变化：

```
执行前
UserProfile {
  owner: payer_pubkey,
  name: "张三",
  age: 25,
  email: "zhangsan@example.com"
}

        │ owner 签名后调用 UpdateProfile
        ▼

执行后
UserProfile {
  owner: payer_pubkey,
  name: "李四",
  age: 26,
  email: "zhangsan@example.com"
}
```

为什么更新时 `profile_account` 不需要签名？

```
因为权限不是靠 profile_account 私钥证明的。

更新权限来自：
  1. profile_account 是当前程序可写的数据账户
  2. UserProfile.owner 字段等于签名的钱包 owner_info.key

所以需要签名的是 owner_info，不是 profile_account。
```

---

## 7. CloseProfile：关闭配置文件并退租金

关闭账户时，程序会把 `Profile Account` 里的 lamports 转回给 owner，然后把 profile 账户余额清零、data 清零。

客户端指令：

```
ProfileProgram.CloseProfile
┌──────────────────────────────────────────┐
│ program_id: program_id                   │
│ accounts:                                │
│   [0] owner                              │ signer, writable
│   [1] profile_account                    │ writable
│ data: [2]                                │ CloseProfile
└──────────────────────────────────────────┘
```

账户数组对应链上代码：

```rust
let owner_info = next_account_info(account_info_iter)?;
let profile_info = next_account_info(account_info_iter)?;
```

| 客户端 `keys[i]` | 链上变量 | 权限 | 作用 |
|---|---|---|---|
| `keys[0] = owner` | `owner_info` | signer, writable | 证明权限，并接收退回租金 |
| `keys[1] = profile_account` | `profile_info` | writable | 被关闭的数据账户 |

链上处理流程：

```
process_close_profile
        │
        ├─ 读取 accounts[0] -> owner_info
        ├─ 读取 accounts[1] -> profile_info
        │
        ├─ 鉴权 1: owner_info.is_signer == true ?
        │       是：owner 钱包签名了
        │       否：MissingRequiredSignature
        │
        ├─ 读取 profile_info.data
        ├─ 反序列化为 UserProfile
        │
        ├─ 鉴权 2: profile.is_initialized == true ?
        │       是：可以关闭
        │       否：UninitializedAccount
        │
        ├─ 鉴权 3: profile.owner == owner_info.key ?
        │       是：签名者就是业务 owner
        │       否：IllegalOwner
        │
        ├─ lamports 转移:
        │       owner_info.lamports += profile_info.lamports
        │       profile_info.lamports = 0
        │
        └─ 清空 profile_info.data
```

账户变化：

```
关闭前
┌──────────────────────────────────────────┐
│ owner wallet                             │
│ lamports: A                              │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│ profile_account                          │
│ lamports: rent_exempt_lamports           │
│ data: UserProfile {...}                  │
└──────────────────────────────────────────┘

        │ CloseProfile
        ▼

关闭后
┌──────────────────────────────────────────┐
│ owner wallet                             │
│ lamports: A + rent_exempt_lamports       │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│ profile_account                          │
│ lamports: 0                              │
│ data: [0, 0, 0, ...]                     │
└──────────────────────────────────────────┘
```

在测试环境里，余额为 0 的账户会表现为账户不存在。

---

## 8. 三个操作的账户对照表

| 操作 | accounts[0] | accounts[1] | accounts[2] | 主要写入谁 | 谁必须签名 |
|---|---|---|---|---|---|
| `CreateProfile` | payer | profile_account | system_program | profile_account.data | payer + profile_account |
| `UpdateProfile` | owner | profile_account | 无 | profile_account.data | owner |
| `CloseProfile` | owner | profile_account | 无 | owner.lamports + profile_account.lamports/data | owner |

---

## 9. 三个操作的鉴权对照表

| 操作 | 鉴权检查 | 目的 |
|---|---|---|
| `CreateProfile` | `payer_info.is_signer` | 确认付款人授权 |
| `CreateProfile` | `profile_info.is_signer` | 确认新账户 Keypair 授权 |
| `CreateProfile` | `profile_info.owner == program_id` | 确认这个数据账户归当前程序管理 |
| `UpdateProfile` | `owner_info.is_signer` | 确认调用者控制这个钱包 |
| `UpdateProfile` | `profile.is_initialized` | 确认数据已经初始化 |
| `UpdateProfile` | `profile.owner == owner_info.key` | 确认调用者是业务数据 owner |
| `CloseProfile` | `owner_info.is_signer` | 确认调用者控制这个钱包 |
| `CloseProfile` | `profile.is_initialized` | 确认数据已经初始化 |
| `CloseProfile` | `profile.owner == owner_info.key` | 确认调用者是业务数据 owner |

---

## 10. `isSigner`、`isWritable`、`owner` 的区别

这三个概念经常混淆：

```
isSigner
  本次交易里，这个账户对应的私钥有没有签名。
  用来证明“我控制这个地址”。

isWritable
  本次交易是否允许程序修改这个账户。
  如果程序要改 data 或 lamports，就必须标 writable。

Account.owner
  这个账户的数据归哪个程序管理。
  只有 owner 程序能修改 Account.data。

UserProfile.owner
  业务字段，表示这个配置文件属于哪个用户。
  本章用它判断谁能 update/close。
```

用本章的更新操作举例：

```
UpdateProfile
┌────────────────────────────────────────────┐
│ owner wallet                               │
│   isSigner: true                           │  证明用户授权
│   UserProfile.owner 必须等于这个地址        │
│                                            │
│ profile_account                            │
│   isWritable: true                         │  因为要改 data
│   isSigner: false                          │  不靠它的私钥鉴权
│   Account.owner: program_id                │  表示当前程序可写它的 data
└────────────────────────────────────────────┘
```

---

## 11. 一张完整调用总图

```
客户端
┌────────────────────────────────────────────────────────────┐
│ 构造 TransactionInstruction                                │
│                                                            │
│ program_id = Profile Program ID                            │
│ accounts = 按链上 next_account_info 顺序排列                │
│ data = Borsh(ProfileInstruction)                           │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
Solana Runtime
┌────────────────────────────────────────────────────────────┐
│ 1. 检查交易签名                                             │
│ 2. 加载 accounts 列表                                       │
│ 3. 根据 isWritable 锁定可写账户                              │
│ 4. 调用 program_id 对应的链上程序                            │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
Profile Program
┌────────────────────────────────────────────────────────────┐
│ process_instruction(program_id, accounts, instruction_data) │
│                                                            │
│ 1. Borsh 反序列化 instruction_data                          │
│ 2. match 到 Create / Update / Close                         │
│ 3. 按顺序读取 accounts                                      │
│ 4. 做 signer / owner / initialized 检查                      │
│ 5. 读写 Profile Account.data 或 lamports                     │
└───────────────────────────────┬────────────────────────────┘
                                │
                                ▼
Profile Account
┌────────────────────────────────────────────────────────────┐
│ Account.owner = program_id                                 │
│ Account.data = UserProfile {                               │
│   is_initialized, owner, name, age, email                   │
│ }                                                          │
└────────────────────────────────────────────────────────────┘
```

---

## 12. 本章应该形成的账户模型直觉

记住这几句话：

1. 账户是 Solana 的存储单元，业务数据存在 `Account.data`。
2. 程序和数据分离：程序账户执行代码，数据账户保存状态。
3. 客户端必须显式传入程序要访问的账户，链上按顺序读取。
4. 创建数据账户通常先调用 `System Program.createAccount`。
5. `Account.owner = program_id` 表示这个账户的数据归哪个程序管理。
6. `UserProfile.owner = user_pubkey` 是业务权限字段，表示谁能更新/关闭这份数据。
7. `isSigner` 证明某个地址授权了本次交易，`isWritable` 表示本次交易允许修改该账户。
8. 创建时关注“账户是否归程序管理”，更新/关闭时关注“签名者是否是业务 owner”。
