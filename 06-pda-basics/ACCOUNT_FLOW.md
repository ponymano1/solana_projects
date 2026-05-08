# 第06节账户与交互图解：PDA 投票系统

这份文档专门解释第06节 PDA 投票系统：TypeScript 示例里哪些地方要注意、程序用到了哪些账户、每个账户存了什么、每个操作客户端发送什么、链上如何校验权限。

先说结论：第06节的 TypeScript 示例大方向是对的，PDA seeds 和账户顺序基本匹配链上代码；但它是“教学示例”，有几个容易误解或实际开发时要注意的点：

1. 手写 Borsh 序列化必须和 Rust 枚举字段顺序完全一致。
2. PDA 没有私钥，所以 `topicPDA` 和 `userVotePDA` 永远不是 signer。
3. `bump` 从客户端传入，但链上必须重新计算并校验，不能直接信客户端。
4. `VoteTopic PDA` 的 seeds 是 `["vote_topic", creator]`，所以当前设计里一个 creator 只能有一个 topic PDA。
5. `UserVote PDA` 的 seeds 是 `["user_vote", topic, voter]`，所以一个 voter 对一个 topic 只能有一个投票记录。

---

## 1. 本章用到了哪些账户

### 1.1 CreateTopic 用到的账户

创建投票主题时，客户端传 3 个账户：

```text
CreateTopic accounts
┌────────────────────────────────────────────┐
│ [0] creator_account                        │ signer, writable
│ [1] topic_account                          │ writable, PDA
│ [2] system_program                         │ readonly
└────────────────────────────────────────────┘
```

对应链上代码：

```rust
let creator_account = next_account_info(accounts_iter)?;
let topic_account = next_account_info(accounts_iter)?;
let system_program = next_account_info(accounts_iter)?;
```

账户说明：

```text
┌──────────────────────────────┐
│ Creator Account              │
│ 创建者钱包                    │
│                              │
│ 有私钥                        │
│ signer: true                 │
│ writable: true               │
│ 作用:                         │
│ - 支付创建 topic PDA 的租金    │
│ - 成为 VoteTopic.creator      │
└──────────────────────────────┘

┌──────────────────────────────┐
│ VoteTopic PDA                │
│ 投票主题数据账户              │
│                              │
│ 没有私钥                      │
│ signer: false                │
│ writable: true               │
│ seeds:                       │
│   ["vote_topic", creator]    │
│ 作用:                         │
│ - 保存主题描述                 │
│ - 保存 A/B 票数                │
│ - 保存 creator 和 bump         │
└──────────────────────────────┘

┌──────────────────────────────┐
│ System Program               │
│ Solana 内置系统程序           │
│                              │
│ signer: false                │
│ writable: false              │
│ 作用:                         │
│ - 被当前程序 CPI 调用          │
│ - 创建 VoteTopic PDA          │
└──────────────────────────────┘
```

---

### 1.2 Vote 用到的账户

投票时，客户端传 4 个账户：

```text
Vote accounts
┌────────────────────────────────────────────┐
│ [0] voter_account                          │ signer, writable
│ [1] topic_account                          │ writable, PDA
│ [2] user_vote_account                      │ writable, PDA
│ [3] system_program                         │ readonly
└────────────────────────────────────────────┘
```

对应链上代码：

```rust
let voter_account = next_account_info(accounts_iter)?;
let topic_account = next_account_info(accounts_iter)?;
let user_vote_account = next_account_info(accounts_iter)?;
let system_program = next_account_info(accounts_iter)?;
```

账户说明：

```text
┌──────────────────────────────┐
│ Voter Account                │
│ 投票者钱包                    │
│                              │
│ 有私钥                        │
│ signer: true                 │
│ writable: true               │
│ 作用:                         │
│ - 支付创建 UserVote PDA 的租金 │
│ - 证明是谁在投票              │
└──────────────────────────────┘

┌──────────────────────────────┐
│ VoteTopic PDA                │
│ 投票主题账户                  │
│                              │
│ 没有私钥                      │
│ signer: false                │
│ writable: true               │
│ 作用:                         │
│ - 读取主题信息                 │
│ - 更新 option_a_votes 或       │
│   option_b_votes              │
└──────────────────────────────┘

┌──────────────────────────────┐
│ UserVote PDA                 │
│ 用户投票记录账户              │
│                              │
│ 没有私钥                      │
│ signer: false                │
│ writable: true               │
│ seeds:                       │
│   ["user_vote", topic, voter]│
│ 作用:                         │
│ - 记录某个用户对某个主题投过票 │
│ - 防止重复投票                │
└──────────────────────────────┘

┌──────────────────────────────┐
│ System Program               │
│ Solana 内置系统程序           │
│                              │
│ signer: false                │
│ writable: false              │
│ 作用:                         │
│ - 被当前程序 CPI 调用          │
│ - 创建 UserVote PDA           │
└──────────────────────────────┘
```

