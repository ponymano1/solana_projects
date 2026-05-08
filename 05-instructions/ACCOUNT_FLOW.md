# 第05节账户与交互图解：Todo 应用

这份文档专门解释第05节 Todo 程序的账户模型：用了哪些账户、账户里存哪些数据、客户端每个操作传了什么、链上如何校验权限。

先说结论：第05节当前代码的 `Initialize` 假设 `TodoList Account` 已经被创建好，并且 `Account.owner` 已经是当前程序。它只要求 `owner_account` 签名，不要求 `todo_list_account` 签名。

---

## 1. 第05节用了哪些账户

每次调用 Todo 程序，核心账户只有两个：

```text
┌──────────────────────────────┐
│ Owner Account                │
│ 用户钱包                      │
│                              │
│ key: owner_pubkey            │
│ signer: true                 │
│ 作用:                         │
│ - 初始化时成为 TodoList.owner  │
│ - 后续增删改 Todo 时做权限证明  │
│ - 支付交易费                  │
└──────────────────────────────┘

┌──────────────────────────────┐
│ TodoList Account             │
│ Todo 列表数据账户             │
│                              │
│ key: todo_list_pubkey        │
│ Account.owner: program_id    │
│ data: Borsh(TodoList)        │
│ 作用: 保存 todos 和 next_id    │
└──────────────────────────────┘
```

如果是真实客户端创建 `TodoList Account`，还会额外用到：

```text
┌──────────────────────────────┐
│ System Program               │
│ Solana 内置程序               │
│                              │
│ 作用: 创建 TodoList Account   │
│      分配空间                 │
│      设置 Account.owner       │
└──────────────────────────────┘
```

但注意：第05节当前链上 `Initialize` 指令本身没有接收 `System Program` 账户，也没有在程序内 CPI 创建账户。

---

## 2. 有几个 TodoList Account

一条指令只操作 1 个 `TodoList Account`：

```text
accounts[0] = owner_account
accounts[1] = todo_list_account
```

但同一个 Todo 程序可以管理很多个 TodoList 数据账户：

```text
Todo Program: program_id
┌────────────────────────────────────────────┐
│ 只保存程序代码，不保存某一个用户的 todos     │
└────────────────────────────────────────────┘
        │
        ├── todo_list_account_A
        │   ┌────────────────────────────────┐
        │   │ Account.owner: program_id      │
        │   │ data: TodoList {               │
        │   │   owner: Alice                 │
        │   │   todos: [...]                 │
        │   │ }                              │
        │   └────────────────────────────────┘
        │
        ├── todo_list_account_B
        │   ┌────────────────────────────────┐
        │   │ Account.owner: program_id      │
        │   │ data: TodoList {               │
        │   │   owner: Bob                   │
        │   │   todos: [...]                 │
        │   │ }                              │
        │   └────────────────────────────────┘
        │
        └── todo_list_account_C
            ┌────────────────────────────────┐
            │ Account.owner: program_id      │
            │ data: TodoList {               │
            │   owner: Alice                 │
            │   todos: [...]                 │
            │ }                              │
            └────────────────────────────────┘
```

程序操作哪一个列表，完全由客户端这次传入哪个 `todo_list_account` 决定。

---

## 3. TodoList Account 里有哪些数据

链上账户本身长这样：

```text
TodoList Account
┌────────────────────────────────────────────┐
│ lamports: rent_exempt_lamports             │
│ Account.owner: program_id                  │  Solana 账户层 owner
│ executable: false                          │
│ data: Borsh 序列化后的 TodoList             │
└────────────────────────────────────────────┘
```

`data` 里存的是：

```rust
pub struct TodoList {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub todos: Vec<TodoItem>,
    pub next_id: u32,
}
```

每个 Todo 项是：

```rust
pub struct TodoItem {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub completed: bool,
}
```

数据图：

```text
TodoList Account.data
┌────────────────────────────────────────────┐
│ TodoList {                                 │
│   is_initialized: true                     │
│   owner: owner_pubkey                      │  业务权限 owner
│   todos: [                                 │
│     TodoItem {                             │
│       id: 0                                │
│       title: "学习Solana"                  │
│       description: "完成第05节"             │
│       completed: false                     │
│     },                                     │
│     TodoItem {                             │
│       id: 1                                │
│       title: "写测试"                       │
│       description: "覆盖增删改"              │
│       completed: true                      │
│     }                                      │
│   ]                                        │
│   next_id: 2                               │
│ }                                          │
└────────────────────────────────────────────┘
```

这里也有两层 owner：

