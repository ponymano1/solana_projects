# 第07节 - CPI基础：转账记录器

本节学习CPI（Cross-Program Invocation，跨程序调用），实现一个转账记录器程序。

## 学习目标

- ✅ 理解CPI的概念和作用
- ✅ 掌握invoke和invoke_signed的使用
- ✅ 学会调用System Program进行转账
- ✅ 理解PDA在CPI中的应用
- ✅ 掌握客户端如何构建CPI交易

## 快速开始

```bash
cd 07-cpi-basics
cargo build-bpf
cargo test
```

---

## 核心概念

### 1. 什么是CPI？

CPI（Cross-Program Invocation）允许一个程序调用另一个程序的指令。

**CPI流程：**
```
你的程序
    ↓ invoke()
System Program (转账)
    ↓
更新账户余额
```

### 2. invoke vs invoke_signed

| 函数 | 用途 | 签名者 |
|------|------|--------|
| `invoke` | 普通CPI调用 | 交易中的签名者 |
| `invoke_signed` | 使用PDA签名的CPI | PDA（程序代签） |

### 3. 转账记录器架构

```
┌─────────────────────────────────────┐
│  用户发起转账                        │
└──────┬──────────────────────────────┘
       │
       ↓
┌─────────────────────────────────────┐
│  转账记录器程序                      │
│  1. 调用System Program转账          │
│  2. 记录转账信息                     │
└──────┬──────────────────────────────┘
       │
       ↓
┌─────────────────────────────────────┐
│  TransferRecord账户                 │
│  from: user_pubkey                  │
│  to: recipient_pubkey               │
│  amount: 1000000                    │
│  timestamp: 12345                   │
└─────────────────────────────────────┘
```

---

## 程序功能

### 1. 普通转账（invoke）

```rust
invoke(
    &system_instruction::transfer(from, to, amount),
    &[from_account, to_account, system_program],
)?;
```

### 2. PDA转账（invoke_signed）

```rust
invoke_signed(
    &system_instruction::transfer(pda, to, amount),
    &[pda_account, to_account, system_program],
    &[&[b"transfer_vault", &[bump]]], // PDA签名种子
)?;
```

---

## 客户端调用

### 步骤1：普通转账并记录

```typescript
// 序列化TransferWithRecord指令
function serializeTransferWithRecord(amount: bigint): Buffer {
  const buffer = Buffer.alloc(9);
  buffer[0] = 0; // 指令索引
  buffer.writeBigUInt64LE(amount, 1); // amount
  return buffer;
}

const transferIx = new TransactionInstruction({
  keys: [
    { pubkey: from.publicKey, isSigner: true, isWritable: true },
    { pubkey: to.publicKey, isSigner: false, isWritable: true },
    { pubkey: recordAccount.publicKey, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: serializeTransferWithRecord(BigInt(1000000)),
});
```

### 步骤2：从PDA转账

```typescript
// 计算PDA
const [pdaAccount, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from('transfer_vault')],
  PROGRAM_ID
);

// 序列化TransferFromPDA指令
function serializeTransferFromPDA(amount: bigint, bump: number): Buffer {
  const buffer = Buffer.alloc(10);
  buffer[0] = 1; // 指令索引
  buffer.writeBigUInt64LE(amount, 1); // amount
  buffer[9] = bump; // bump seed
  return buffer;
}

const transferFromPDAIx = new TransactionInstruction({
  keys: [
    { pubkey: pdaAccount, isSigner: false, isWritable: true },
    { pubkey: to.publicKey, isSigner: false, isWritable: true },
    { pubkey: recordAccount.publicKey, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: serializeTransferFromPDA(BigInt(1000000), bump),
});
```

### 客户端与链上对应

| 操作 | 客户端 | 链上 |
|------|--------|------|
| 普通转账 | from需要签名 | invoke() |
| PDA转账 | PDA不需要签名 | invoke_signed() |
| 记录账户 | 不需要签名 | 写入转账记录 |

---

## 常见问题

### Q: 什么时候使用invoke vs invoke_signed？
A:
- `invoke`: 当签名者是交易中的普通账户时
- `invoke_signed`: 当签名者是PDA时

### Q: CPI有什么限制？
A:
- 最多4层调用深度
- 被调用程序必须是可执行的
- 账户必须正确传递

### Q: 为什么需要CPI？
A:
- 代码复用（调用已有程序）
- 组合性（程序间协作）
- 简化开发（不需要重复实现）

### Q: 如何调试CPI？
A:
- 使用`msg!`打印日志
- 检查账户权限
- 验证签名种子

---

## 下一步

- 第08节：测试与调试

## 参考资料

- [Solana CPI文档](https://docs.solana.com/developing/programming-model/calling-between-programs)