---

## 2. 每个账户里存了什么

### 2.1 VoteTopic PDA 存储内容

链上账户外层：

```text
VoteTopic PDA Account
┌────────────────────────────────────────────┐
│ key: topic_pda                             │
│ lamports: rent_exempt_lamports             │
│ Account.owner: program_id                  │
│ executable: false                          │
│ data: Borsh(VoteTopic)                     │
└────────────────────────────────────────────┘
```

`data` 里是：

```rust
pub struct VoteTopic {
    pub is_initialized: bool,
    pub creator: Pubkey,
    pub description: String,
    pub option_a_votes: u64,
    pub option_b_votes: u64,
    pub bump: u8,
}
```

图示：

```text
VoteTopic PDA.data
┌────────────────────────────────────────────┐
│ VoteTopic {                                │
│   is_initialized: true                     │
│   creator: creator_pubkey                  │
│   description: "你喜欢Solana吗？"           │
│   option_a_votes: 0                        │
│   option_b_votes: 0                        │
│   bump: topic_bump                         │
│ }                                          │
└────────────────────────────────────────────┘
```

空间大小是动态的，和 `description` 字节长度有关：

```text
VoteTopic::space(description_len)
= 1 + 32 + 4 + description_len + 8 + 8 + 1
```

其中：

```text
1                  is_initialized
32                 creator Pubkey
4 + description_len String 的 Borsh 格式
8                  option_a_votes
8                  option_b_votes
1                  bump
```

---

### 2.2 UserVote PDA 存储内容

链上账户外层：

```text
UserVote PDA Account
┌────────────────────────────────────────────┐
│ key: user_vote_pda                         │
│ lamports: rent_exempt_lamports             │
│ Account.owner: program_id                  │
│ executable: false                          │
│ data: Borsh(UserVote)                      │
└────────────────────────────────────────────┘
```

`data` 里是：

```rust
pub struct UserVote {
    pub is_initialized: bool,
    pub topic: Pubkey,
    pub voter: Pubkey,
    pub vote_option: u8,
    pub bump: u8,
}
```

图示：

```text
UserVote PDA.data
┌────────────────────────────────────────────┐
│ UserVote {                                 │
│   is_initialized: true                     │
│   topic: topic_pda                         │
│   voter: voter_pubkey                      │
│   vote_option: 0                           │  0=A, 1=B
│   bump: user_vote_bump                     │
│ }                                          │
└────────────────────────────────────────────┘
```

空间大小固定：

```text
UserVote::space()
= 1 + 32 + 32 + 1 + 1
= 67 bytes
```

---

## 3. PDA 地址是怎么来的

### 3.1 VoteTopic PDA

客户端：

```typescript
const [topicPDA, topicBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('vote_topic'),
    creator.publicKey.toBuffer(),
  ],
  PROGRAM_ID
);
```

链上：

```rust
let (expected_pda, expected_bump) = Pubkey::find_program_address(
    &[b"vote_topic", creator_account.key.as_ref()],
    program_id,
);
```

两边必须完全一致：

```text
seeds = ["vote_topic", creator_pubkey]
program_id = 当前投票程序 ID
```

当前设计的含义：

```text
一个 creator_pubkey 对应一个固定 topic PDA。

同一个 creator 如果想创建多个主题，当前 seeds 不够，
需要把主题 id、标题 hash 或自增编号也加入 seeds。
```

---

### 3.2 UserVote PDA

客户端：

```typescript
const [userVotePDA, userVoteBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('user_vote'),
    topicPDA.toBuffer(),
    voter.publicKey.toBuffer(),
  ],
  PROGRAM_ID
);
```

链上：

```rust
let (expected_pda, expected_bump) = Pubkey::find_program_address(
    &[
        b"user_vote",
        topic_account.key.as_ref(),
        voter_account.key.as_ref(),
    ],
    program_id,
);
```