| 名称 | 在哪里 | 值 | 作用 |
|---|---|---|---|
| `Account.owner` | Solana 账户字段 | `program_id` | 决定哪个程序能写 `TodoList Account.data` |
| `TodoList.owner` | 业务数据字段 | `owner_pubkey` | 决定哪个用户能增删改 Todo |

---

## 4. 账户创建和 Initialize 的关系

第05节当前代码只实现了 `Initialize`，没有在链上程序里调用 System Program 创建账户。

所以真实客户端通常需要先创建账户，再初始化：

```text
Transaction
┌────────────────────────────────────────────┐
│ signer: payer / owner                      │
│ signer: todo_list_account                  │  普通 Keypair 新账户创建时需要
│                                            │
│ Instruction 1: SystemProgram.createAccount │
│ Instruction 2: TodoProgram.Initialize      │
└────────────────────────────────────────────┘
```

第一条指令创建空账户：

```text
SystemProgram.createAccount
┌────────────────────────────────────────────┐
│ fromPubkey: owner                          │  谁付款
│ newAccountPubkey: todo_list_account        │  新建哪个数据账户
│ lamports: rent_exempt_lamports             │  租金豁免余额
│ space: TodoList::max_space()               │  最大空间 2671 bytes
│ programId: Todo Program ID                 │  设置 Account.owner
└────────────────────────────────────────────┘
```

执行后：

```text
todo_list_account
┌────────────────────────────────────────────┐
│ lamports: rent_exempt_lamports             │
│ Account.owner: Todo Program ID             │
│ data: 2671 个 0                            │
└────────────────────────────────────────────┘
```

第二条指令初始化业务数据：

```text
TodoProgram.Initialize
┌────────────────────────────────────────────┐
│ accounts[0]: owner_account                 │ signer
│ accounts[1]: todo_list_account             │ writable
│ data: [0]                                  │ Initialize
└────────────────────────────────────────────┘
```

执行后：

```text
TodoList Account.data
┌────────────────────────────────────────────┐
│ TodoList {                                 │
│   is_initialized: true                     │
│   owner: owner_account.key                 │
│   todos: []                                │
│   next_id: 0                               │
│ }                                          │
└────────────────────────────────────────────┘
```

---

## 5. TypeScript 示例里容易出错的点

README 里的 TypeScript 示例是概念示例，实际项目里要注意下面几点。

### 5.1 `todoListAccount` 创建时签名，但 Initialize 指令里不标 signer

创建账户的交易需要：

```typescript
await sendAndConfirmTransaction(
  connection,
  transaction,
  [payer, todoListAccount]
);
```

这是因为 `SystemProgram.createAccount` 创建普通 Keypair 账户时要求新账户签名。

但第05节当前 `Initialize` 指令账户元信息是：

```typescript
const initializeIx = new TransactionInstruction({
  keys: [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: Buffer.from([0]),
});
```

这并不矛盾：

```text
同一个 transaction 的 signer 列表里有 todoListAccount，
是为了满足 SystemProgram.createAccount。

Initialize 指令自己的 accounts 里 todoListAccount isSigner=false，
是因为第05节链上 Initialize 没有检查 todo_list_account.is_signer。
```

如果你想让第05节和第04节完全一致，也可以修改链上代码，让 `Initialize` 也检查 `todo_list_account.is_signer`，但当前代码没有这么做。

### 5.2 README 的 `borsh.deserialize(schema, TodoList, data)` 可能和你安装的 borsh 版本不兼容

JavaScript/TypeScript 的 `borsh` 包不同版本 API 差异比较大。README 里的 schema 写法是老版本风格，有些版本会报错。

为了教程理解账户数据，建议先用手动反序列化理解布局，或者固定使用和示例匹配的 borsh 版本。

### 5.3 `space = 2671` 必须和链上最大空间一致

链上最大空间来自：

```rust
TodoList::max_space()
```

计算方式：

```text
1 + 32 + 4 + 10 * (4 + 4 + 50 + 4 + 200 + 1) + 4 = 2671
```

客户端创建账户时必须分配足够空间，否则后续 `todos.push(...)` 后序列化写回可能失败。

### 5.4 字符串长度限制按 UTF-8 字节数，不是字符数

链上检查：

```rust
if title.len() > MAX_TITLE_LEN { ... }
if description.len() > MAX_DESCRIPTION_LEN { ... }
```

Rust `String::len()` 返回 UTF-8 字节数。

所以中文字符通常占 3 个字节：

```text
"学习Solana" 的字节数 > 字符数
```

---

## 6. 指令一：Initialize

### 客户端传入

