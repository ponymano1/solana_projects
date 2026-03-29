# 第05节 - 指令处理：Todo应用

本节实现一个Todo应用，学习复杂指令结构、参数处理、自定义错误和权限控制。

## 目录

- [学习目标](#学习目标)
- [快速开始](#快速开始)
- [核心概念](#核心概念)
- [数据结构详解](#数据结构详解)
- [指令处理流程](#指令处理流程)
- [客户端调用](#客户端调用)
- [常见问题](#常见问题)

---

## 学习目标

- ✅ 设计复杂的指令结构（带参数）
- ✅ 实现指令路由和分发
- ✅ 掌握自定义错误类型的使用
- ✅ 理解权限控制和访问验证
- ✅ 学会处理动态数据（Vec）

## 前置知识

- 完成第01-04节
- 理解账户模型、Borsh序列化和基本程序结构

## 快速开始

```bash
cd 05-instructions
cargo build-bpf
cargo test
```

### 测试包括

- `test_initialize_todo_list` - 初始化Todo列表
- `test_create_todo` - 创建Todo
- `test_update_todo` - 更新Todo状态
- `test_delete_todo` - 删除Todo
- `test_unauthorized_access` - 测试权限控制

---

## 核心概念

### 1. 复杂指令结构

与第03节的简单枚举不同，本节的指令携带参数：

```rust
pub enum TodoInstruction {
    Initialize,                                    // 无参数
    CreateTodo { title: String, description: String },  // 带参数
    UpdateTodo { id: u32, completed: bool },       // 带参数
    DeleteTodo { id: u32 },                        // 带参数
}
```

**Borsh序列化后：**
```
Initialize → [0]

CreateTodo → [1, title_len(4字节), title_bytes..., desc_len(4字节), desc_bytes...]

UpdateTodo → [2, id(4字节), completed(1字节)]

DeleteTodo → [3, id(4字节)]
```

### 2. 数据存储位置

```
┌─────────────────────────────────────────────┐
│    TodoList Account（Todo列表账户）          │
│                                             │
│  owner: program_id                          │
│  data: [序列化的TodoList]                    │
│                                             │
│  TodoList {                                 │
│    is_initialized: true,                    │
│    owner: user_pubkey,  ← 数据层面的owner   │
│    todos: Vec<TodoItem>,                    │
│    next_id: 3,                              │
│  }                                          │
└─────────────────────────────────────────────┘
```

### 3. 自定义错误类型

使用`thiserror`库定义清晰的错误：

```rust
#[derive(Error, Debug, Copy, Clone)]
pub enum TodoError {
    #[error("账户已初始化")]
    AlreadyInitialized,

    #[error("账户未初始化")]
    UninitializedAccount,

    #[error("权限不足")]
    Unauthorized,

    #[error("标题过长")]
    TitleTooLong,

    #[error("Todo列表已满")]
    TodoListFull,

    #[error("Todo未找到")]
    TodoNotFound,
}
```

**转换为ProgramError：**
```rust
impl From<TodoError> for ProgramError {
    fn from(e: TodoError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
```

---

## 数据结构详解

### TodoItem结构

```rust
pub struct TodoItem {
    pub id: u32,           // 4字节
    pub title: String,     // 4字节(长度) + 实际字节
    pub description: String, // 4字节(长度) + 实际字节
    pub completed: bool,   // 1字节
}
```

**Borsh序列化示例：**
```
TodoItem {
    id: 1,
    title: "学习Solana",
    description: "完成第05节",
    completed: false,
}

序列化为：
[
    01, 00, 00, 00,              // id = 1
    0F, 00, 00, 00,              // title长度 = 15字节
    E5, AD, A6, E4, B9, A0, ...  // "学习Solana" UTF-8编码
    12, 00, 00, 00,              // description长度 = 18字节
    E5, AE, 8C, E6, 88, 90, ...  // "完成第05节" UTF-8编码
    00                           // completed = false
]
```

### TodoList结构

```rust
pub struct TodoList {
    pub is_initialized: bool,  // 1字节
    pub owner: Pubkey,         // 32字节
    pub todos: Vec<TodoItem>,  // 4字节(长度) + 元素
    pub next_id: u32,          // 4字节
}
```

**空间计算：**
```
最小空间（空列表）：
1 + 32 + 4 + 4 = 41字节

最大空间（10个Todo，每个最大长度）：
1 + 32 + 4 + 4 + 10 * (4 + 4 + 50 + 4 + 200 + 1) = 2671字节
```

---

## 指令处理流程

### 创建Todo流程

```
客户端
  ↓
发送CreateTodo指令
  data: [1, title_len, title_bytes, desc_len, desc_bytes]
  ↓
process_instruction
  ↓
反序列化为TodoInstruction::CreateTodo { title, description }
  ↓
process_create_todo
  ↓
1. 验证签名者
   if !owner_account.is_signer { return Err(...) }
  ↓
2. 验证账户owner
   if todo_list_account.owner != program_id { return Err(...) }
  ↓
3. 反序列化TodoList
   let mut todo_list = TodoList::try_from_slice(&data)?
  ↓
4. 验证初始化状态
   if !todo_list.is_initialized { return Err(...) }
  ↓
5. 验证权限（数据层面的owner）
   if todo_list.owner != *owner_account.key { return Err(...) }
  ↓
6. 验证参数
   if title.len() > MAX_TITLE_LEN { return Err(...) }
  ↓
7. 检查容量
   if todo_list.todos.len() >= MAX_TODOS { return Err(...) }
  ↓
8. 创建TodoItem
   let new_todo = TodoItem { id: todo_list.next_id, ... }
  ↓
9. 添加到列表
   todo_list.todos.push(new_todo)
   todo_list.next_id += 1
  ↓
10. 序列化并写入
    todo_list.serialize(&mut account.data)?
  ↓
返回成功
```

---

## 客户端调用

### 完整流程示例

#### 步骤1：创建Todo列表账户

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
import * as borsh from 'borsh';

const PROGRAM_ID = new PublicKey('你的程序ID');
const connection = new Connection('http://localhost:8899', 'confirmed');
const payer = Keypair.fromSecretKey(/* 你的密钥 */);

// 1. 生成Todo列表账户
const todoListAccount = Keypair.generate();

// 2. 计算空间和租金
const space = 2671; // 最大空间
const rentExemptLamports = await connection.getMinimumBalanceForRentExemption(space);

// 3. 创建账户
const createAccountIx = SystemProgram.createAccount({
  fromPubkey: payer.publicKey,
  newAccountPubkey: todoListAccount.publicKey,
  lamports: rentExemptLamports,
  space: space,
  programId: PROGRAM_ID,
});
```

#### 步骤2：初始化Todo列表

```typescript
// 构建Initialize指令
const initializeIx = new TransactionInstruction({
  keys: [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: Buffer.from([0]), // Initialize指令索引
});

// 发送交易
const transaction = new Transaction().add(createAccountIx, initializeIx);
await sendAndConfirmTransaction(
  connection,
  transaction,
  [payer, todoListAccount]
);

console.log('✓ Todo列表已初始化');
```

#### 步骤3：创建Todo

```typescript
// 序列化CreateTodo指令
function serializeCreateTodo(title: string, description: string): Buffer {
  // 指令索引
  const instructionIndex = Buffer.from([1]);

  // 序列化title
  const titleBytes = Buffer.from(title, 'utf-8');
  const titleLen = Buffer.alloc(4);
  titleLen.writeUInt32LE(titleBytes.length, 0);

  // 序列化description
  const descBytes = Buffer.from(description, 'utf-8');
  const descLen = Buffer.alloc(4);
  descLen.writeUInt32LE(descBytes.length, 0);

  return Buffer.concat([
    instructionIndex,
    titleLen,
    titleBytes,
    descLen,
    descBytes,
  ]);
}

const createTodoIx = new TransactionInstruction({
  keys: [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: serializeCreateTodo('学习Solana', '完成第05节'),
});

await sendAndConfirmTransaction(
  connection,
  new Transaction().add(createTodoIx),
  [payer]
);

console.log('✓ Todo已创建');
```

#### 步骤4：更新Todo

```typescript
// 序列化UpdateTodo指令
function serializeUpdateTodo(id: number, completed: boolean): Buffer {
  const buffer = Buffer.alloc(6);
  buffer[0] = 2; // UpdateTodo指令索引
  buffer.writeUInt32LE(id, 1); // id
  buffer[5] = completed ? 1 : 0; // completed
  return buffer;
}

const updateTodoIx = new TransactionInstruction({
  keys: [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: serializeUpdateTodo(0, true), // 标记第一个Todo为完成
});

await sendAndConfirmTransaction(
  connection,
  new Transaction().add(updateTodoIx),
  [payer]
);

console.log('✓ Todo已更新');
```

#### 步骤5：读取Todo列表

```typescript
// 定义数据结构
class TodoItem {
  id: number;
  title: string;
  description: string;
  completed: boolean;

  constructor(fields: any) {
    this.id = fields.id;
    this.title = fields.title;
    this.description = fields.description;
    this.completed = fields.completed;
  }
}

class TodoList {
  is_initialized: boolean;
  owner: Uint8Array;
  todos: TodoItem[];
  next_id: number;

  constructor(fields: any) {
    this.is_initialized = fields.is_initialized;
    this.owner = fields.owner;
    this.todos = fields.todos.map((t: any) => new TodoItem(t));
    this.next_id = fields.next_id;
  }
}

// Borsh schema
const schema = new Map([
  [TodoItem, {
    kind: 'struct',
    fields: [
      ['id', 'u32'],
      ['title', 'string'],
      ['description', 'string'],
      ['completed', 'bool'],
    ],
  }],
  [TodoList, {
    kind: 'struct',
    fields: [
      ['is_initialized', 'bool'],
      ['owner', [32]],
      ['todos', [TodoItem]],
      ['next_id', 'u32'],
    ],
  }],
]);

// 读取并反序列化
const accountInfo = await connection.getAccountInfo(todoListAccount.publicKey);
const todoList = borsh.deserialize(schema, TodoList, accountInfo.data);

console.log('Todo列表:');
todoList.todos.forEach(todo => {
  console.log(`  [${todo.completed ? '✓' : ' '}] ${todo.title}: ${todo.description}`);
});
```

### 客户端与链上对应关系

#### 指令数据对应

| 客户端 | 链上 | 数据格式 |
|--------|------|----------|
| `Buffer.from([0])` | `TodoInstruction::Initialize` | `[0]` |
| `serializeCreateTodo(...)` | `TodoInstruction::CreateTodo{...}` | `[1, title_len, title, desc_len, desc]` |
| `serializeUpdateTodo(...)` | `TodoInstruction::UpdateTodo{...}` | `[2, id, completed]` |
| `serializeDeleteTodo(...)` | `TodoInstruction::DeleteTodo{...}` | `[3, id]` |

#### 账户数组对应

| 客户端 keys[i] | 链上 accounts[i] | 说明 |
|----------------|------------------|------|
| `keys[0]` | `owner_account` | 所有者，需要签名 |
| `keys[1]` | `todo_list_account` | Todo列表账户，需要写入 |

**注意：**
- Initialize时owner需要签名（`isSigner: true`）
- 其他操作时owner也需要签名（权限验证）
- todo_list_account不需要签名（`isSigner: false`）

---

## 常见问题

### Q: 为什么需要自定义错误类型？
A:
- 提供清晰的错误信息，便于调试
- 类型安全，编译时检查
- 更好的用户体验
- 便于错误处理和日志记录

### Q: 如何设计指令结构？
A:
- 每个指令应该有明确的单一职责
- 使用枚举的变体来携带不同的参数
- 参数应该是必需的，避免使用Option
- 考虑指令的组合和顺序依赖

### Q: 为什么要限制Todo列表的大小？
A:
- 账户空间是有限的，需要预先分配
- 防止无限增长导致的性能问题
- 简化租金计算
- 这是Solana程序的常见模式

### Q: 如何处理Vec的序列化？
A:
- Borsh自动处理Vec的序列化
- 先序列化Vec的长度（4字节）
- 然后序列化每个元素
- 反序列化时按相同顺序读取

### Q: 权限检查应该放在哪里？
A:
- 在每个需要权限的操作开始时进行
- 在修改数据之前完成
- 使用`is_signer`检查签名
- 使用`owner`字段检查所有权

### Q: String如何序列化？
A:
- Borsh将String序列化为：长度(u32) + UTF-8字节
- 例如："Hello" → `[05, 00, 00, 00, 48, 65, 6C, 6C, 6F]`

---

## 最佳实践

### 1. 参数验证

```rust
// ✅ 正确：验证所有参数
if title.len() > MAX_TITLE_LEN {
    return Err(TodoError::TitleTooLong.into());
}
if description.len() > MAX_DESCRIPTION_LEN {
    return Err(TodoError::DescriptionTooLong.into());
}

// ❌ 错误：不验证参数
let new_todo = TodoItem { title, description, ... };
```

### 2. 权限控制

```rust
// ✅ 正确：验证签名和所有权
if !owner_account.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
}
if todo_list.owner != *owner_account.key {
    return Err(TodoError::Unauthorized.into());
}

// ❌ 错误：不验证权限
todo_list.todos.push(new_todo);
```

### 3. 状态检查

```rust
// ✅ 正确：检查初始化状态
if !todo_list.is_initialized {
    return Err(TodoError::UninitializedAccount.into());
}

// ❌ 错误：不检查状态
let mut todo_list = TodoList::try_from_slice(&data)?;
```

---

## 项目结构

```
05-instructions/
├── src/
│   ├── lib.rs          # 程序入口点，指令分发
│   ├── instruction.rs  # 指令枚举定义
│   ├── state.rs        # 数据结构
│   └── error.rs        # 自定义错误类型
└── tests/
    └── integration.rs  # 集成测试
```

---

## 下一步

完成本节后，继续学习：
- **第06节：PDA基础** - 学习程序派生地址
- **第07节：CPI基础** - 学习跨程序调用
- **第08节：测试与调试** - 学习测试框架

---

## 参考资料

- [Solana程序开发文档](https://docs.solana.com/developing/on-chain-programs/overview)
- [Borsh序列化](https://borsh.io/)
- [thiserror库文档](https://docs.rs/thiserror/)