两边必须完全一致：

```text
seeds = ["user_vote", topic_pda, voter_pubkey]
program_id = 当前投票程序 ID
```

当前设计的含义：

```text
一个 voter 对一个 topic 只有一个 userVotePDA。
所以可以用这个 PDA 防止重复投票。
```

---

## 4. bump 是什么，前端和链上分别怎么得到

### 4.1 bump 是什么

`bump` 是 PDA 派生时额外加进去的一个 `u8` seed，范围是 `0~255`。

PDA 有一个核心要求：

```text
PDA 必须不在 Ed25519 曲线上。
```

因为如果地址在曲线上，理论上就可能存在对应私钥；而 PDA 的设计目标是“没有私钥，只能由程序通过 seeds 代表它签名”。

所以 Solana 会不断尝试不同的 bump：

```text
seeds + 255 + program_id  → 派生地址 → 如果不在曲线上，成功
seeds + 254 + program_id  → 派生地址 → 如果不在曲线上，成功
seeds + 253 + program_id  → 派生地址 → 如果不在曲线上，成功
...
```

第一个成功的 bump，就是最终 bump。

可以把 bump 理解成：

```text
为了让 seeds 能稳定派生出一个合法 PDA 地址，额外补进去的那个小数字。
```

---

### 4.2 前端怎么得到 bump

前端用 `findProgramAddressSync`，它一次返回两个值：

```typescript
const [topicPDA, topicBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('vote_topic'),
    creator.publicKey.toBuffer(),
  ],
  PROGRAM_ID
);
```

返回：

```text
topicPDA   = 派生出来的 PDA 地址
topicBump  = 对应这个 PDA 的 bump
```

用户投票记录也是一样：

```typescript
const [userVotePDA, userVoteBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('user_vote'),
    topicPDA.toBuffer(),
    voter.publicKey.toBuffer(),
  ],
  PROGRAM_ID
);
```

返回：

```text
userVotePDA   = 用户投票记录 PDA 地址
userVoteBump  = 对应这个 PDA 的 bump
```

前端会把 bump 放进 instruction data：

```text
CreateTopic data:
  [0, description, topicBump]

Vote data:
  [1, option, userVoteBump]
```

---

### 4.3 链上怎么得到 bump

链上也用同样的 seeds 重新计算：

```rust
let (expected_pda, expected_bump) = Pubkey::find_program_address(
    &[b"vote_topic", creator_account.key.as_ref()],
    program_id,
);
```

或者：

```rust
let (expected_pda, expected_bump) = Pubkey::find_program_address(
    &[
        b"user_vote",
        topic_account.key.as_ref(),
        voter_account.key.as_ref(),
    ],
    program_id,
);
```

链上会拿这两个结果和客户端传入的账户、bump 做比较：

```rust
if topic_account.key != &expected_pda {
    return Err(VoteError::InvalidPDA.into());
}

if bump != expected_bump {
    return Err(VoteError::InvalidPDA.into());
}
```

含义是：

```text
客户端传来的 PDA 地址，必须等于链上重新算出来的 PDA。
客户端传来的 bump，必须等于链上重新算出来的 bump。
```

链上不能直接相信前端传来的 bump，因为前端可能传错，也可能恶意传假账户。

---

### 4.4 bump 是不是 Solana 自动存到链上的

不是。

Solana 系统层面不会自动给 PDA 账户保存 bump：

```text
PDA 账户没有一个系统字段叫 bump。
Runtime 不会帮你保存 bump。
```

但是程序可以选择把 bump 存进自己的账户数据里。第06节就是这么做的。

`VoteTopic` 里有：

```rust
pub struct VoteTopic {
    pub is_initialized: bool,
    pub creator: Pubkey,
    pub description: String,
    pub option_a_votes: u64,
    pub option_b_votes: u64,
    pub bump: u8,
}
```

所以 `topicBump` 被存进：

```text
VoteTopic PDA.data.bump
```

`UserVote` 里也有：

```rust
pub struct UserVote {
    pub is_initialized: bool,
    pub topic: Pubkey,
    pub voter: Pubkey,
    pub vote_option: u8,
    pub bump: u8,
}
```

所以 `userVoteBump` 被存进：

```text
UserVote PDA.data.bump
```

也就是说：