```text
program_id = Todo Program ID
accounts:
  [0] owner_account       signer, writable
  [1] todo_list_account   writable

data:
  [0]
```

TS 构造：

```typescript
const initializeIx = new TransactionInstruction({
  keys: [
    { pubkey: owner.publicKey, isSigner: true, isWritable: true },
    { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: Buffer.from([0]),
});
```

链上读取账户：

```rust
let owner_account = next_account_info(accounts_iter)?;
let todo_list_account = next_account_info(accounts_iter)?;
```

### 链上权限校验

```text
1. owner_account.is_signer == true
   目的：确认初始化人授权，并成为 TodoList.owner

2. todo_list_account.owner == program_id
   目的：确认这个数据账户归当前程序管理，程序可以写 data

3. todo_list.is_initialized == false
   目的：防止重复初始化覆盖已有 TodoList
```

### 链上写入数据

```text
TodoList {
  is_initialized: true,
  owner: owner_account.key,
  todos: [],
  next_id: 0,
}
```

### 账户变化

```text
初始化前
TodoList Account.data:
  可能是全 0，或反序列化失败后按未初始化处理

初始化后
TodoList Account.data:
  is_initialized = true
  owner = owner_account.key
  todos = []
  next_id = 0
```

---

## 7. 指令二：CreateTodo

### 客户端传入

```text
program_id = Todo Program ID
accounts:
  [0] owner_account       signer, writable
  [1] todo_list_account   writable

data:
  [1, title_len, title_bytes, description_len, description_bytes]
```

TS 序列化：

```typescript
function serializeString(value: string): Buffer {
  const bytes = Buffer.from(value, 'utf8');
  const len = Buffer.alloc(4);
  len.writeUInt32LE(bytes.length, 0);
  return Buffer.concat([len, bytes]);
}

function serializeCreateTodo(title: string, description: string): Buffer {
  return Buffer.concat([
    Buffer.from([1]),
    serializeString(title),
    serializeString(description),
  ]);
}
```

数据图：

```text
CreateTodo data
┌────┬──────────────┬─────────────┬──────────────────┬────────────────────┐
│ 01 │ title_len u32│ title bytes │ desc_len u32     │ desc bytes         │
└────┴──────────────┴─────────────┴──────────────────┴────────────────────┘
```

### 链上权限校验

```text
1. owner_account.is_signer == true
   目的：确认调用者控制这个钱包

2. todo_list_account.owner == program_id
   目的：确认 TodoList 数据账户归当前程序管理

3. todo_list.is_initialized == true
   目的：确认列表已经初始化

4. todo_list.owner == owner_account.key
   目的：确认签名者就是这个 TodoList 的业务 owner
```

### 参数校验

```text
1. title.len() <= MAX_TITLE_LEN，当前是 50 字节
2. description.len() <= MAX_DESCRIPTION_LEN，当前是 200 字节
3. todo_list.todos.len() < MAX_TODOS，当前最多 10 条
```

### 链上写入数据

```text
new_todo = TodoItem {
  id: todo_list.next_id,
  title,
  description,
  completed: false,
}

todo_list.todos.push(new_todo)
todo_list.next_id += 1
```

### 账户变化

```text
执行前
TodoList {
  todos: [],
  next_id: 0,
}

执行 CreateTodo("学习Solana", "完成第05节")

执行后
TodoList {
  todos: [
    TodoItem {
      id: 0,
      title: "学习Solana",
      description: "完成第05节",
      completed: false,
    }
  ],
  next_id: 1,
}
```

---

## 8. 指令三：UpdateTodo

### 客户端传入

```text
program_id = Todo Program ID
accounts:
  [0] owner_account       signer, writable
  [1] todo_list_account   writable

data:
  [2, id(u32), completed(bool)]
```

TS 序列化：

```typescript
function serializeUpdateTodo(id: number, completed: boolean): Buffer {
  const buffer = Buffer.alloc(6);
  buffer[0] = 2;
  buffer.writeUInt32LE(id, 1);
  buffer[5] = completed ? 1 : 0;
  return buffer;
}
```

数据图：

```text
UpdateTodo data
┌────┬────────────┬────────────┐
│ 02 │ id u32     │ completed  │
└────┴────────────┴────────────┘
```

### 链上权限校验

和 `CreateTodo` 一样：

```text
1. owner_account.is_signer == true
2. todo_list_account.owner == program_id
3. todo_list.is_initialized == true
4. todo_list.owner == owner_account.key
```

### 参数和业务校验

```text
查找 todo_list.todos 里 id == 参数 id 的 TodoItem
如果找不到，返回 TodoNotFound
```

