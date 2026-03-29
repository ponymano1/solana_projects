# 第06节 - PDA基础：投票系统

本节学习PDA（Program Derived Address）的概念和使用，实现一个基于PDA的投票系统。

## 学习目标

- ✅ 理解PDA的概念和作用
- ✅ 掌握find_program_address的使用
- ✅ 理解bump seed的作用
- ✅ 学会使用invoke_signed创建PDA账户
- ✅ 掌握客户端如何计算和使用PDA

## 快速开始

```bash
cd 06-pda-basics
cargo build-bpf
cargo test
```

---

## 核心概念

### 1. 什么是PDA？

PDA（Program Derived Address）是由程序ID和种子（seeds）派生出的地址。

**关键特性：**
- 不在Ed25519曲线上
- 没有对应的私钥
- 只能由程序代表它签名
- 确定性：相同种子总是生成相同PDA

### 2. PDA派生过程

```
种子(seeds) + 程序ID + bump seed
         ↓
    SHA256哈希
         ↓
    检查是否在曲线上
         ↓
    在曲线上？ → bump--，重试
         ↓
    不在曲线上 → 找到PDA！
```

**代码示例：**
```rust
let (pda, bump) = Pubkey::find_program_address(
    &[b"vote_topic", creator.as_ref()],
    program_id,
);
```

### 3. PDA vs 普通地址

| 特性 | 普通地址 | PDA |
|------|---------|-----|
| 私钥 | 有 | 无 |
| 签名 | 用户签名 | 程序签名 |
| 创建 | 随机生成 | 确定性派生 |
| 用途 | 用户账户 | 程序管理的账户 |

### 4. 投票系统架构

```
┌─────────────────────────────────────┐
│  VoteTopic PDA                      │
│  seeds: ["vote_topic", creator]     │
│                                     │
│  creator: user_pubkey               │
│  description: "最喜欢的颜色？"       │
│  option_a_votes: 5                  │
│  option_b_votes: 3                  │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│  UserVote PDA                       │
│  seeds: ["user_vote", topic, voter] │
│                                     │
│  topic: topic_pda                   │
│  voter: user_pubkey                 │
│  vote_option: 0                     │
└─────────────────────────────────────┘
```

---

## 客户端调用

### 步骤1：计算PDA

```typescript
import { PublicKey } from '@solana/web3.js';

// 计算投票主题PDA
function findVoteTopicPDA(
  creator: PublicKey,
  programId: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from('vote_topic'),
      creator.toBuffer(),
    ],
    programId
  );
}

// 计算用户投票PDA
function findUserVotePDA(
  topicPDA: PublicKey,
  voter: PublicKey,
  programId: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from('user_vote'),
      topicPDA.toBuffer(),
      voter.toBuffer(),
    ],
    programId
  );
}

// 使用
const [topicPDA, topicBump] = findVoteTopicPDA(creator.publicKey, PROGRAM_ID);
const [userVotePDA, voteBump] = findUserVotePDA(topicPDA, voter.publicKey, PROGRAM_ID);
```

### 步骤2：创建投票主题

```typescript
// 序列化CreateTopic指令
function serializeCreateTopic(description: string, bump: number): Buffer {
  const descBytes = Buffer.from(description, 'utf-8');
  const buffer = Buffer.alloc(1 + 4 + descBytes.length + 1);
  
  let offset = 0;
  buffer[offset] = 0; // 指令索引
  offset += 1;
  
  buffer.writeUInt32LE(descBytes.length, offset); // description长度
  offset += 4;
  
  descBytes.copy(buffer, offset); // description
  offset += descBytes.length;
  
  buffer[offset] = bump; // bump seed
  
  return buffer;
}

const [topicPDA, bump] = findVoteTopicPDA(creator.publicKey, PROGRAM_ID);

const createTopicIx = new TransactionInstruction({
  keys: [
    { pubkey: creator.publicKey, isSigner: true, isWritable: true },
    { pubkey: topicPDA, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: serializeCreateTopic('最喜欢的颜色？', bump),
});
```

### 步骤3：投票

```typescript
// 序列化Vote指令
function serializeVote(option: number, bump: number): Buffer {
  const buffer = Buffer.alloc(3);
  buffer[0] = 1;      // 指令索引
  buffer[1] = option; // 投票选项 (0 或 1)
  buffer[2] = bump;   // bump seed
  return buffer;
}

const [topicPDA, _] = findVoteTopicPDA(creator.publicKey, PROGRAM_ID);
const [userVotePDA, voteBump] = findUserVotePDA(topicPDA, voter.publicKey, PROGRAM_ID);

const voteIx = new TransactionInstruction({
  keys: [
    { pubkey: voter.publicKey, isSigner: true, isWritable: true },
    { pubkey: topicPDA, isSigner: false, isWritable: true },
    { pubkey: userVotePDA, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: serializeVote(0, voteBump), // 投票给选项A
});
```

### 客户端与链上对应

| 客户端 | 链上 | 说明 |
|--------|------|------|
| `findVoteTopicPDA(...)` | `Pubkey::find_program_address(&[b"vote_topic", creator], program_id)` | 计算主题PDA |
| `findUserVotePDA(...)` | `Pubkey::find_program_address(&[b"user_vote", topic, voter], program_id)` | 计算投票PDA |
| `topicPDA` | `topic_account` | 投票主题账户 |
| `userVotePDA` | `user_vote_account` | 用户投票记录 |

---

## 常见问题

### Q: PDA和普通地址有什么区别？
A: PDA没有私钥，只能由程序代表它签名。普通地址有私钥，可以由持有私钥的人签名。

### Q: bump seed是什么？
A: bump seed是一个u8值（通常是255-252之间），用于确保派生的地址不在Ed25519曲线上。

### Q: 为什么要验证PDA地址？
A: 防止恶意用户传入错误的账户地址，确保程序操作的是正确的PDA账户。

### Q: 客户端如何知道bump seed？
A: 使用`findProgramAddressSync`计算PDA时会返回bump seed，然后传递给程序。

### Q: 为什么使用PDA而不是普通账户？
A:
- 不需要用户管理私钥
- 程序可以完全控制PDA账户
- 可以为每个用户/资源生成唯一地址
- 防止重复（如防止重复投票）

---

## 下一步

- 第07节：CPI基础
- 第08节：测试与调试

## 参考资料

- [Solana PDA文档](https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses)