```text
bump 不是 Solana 自动存的。
第06节是程序自己把 bump 当作业务数据写进了 PDA account.data。
```

---

### 4.5 为什么要把 bump 存到账户 data 里

不是必须，但很常见。

因为以后如果程序还要对这个 PDA 做 `invoke_signed`，需要提供：

```text
seeds + bump
```

如果 bump 已经存在账户 data 里，后续可以直接读出来用：

```rust
let vote_topic = VoteTopic::deserialize(...)?;
let bump = vote_topic.bump;
```

如果不存，也可以每次重新计算：

```rust
let (_, bump) = Pubkey::find_program_address(
    &[b"vote_topic", creator_account.key.as_ref()],
    program_id,
);
```

两种方式都可以：

| 方式 | 做法 | 优点 | 缺点 |
|---|---|---|---|
| 存 bump | 初始化时写入账户 data | 后续使用方便，少重新计算 | 多占 1 byte，需要保证写入前已校验 |
| 不存 bump | 每次重新 `find_program_address` | 账户数据少，不怕存错 | 每次都要重新计算 |

第06节采用的是：

```text
前端计算 bump
  ↓
前端把 bump 放进 instruction data
  ↓
链上重新计算 expected_bump
  ↓
链上校验 bump == expected_bump
  ↓
链上把 bump 写进 VoteTopic / UserVote 的 data
```

---

### 4.6 bump 在 invoke_signed 里怎么用

创建 PDA 账户时，PDA 没有私钥，不能自己签名。

所以程序要把 seeds + bump 传给 `invoke_signed`：

```rust
invoke_signed(
    &system_instruction::create_account(...),
    &[creator_account.clone(), topic_account.clone(), system_program.clone()],
    &[&[b"vote_topic", creator_account.key.as_ref(), &[bump]]],
)?;
```

Runtime 会用这些 seeds + bump + 当前 program_id 重新派生 PDA。

如果派生结果等于 `topic_account.key`，Runtime 就认为：

```text
当前程序可以代表这个 PDA 签名。
```

所以 bump 在 `invoke_signed` 里的作用是：

```text
帮助 Runtime 验证这个 PDA 真的是由当前程序用这组 seeds 派生出来的。
```

---

## 5. 为什么 PDA 不需要 signer

PDA 没有私钥，所以客户端不可能让 PDA 签名：

```text
topicPDA.isSigner = false
userVotePDA.isSigner = false
```

PDA 创建时，System Program 的 `create_account` 需要“新账户签名”。但是 PDA 没有私钥，怎么办？

答案是程序用 `invoke_signed` 代表 PDA 签名：

```rust
invoke_signed(
    &system_instruction::create_account(...),
    &[creator_account.clone(), topic_account.clone(), system_program.clone()],
    &[&[b"vote_topic", creator_account.key.as_ref(), &[bump]]],
)?;
```

可以理解成：

```text
客户端不能替 PDA 签名。
当前程序可以用正确 seeds + bump 证明：
  这个 PDA 是我这个 program_id 派生出来的。

Solana Runtime 验证 seeds 正确后，允许程序代表 PDA 完成创建账户。
```

---

## 6. TypeScript 示例里要注意的问题

### 5.1 手写 Borsh 序列化必须完全匹配 Rust

Rust 指令：

```rust
pub enum VoteInstruction {
    CreateTopic { description: String, bump: u8 },
    Vote { option: u8, bump: u8 },
}
```

Borsh 枚举序列化格式：

```text
CreateTopic:
┌────┬──────────────────────┬──────┐
│ 00 │ description: String  │ bump │
└────┴──────────────────────┴──────┘

Vote:
┌────┬────────┬──────┐
│ 01 │ option │ bump │
└────┴────────┴──────┘
```

String 的 Borsh 格式：

```text
String = u32 小端长度 + UTF-8 bytes
```

所以 README 中 `serializeCreateTopic` 的思路是对的：

```typescript
function serializeCreateTopic(description: string, bump: number): Buffer {
  const descBytes = Buffer.from(description, 'utf-8');
  const buffer = Buffer.alloc(1 + 4 + descBytes.length + 1);

  let offset = 0;
  buffer[offset] = 0;
  offset += 1;

  buffer.writeUInt32LE(descBytes.length, offset);
  offset += 4;

  descBytes.copy(buffer, offset);
  offset += descBytes.length;

  buffer[offset] = bump;

  return buffer;
}
```

