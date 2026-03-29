# 客户端调用指南

本文档详细说明如何从客户端（TypeScript/JavaScript）调用第04节的UserProfile程序。

## 目录

1. [环境准备](#环境准备)
2. [创建配置文件](#创建配置文件)
3. [更新配置文件](#更新配置文件)
4. [关闭配置文件](#关闭配置文件)
5. [数据序列化详解](#数据序列化详解)
6. [完整示例代码](#完整示例代码)

## 环境准备

### 安装依赖

```bash
npm install @solana/web3.js @solana/spl-token
```

### 导入必要的库

```typescript
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import * as borsh from 'borsh';
```

### 连接到Solana网络

```typescript
// 连接到本地测试网
const connection = new Connection('http://localhost:8899', 'confirmed');

// 或连接到devnet
// const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

// 加载付款人钱包
const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync('path/to/keypair.json', 'utf-8')))
);

// 你的程序ID（部署后获得）
const programId = new PublicKey('你的程序ID');
```

## 创建配置文件

### 步骤1：生成新账户

```typescript
// 生成新的配置文件账户
const profileAccount = Keypair.generate();
console.log('配置文件账户地址:', profileAccount.publicKey.toBase58());
```

### 步骤2：计算租金

```typescript
// UserProfile的大小：132字节
const PROFILE_SIZE = 132;

// 获取租金豁免所需的lamports
const rentExemptLamports = await connection.getMinimumBalanceForRentExemption(
  PROFILE_SIZE
);
console.log('租金:', rentExemptLamports / LAMPORTS_PER_SOL, 'SOL');
```

### 步骤3：构建创建账户指令

```typescript
// 指令1：调用System Program创建账户
const createAccountIx = SystemProgram.createAccount({
  fromPubkey: payer.publicKey,              // 付款人
  newAccountPubkey: profileAccount.publicKey, // 新账户地址
  lamports: rentExemptLamports,             // 租金
  space: PROFILE_SIZE,                      // 空间大小
  programId: programId,                     // 设置owner为你的程序
});
```

**这一步做了什么？**
- 从payer账户扣除租金
- 创建新账户，分配132字节空间
- 设置账户的owner为programId
- 此时账户的data是空的，需要下一步初始化

### 步骤4：序列化指令数据

```typescript
// 定义指令枚举
enum ProfileInstruction {
  CreateProfile = 0,
  UpdateProfile = 1,
  CloseProfile = 2,
}

// 序列化字符串的辅助函数
function serializeString(str: string): Buffer {
  const bytes = Buffer.from(str, 'utf-8');
  const length = Buffer.alloc(4);
  length.writeUInt32LE(bytes.length, 0);
  return Buffer.concat([length, bytes]);
}

// 构建CreateProfile指令数据
const name = "张三";
const age = 25;
const email = "zhangsan@example.com";

const instructionData = Buffer.concat([
  Buffer.from([ProfileInstruction.CreateProfile]), // 指令索引
  serializeString(name),                           // name
  Buffer.from([age]),                              // age
  serializeString(email),                          // email
]);
```

**数据格式对应：**

```
客户端发送:
[0, 12, 0, 0, 0, 229, 188, 160, 228, 184, 137, ..., 25, 24, 0, 0, 0, ...]
 ↑  ↑--------↑  ↑--------------------------↑      ↑   ↑--------↑
 指令 name长度   name的UTF-8字节              age  email长度

链上接收:
ProfileInstruction::CreateProfile {
    name: "张三",
    age: 25,
    email: "zhangsan@example.com",
}
```

### 步骤5：构建初始化指令

```typescript
// 指令2：调用你的程序初始化配置文件
const createProfileIx = new TransactionInstruction({
  keys: [
    // 账户列表，顺序必须与链上process_create_profile中的next_account_info顺序一致
    {
      pubkey: payer.publicKey,              // accounts[0]: payer
      isSigner: true,                       // 需要签名
      isWritable: true,                     // 需要写入（扣除租金）
    },
    {
      pubkey: profileAccount.publicKey,     // accounts[1]: profile_account
      isSigner: true,                       // 需要签名（新账户必须签名）
      isWritable: true,                     // 需要写入（初始化data）
    },
    {
      pubkey: SystemProgram.programId,      // accounts[2]: system_program
      isSigner: false,                      // 不需要签名
      isWritable: false,                    // 不需要写入
    },
  ],
  programId: programId,
  data: instructionData,
});
```

**accounts数组对应关系：**

| 客户端 keys[i] | 链上 accounts[i] | 说明 |
|----------------|------------------|------|
| `keys[0]` | `payer_info` | 付款人，需要签名和写入 |
| `keys[1]` | `profile_info` | 配置文件账户，需要签名和写入 |
| `keys[2]` | `system_program_info` | System Program，只读 |

### 步骤6：发送交易

```typescript
// 创建交易，包含两个指令
const transaction = new Transaction().add(createAccountIx, createProfileIx);

// 发送并确认交易
const signature = await sendAndConfirmTransaction(
  connection,
  transaction,
  [payer, profileAccount], // 签名者：payer和profileAccount
  {
    commitment: 'confirmed',
  }
);

console.log('交易签名:', signature);
console.log('配置文件创建成功！');
```

**签名者说明：**
- `payer`：付款人，需要签名授权扣款
- `profileAccount`：新账户，需要签名证明拥有私钥

### 步骤7：验证结果

```typescript
// 读取账户数据
const accountInfo = await connection.getAccountInfo(profileAccount.publicKey);

if (accountInfo) {
  console.log('账户owner:', accountInfo.owner.toBase58());
  console.log('账户余额:', accountInfo.lamports / LAMPORTS_PER_SOL, 'SOL');
  console.log('账户大小:', accountInfo.data.length, '字节');

  // 反序列化数据（需要实现Borsh反序列化）
  const profile = deserializeProfile(accountInfo.data);
  console.log('配置文件:', profile);
}
```

## 更新配置文件

### 构建更新指令

```typescript
// 更新指令数据格式
// Option<T>: 1字节标志 + 数据
function serializeOption<T>(value: T | null, serializer: (v: T) => Buffer): Buffer {
  if (value === null) {
    return Buffer.from([0]); // None
  } else {
    return Buffer.concat([Buffer.from([1]), serializer(value)]); // Some(value)
  }
}

// 构建UpdateProfile指令数据
const updateData = Buffer.concat([
  Buffer.from([ProfileInstruction.UpdateProfile]), // 指令索引
  serializeOption("李四", serializeString),         // name: Some("李四")
  serializeOption(26, (age) => Buffer.from([age])), // age: Some(26)
  serializeOption(null, serializeString),           // email: None（不更新）
]);

// 构建更新指令
const updateProfileIx = new TransactionInstruction({
  keys: [
    {
      pubkey: payer.publicKey,              // accounts[0]: owner（数据owner）
      isSigner: true,                       // 需要签名
      isWritable: true,                     // 可能需要支付交易费
    },
    {
      pubkey: profileAccount.publicKey,     // accounts[1]: profile_account
      isSigner: false,                      // 不需要签名（验证的是数据owner）
      isWritable: true,                     // 需要写入（更新data）
    },
  ],
  programId: programId,
  data: updateData,
});

// 发送交易
const updateTx = new Transaction().add(updateProfileIx);
const updateSig = await sendAndConfirmTransaction(
  connection,
  updateTx,
  [payer], // 只需要payer签名（数据owner）
);

console.log('更新成功:', updateSig);
```

**注意差异：**
- 创建时：profile账户需要签名（`isSigner: true`）
- 更新时：profile账户不需要签名（`isSigner: false`）
- 原因：更新时验证的是数据owner（payer），不是账户owner

## 关闭配置文件

### 构建关闭指令

```typescript
// 构建CloseProfile指令数据
const closeData = Buffer.from([ProfileInstruction.CloseProfile]);

// 构建关闭指令
const closeProfileIx = new TransactionInstruction({
  keys: [
    {
      pubkey: payer.publicKey,              // accounts[0]: owner（数据owner）
      isSigner: true,                       // 需要签名
      isWritable: true,                     // 需要写入（接收返还的租金）
    },
    {
      pubkey: profileAccount.publicKey,     // accounts[1]: profile_account
      isSigner: false,                      // 不需要签名
      isWritable: true,                     // 需要写入（清空账户）
    },
  ],
  programId: programId,
  data: closeData,
});

// 发送交易
const closeTx = new Transaction().add(closeProfileIx);
const closeSig = await sendAndConfirmTransaction(
  connection,
  closeTx,
  [payer],
);

console.log('关闭成功，租金已返还:', closeSig);

// 验证账户已关闭
const closedAccount = await connection.getAccountInfo(profileAccount.publicKey);
console.log('账户是否存在:', closedAccount !== null); // 应该是false
```

## 数据序列化详解

### Borsh序列化规则

Borsh是Solana使用的二进制序列化格式。

**基本类型：**

| Rust类型 | 字节数 | TypeScript序列化 |
|----------|--------|------------------|
| `bool` | 1 | `Buffer.from([value ? 1 : 0])` |
| `u8` | 1 | `Buffer.from([value])` |
| `u32` | 4 | `buffer.writeUInt32LE(value, 0)` |
| `u64` | 8 | `buffer.writeBigUInt64LE(BigInt(value), 0)` |
| `Pubkey` | 32 | `publicKey.toBuffer()` |

**复合类型：**

```typescript
// String: 长度(u32) + UTF-8字节
function serializeString(str: string): Buffer {
  const bytes = Buffer.from(str, 'utf-8');
  const length = Buffer.alloc(4);
  length.writeUInt32LE(bytes.length, 0);
  return Buffer.concat([length, bytes]);
}

// Option<T>: 标志(u8) + 数据
// None: [0]
// Some(value): [1, ...serialize(value)]
function serializeOption<T>(
  value: T | null,
  serializer: (v: T) => Buffer
): Buffer {
  if (value === null) {
    return Buffer.from([0]);
  } else {
    return Buffer.concat([Buffer.from([1]), serializer(value)]);
  }
}

// Vec<T>: 长度(u32) + 元素
function serializeVec<T>(
  items: T[],
  serializer: (v: T) => Buffer
): Buffer {
  const length = Buffer.alloc(4);
  length.writeUInt32LE(items.length, 0);
  const elements = items.map(serializer);
  return Buffer.concat([length, ...elements]);
}
```

### 反序列化示例

```typescript
// 反序列化UserProfile
function deserializeProfile(data: Buffer): any {
  let offset = 0;

  // is_initialized: bool (1字节)
  const isInitialized = data[offset] !== 0;
  offset += 1;

  // owner: Pubkey (32字节)
  const owner = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;

  // name: [u8; 32] (32字节)
  const nameBytes = data.slice(offset, offset + 32);
  offset += 32;

  // name_len: u8 (1字节)
  const nameLen = data[offset];
  offset += 1;

  // age: u8 (1字节)
  const age = data[offset];
  offset += 1;

  // email: [u8; 64] (64字节)
  const emailBytes = data.slice(offset, offset + 64);
  offset += 64;

  // email_len: u8 (1字节)
  const emailLen = data[offset];
  offset += 1;

  // 转换为字符串
  const name = Buffer.from(nameBytes.slice(0, nameLen)).toString('utf-8');
  const email = Buffer.from(emailBytes.slice(0, emailLen)).toString('utf-8');

  return {
    isInitialized,
    owner: owner.toBase58(),
    name,
    age,
    email,
  };
}
```

## 完整示例代码

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
import * as fs from 'fs';

// 配置
const PROGRAM_ID = new PublicKey('你的程序ID');
const PROFILE_SIZE = 132;

// 指令枚举
enum ProfileInstruction {
  CreateProfile = 0,
  UpdateProfile = 1,
  CloseProfile = 2,
}

// 序列化辅助函数
function serializeString(str: string): Buffer {
  const bytes = Buffer.from(str, 'utf-8');
  const length = Buffer.alloc(4);
  length.writeUInt32LE(bytes.length, 0);
  return Buffer.concat([length, bytes]);
}

function serializeOption<T>(
  value: T | null,
  serializer: (v: T) => Buffer
): Buffer {
  if (value === null) {
    return Buffer.from([0]);
  } else {
    return Buffer.concat([Buffer.from([1]), serializer(value)]);
  }
}

// 主函数
async function main() {
  // 连接到本地测试网
  const connection = new Connection('http://localhost:8899', 'confirmed');

  // 加载付款人
  const payer = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync('keypair.json', 'utf-8')))
  );

  console.log('付款人地址:', payer.publicKey.toBase58());

  // 1. 创建配置文件
  console.log('\n=== 创建配置文件 ===');
  const profileAccount = Keypair.generate();
  const rentExemptLamports = await connection.getMinimumBalanceForRentExemption(
    PROFILE_SIZE
  );

  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: profileAccount.publicKey,
    lamports: rentExemptLamports,
    space: PROFILE_SIZE,
    programId: PROGRAM_ID,
  });

  const createData = Buffer.concat([
    Buffer.from([ProfileInstruction.CreateProfile]),
    serializeString("张三"),
    Buffer.from([25]),
    serializeString("zhangsan@example.com"),
  ]);

  const createProfileIx = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: profileAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: createData,
  });

  const createTx = new Transaction().add(createAccountIx, createProfileIx);
  const createSig = await sendAndConfirmTransaction(
    connection,
    createTx,
    [payer, profileAccount]
  );

  console.log('创建成功:', createSig);
  console.log('配置文件地址:', profileAccount.publicKey.toBase58());

  // 2. 更新配置文件
  console.log('\n=== 更新配置文件 ===');
  const updateData = Buffer.concat([
    Buffer.from([ProfileInstruction.UpdateProfile]),
    serializeOption("李四", serializeString),
    serializeOption(26, (age) => Buffer.from([age])),
    serializeOption(null, serializeString),
  ]);

  const updateProfileIx = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: profileAccount.publicKey, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data: updateData,
  });

  const updateTx = new Transaction().add(updateProfileIx);
  const updateSig = await sendAndConfirmTransaction(
    connection,
    updateTx,
    [payer]
  );

  console.log('更新成功:', updateSig);

  // 3. 关闭配置文件
  console.log('\n=== 关闭配置文件 ===');
  const closeData = Buffer.from([ProfileInstruction.CloseProfile]);

  const closeProfileIx = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: profileAccount.publicKey, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data: closeData,
  });

  const closeTx = new Transaction().add(closeProfileIx);
  const closeSig = await sendAndConfirmTransaction(
    connection,
    closeTx,
    [payer]
  );

  console.log('关闭成功:', closeSig);
}