### 链上写入数据

```text
todo.completed = completed
```

### 账户变化

```text
执行前
TodoItem { id: 0, completed: false }

执行 UpdateTodo { id: 0, completed: true }

执行后
TodoItem { id: 0, completed: true }
```

---

## 9. 指令四：DeleteTodo

### 客户端传入

```text
program_id = Todo Program ID
accounts:
  [0] owner_account       signer, writable
  [1] todo_list_account   writable

data:
  [3, id(u32)]
```

TS 序列化：

```typescript
function serializeDeleteTodo(id: number): Buffer {
  const buffer = Buffer.alloc(5);
  buffer[0] = 3;
  buffer.writeUInt32LE(id, 1);
  return buffer;
}
```

数据图：

```text
DeleteTodo data
┌────┬────────────┐
│ 03 │ id u32     │
└────┴────────────┘
```

### 链上权限校验

和 `CreateTodo` 一样：

```text
1. owner_account.is_signer == true
2. todo_list_account.owner == program_id
3. todo_list.is_initialized == true
4. todo_list.owner == owner_account.key
```

### 参数和业务校验

```text
查找 todo_list.todos 里 id == 参数 id 的 TodoItem
如果找不到，返回 TodoNotFound
```

### 链上写入数据

```text
todo_list.todos.remove(index)
```

注意：删除不会回退 `next_id`。

```text
执行前
TodoList {
  todos: [id=0, id=1],
  next_id: 2,
}

删除 id=0 后
TodoList {
  todos: [id=1],
  next_id: 2,
}

再创建一个新的 Todo
新 Todo 的 id 是 2，不是 0
```

---

## 10. 四个指令总表

| 指令 | accounts[0] | accounts[1] | data | 修改什么 |
|---|---|---|---|---|
| `Initialize` | owner signer | todo_list writable | `[0]` | 初始化整个 TodoList |
| `CreateTodo` | owner signer | todo_list writable | `[1, title, desc]` | push 新 Todo，next_id + 1 |
| `UpdateTodo` | owner signer | todo_list writable | `[2, id, completed]` | 修改某个 Todo.completed |
| `DeleteTodo` | owner signer | todo_list writable | `[3, id]` | 删除某个 Todo |

---

## 11. 权限校验总表

| 校验 | Initialize | CreateTodo | UpdateTodo | DeleteTodo | 目的 |
|---|---|---|---|---|---|
| `owner_account.is_signer` | 是 | 是 | 是 | 是 | 调用者必须签名 |
| `todo_list_account.owner == program_id` | 是 | 是 | 是 | 是 | 数据账户必须归当前程序管理 |
| `todo_list.is_initialized == false` | 是 | 否 | 否 | 否 | 防止重复初始化 |
| `todo_list.is_initialized == true` | 否 | 是 | 是 | 是 | 防止操作未初始化列表 |
| `todo_list.owner == owner_account.key` | 否 | 是 | 是 | 是 | 只有业务 owner 能增删改 |

---

## 12. 更接近真实项目的账户设计

第05节还没进入 PDA，所以它使用普通 Keypair 数据账户来教学。

真实项目里更常见的是 PDA：

```text
todo_list_pda = derive(program_id, ["todo_list", owner_pubkey])
```

PDA 没有私钥，所以不会 signer。

真实项目常见权限模型：

```text
InitializeTodoList:
  owner_account signer
  todo_list_pda writable, not signer
  system_program readonly

Create/Update/Delete:
  owner_account signer
  todo_list_pda writable, not signer
```

校验重点：

```text
1. owner_account.is_signer
2. todo_list_pda 地址是否由正确 seeds 派生
3. todo_list_account.owner == program_id
4. TodoList.owner == owner_account.key
```

这个模式会在 PDA 章节更自然。

---

## 13. 最重要的直觉

1. `TodoList Account` 是数据容器，不是程序。
2. Todo 数据存在 `TodoList Account.data` 里。
3. 客户端每次必须显式传入要操作的 `todo_list_account`。
4. 链上程序按 accounts 数组顺序读取账户。
5. `Account.owner == program_id` 决定程序能不能写这个账户的 data。
6. `TodoList.owner == owner_account.key` 决定哪个用户能增删改 Todo。
7. 第05节当前 `Initialize` 不要求 `todo_list_account` 签名，但真实创建普通 Keypair 账户时，`SystemProgram.createAccount` 会要求它签名。
8. README 里的 TS 反序列化代码要注意 borsh JS 版本差异，不能盲目照抄到所有版本。