但实际项目更建议用稳定的 Borsh 库或统一封装，避免手写 offset 出错。

### 5.2 `description.length` 和字节长度不是一回事

链上检查：

```rust
if description.len() > VoteTopic::MAX_DESCRIPTION_LEN
```

Rust `String::len()` 是 UTF-8 字节数，不是字符数。

TS 里也应该用：

```typescript
Buffer.from(description, 'utf-8').length
```

不要用：

```typescript
description.length
```

中文字符通常占 3 个字节。

### 5.3 `topicPDA` / `userVotePDA` 不能放进 signers

错误做法：

```typescript
await sendAndConfirmTransaction(connection, tx, [creator, topicPDA]);
```

PDA 是 `PublicKey`，没有私钥，不能签名。

正确做法：

```typescript
await sendAndConfirmTransaction(connection, tx, [creator]);
```

投票也是：

```typescript
await sendAndConfirmTransaction(connection, tx, [voter]);
```

### 5.4 创建 PDA 账户不是客户端直接 `SystemProgram.createAccount`

普通 Keypair 账户常见写法：

```typescript
SystemProgram.createAccount({ ... })
```

但第06节的 PDA 账户不是客户端直接创建，而是：

```text
客户端把 PDA 地址传给程序
程序内部用 invoke_signed CPI 调用 System Program 创建 PDA 账户
```

所以客户端只构造 `CreateTopic` 或 `Vote` 指令，不需要自己额外加 `SystemProgram.createAccount` 指令。

### 5.5 当前重复投票逻辑依赖 UserVote PDA 是否已存在

第一次投票会创建：

```text
user_vote_pda = derive(["user_vote", topic_pda, voter])
```

第二次同一个 voter 对同一个 topic 投票时，会传入同一个 `user_vote_pda`。

链上检查：

```rust
if user_vote_account.owner == program_id && user_vote_account.data_len() > 0 {
    return Err(VoteError::AlreadyVoted.into());
}
```

所以这个 PDA 的存在本身就是“已经投过票”的证明。

---

## 7. 操作一：CreateTopic

### 6.1 客户端先计算 PDA

```typescript
const [topicPDA, topicBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('vote_topic'),
    creator.publicKey.toBuffer(),
  ],
  PROGRAM_ID
);
```

得到：

```text
topicPDA   = 投票主题 PDA 地址
topicBump  = 创建该 PDA 所需 bump
```

### 6.2 客户端发送的账户

```typescript
const createTopicIx = new TransactionInstruction({
  keys: [
    { pubkey: creator.publicKey, isSigner: true, isWritable: true },
    { pubkey: topicPDA, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: serializeCreateTopic(description, topicBump),
});
```

账户表：

| index | 账户 | signer | writable | 作用 |
|---|---|---:|---:|---|
| `[0]` | `creator` | 是 | 是 | 支付租金，成为主题创建者 |
| `[1]` | `topicPDA` | 否 | 是 | 被创建并保存 VoteTopic 数据 |
| `[2]` | `SystemProgram` | 否 | 否 | 被 CPI 调用创建 PDA 账户 |

### 6.3 客户端发送的 data

```text
CreateTopic data
┌────┬──────────────────────┬────────────┐
│ 00 │ description: String  │ topic_bump │
└────┴──────────────────────┴────────────┘
```

例如：

```text
description = "你喜欢Solana吗？"
bump = 253
```

序列化后结构：

```text
[0, desc_len(u32), desc_utf8_bytes..., 253]
```

### 6.4 链上校验

链上 `process_create_topic` 做这些检查：

```text
1. creator_account.is_signer == true
   目的：确认创建者授权，并允许从 creator 扣租金

2. description.len() <= VoteTopic::MAX_DESCRIPTION_LEN
   目的：限制账户数据大小，当前最大 200 字节

3. 重新计算 expected_pda
   seeds = ["vote_topic", creator_account.key]

4. topic_account.key == expected_pda
   目的：防止客户端传入错误账户

5. bump == expected_bump
   目的：确认客户端传入的 bump 是正确 bump
```

### 6.5 链上创建账户

校验通过后，程序内部 CPI：