main().catch(console.error);
```

## 常见错误

### 1. 账户顺序错误

```
错误: Program failed: incorrect program id for instruction
原因: keys数组的顺序与链上next_account_info的顺序不一致
解决: 检查keys数组顺序，确保与链上代码一致
```

### 2. 签名者缺失

```
错误: Transaction signature verification failure
原因: 需要签名的账户没有在sendAndConfirmTransaction的签名者数组中
解决: 确保所有isSigner: true的账户都在签名者数组中
```

### 3. 数据序列化错误

```
错误: Program failed: invalid instruction data
原因: 指令数据格式与链上期望的不一致
解决: 检查Borsh序列化格式，确保与链上定义一致
```

### 4. 权限不足

```
错误: Program failed: illegal owner
原因: 调用者不是数据owner
解决: 确保使用正确的owner账户签名
```

## 调试技巧

### 1. 查看交易日志

```typescript
const signature = await sendAndConfirmTransaction(...);
const txDetails = await connection.getTransaction(signature, {
  commitment: 'confirmed',
});
console.log('交易日志:', txDetails?.meta?.logMessages);
```

### 2. 模拟交易

```typescript
const simulation = await connection.simulateTransaction(transaction);
console.log('模拟结果:', simulation);
```

### 3. 检查账户状态

```typescript
const accountInfo = await connection.getAccountInfo(profileAccount.publicKey);
console.log('账户owner:', accountInfo?.owner.toBase58());
console.log('账户数据长度:', accountInfo?.data.length);
console.log('账户余额:', accountInfo?.lamports);
```

## 总结

客户端调用Solana程序的关键点：

1. **账户顺序**：keys数组必须与链上next_account_info顺序一致
2. **签名者**：isSigner: true的账户必须在签名者数组中
3. **数据序列化**：使用Borsh格式，与链上定义保持一致
4. **权限验证**：理解两层owner概念，使用正确的账户签名
5. **错误处理**：查看交易日志，使用模拟交易调试

掌握这些要点，你就能顺利地从客户端调用Solana程序了！