```rust
invoke_signed(
    &system_instruction::create_account(
        creator_account.key,
        topic_account.key,
        lamports,
        space as u64,
        program_id,
    ),
    &[
        creator_account.clone(),
        topic_account.clone(),
        system_program.clone(),
    ],
    &[&[b"vote_topic", creator_account.key.as_ref(), &[bump]]],
)?;
```

效果：

```text
creator_account.lamports -= rent

topic_account:
  lamports = rent
  Account.owner = program_id
  data_len = VoteTopic::space(description.len())
```

### 6.6 链上写入数据

```text
VoteTopic {
  is_initialized: true,
  creator: creator_account.key,
  description,
  option_a_votes: 0,
  option_b_votes: 0,
  bump,
}
```

---

## 8. 操作二：Vote

### 7.1 客户端先计算两个 PDA

先计算主题 PDA：

```typescript
const [topicPDA] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('vote_topic'),
    creator.publicKey.toBuffer(),
  ],
  PROGRAM_ID
);
```

再计算用户投票记录 PDA：

```typescript
const [userVotePDA, userVoteBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('user_vote'),
    topicPDA.toBuffer(),
    voter.publicKey.toBuffer(),
  ],
  PROGRAM_ID
);
```

注意：投票时需要知道 `topicPDA`。如果你只知道 creator，就能按当前 seeds 算出 topicPDA；如果将来支持一个 creator 多个 topic，就必须额外知道 topic id 或其他 seed。

### 7.2 客户端发送的账户

```typescript
const voteIx = new TransactionInstruction({
  keys: [
    { pubkey: voter.publicKey, isSigner: true, isWritable: true },
    { pubkey: topicPDA, isSigner: false, isWritable: true },
    { pubkey: userVotePDA, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: serializeVote(0, userVoteBump),
});
```

账户表：

| index | 账户 | signer | writable | 作用 |
|---|---|---:|---:|---|
| `[0]` | `voter` | 是 | 是 | 投票者，支付创建 UserVote PDA 的租金 |
| `[1]` | `topicPDA` | 否 | 是 | 读取主题并更新票数 |
| `[2]` | `userVotePDA` | 否 | 是 | 创建并保存用户投票记录 |
| `[3]` | `SystemProgram` | 否 | 否 | 被 CPI 调用创建 UserVote PDA |

### 7.3 客户端发送的 data

```text
Vote data
┌────┬──────────────┬────────────────┐
│ 01 │ option: u8   │ user_vote_bump │
└────┴──────────────┴────────────────┘
```

其中：

```text
option = 0 表示选项 A
option = 1 表示选项 B
```

TS：

```typescript
function serializeVote(option: number, bump: number): Buffer {
  const buffer = Buffer.alloc(3);
  buffer[0] = 1;
  buffer[1] = option;
  buffer[2] = bump;
  return buffer;
}
```

### 7.4 链上校验

链上 `process_vote` 做这些检查：

```text
1. voter_account.is_signer == true
   目的：确认投票者授权，并允许从 voter 扣租金

2. option <= 1
   目的：只能投 A 或 B

3. topic_account.owner == program_id
   目的：确认主题账户归当前程序管理

4. 反序列化 VoteTopic

5. vote_topic.is_initialized == true
   目的：确认主题已经创建

6. 重新计算 expected user_vote_pda
   seeds = ["user_vote", topic_account.key, voter_account.key]

7. user_vote_account.key == expected_pda
   目的：防止客户端传入错误投票记录账户

8. bump == expected_bump
   目的：确认客户端传入的 user_vote_bump 正确

9. user_vote_account.owner == program_id && data_len > 0 ?
   如果是，说明这个 userVote PDA 已经存在，返回 AlreadyVoted
```

### 7.5 链上创建 UserVote PDA

如果用户投票记录账户还不存在，程序 CPI 创建它：

```rust
invoke_signed(
    &system_instruction::create_account(
        voter_account.key,
        user_vote_account.key,
        lamports,
        space as u64,
        program_id,
    ),
    &[
        voter_account.clone(),
        user_vote_account.clone(),
        system_program.clone(),
    ],
    &[&[
        b"user_vote",
        topic_account.key.as_ref(),
        voter_account.key.as_ref(),
        &[bump],
    ]],
)?;
```

效果：

```text
voter_account.lamports -= rent

user_vote_account:
  lamports = rent
  Account.owner = program_id
  data_len = UserVote::space() = 67
```

### 7.6 链上写入 UserVote 并更新票数

写入用户投票记录：

```text
UserVote {
  is_initialized: true,
  topic: topic_account.key,
  voter: voter_account.key,
  vote_option: option,
  bump,
}
```

更新主题票数：

```text
if option == 0:
  vote_topic.option_a_votes += 1
else:
  vote_topic.option_b_votes += 1
```

账户变化示例：

```text
投票前
VoteTopic {
  option_a_votes: 0,
  option_b_votes: 0,
}
UserVote PDA 不存在

voter 投 option = 0

投票后
VoteTopic {
  option_a_votes: 1,
  option_b_votes: 0,
}
UserVote PDA {
  topic: topic_pda,
  voter: voter_pubkey,
  vote_option: 0,
}
```

---

## 9. 两个操作总表

| 操作 | accounts | data | 创建账户 | 修改数据 |
|---|---|---|---|---|
| `CreateTopic` | creator, topicPDA, systemProgram | `[0, description, topic_bump]` | 创建 `VoteTopic PDA` | 写入 `VoteTopic` |
| `Vote` | voter, topicPDA, userVotePDA, systemProgram | `[1, option, user_vote_bump]` | 创建 `UserVote PDA` | 写入 `UserVote`，更新 `VoteTopic` 票数 |

---

## 10. 权限和校验总表

| 校验 | CreateTopic | Vote | 目的 |
|---|---|---|---|
| `creator/voter.is_signer` | 是 | 是 | 用户必须授权，并支付 PDA 账户租金 |
| 描述长度 | 是 | 否 | 限制 `VoteTopic` 账户大小 |
| 投票选项 `option <= 1` | 否 | 是 | 只允许 A/B 两个选项 |
| 重新计算 PDA | 是 | 是 | 不信任客户端传入地址 |
| 校验 PDA key | 是 | 是 | 确认账户地址是预期 PDA |
| 校验 bump | 是 | 是 | 确认 PDA bump 正确 |
| `topic_account.owner == program_id` | 否 | 是 | 确认主题账户归当前程序管理 |
| `vote_topic.is_initialized` | 否 | 是 | 确认主题已经创建 |
| 检查 `UserVote PDA` 是否已存在 | 否 | 是 | 防止重复投票 |

---

## 11. 一张总流程图

```text
CreateTopic

客户端
  ├─ 计算 topicPDA = derive(["vote_topic", creator])
  ├─ 构造 data = [0, description, topic_bump]
  └─ 发送 accounts = [creator, topicPDA, systemProgram]
        │
        ▼
链上程序
  ├─ 检查 creator signer
  ├─ 检查 description 长度
  ├─ 重新计算 topicPDA 和 bump
  ├─ invoke_signed 创建 topicPDA 账户
  └─ 写入 VoteTopic 数据
```

```text
Vote

客户端
  ├─ 计算 topicPDA
  ├─ 计算 userVotePDA = derive(["user_vote", topicPDA, voter])
  ├─ 构造 data = [1, option, user_vote_bump]
  └─ 发送 accounts = [voter, topicPDA, userVotePDA, systemProgram]
        │
        ▼
链上程序
  ├─ 检查 voter signer
  ├─ 检查 option
  ├─ 检查 topic_account.owner
  ├─ 读取 VoteTopic
  ├─ 重新计算 userVotePDA 和 bump
  ├─ 检查是否已投票
  ├─ invoke_signed 创建 userVotePDA
  ├─ 写入 UserVote
  └─ 更新 VoteTopic 票数
```

---

## 12. 最重要的直觉

1. PDA 是地址，不是 Keypair，没有私钥。
2. 客户端负责计算 PDA，并把 PDA 账户传给链上程序。
3. 链上程序不能信客户端传来的 PDA，必须用同样 seeds 重新计算并比较。
4. PDA 账户由程序通过 `invoke_signed` 创建，不是客户端直接用 `SystemProgram.createAccount` 创建。
5. `VoteTopic PDA` 保存主题和票数。
6. `UserVote PDA` 保存某个用户对某个主题的投票记录。
7. 防重复投票的关键是 `UserVote PDA = derive(["user_vote", topic, voter])`，同一个 topic + voter 只能得到同一个 PDA。
8. 当前 `VoteTopic PDA = derive(["vote_topic", creator])`，所以一个 creator 只能有一个 topic；如果要多个主题，需要把额外 seed 加进去。
