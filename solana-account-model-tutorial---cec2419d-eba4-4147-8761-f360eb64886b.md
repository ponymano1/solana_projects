# Solana 账户模型教程

> 一份面向入门开发者的 Solana Account Model 教程。理解账户模型，是理解 Anchor、PDA、SPL Token、CPI 和 Solana 程序开发的基础。

---

## 1. Solana 的账户是什么？

在 Solana 里，最重要的基础概念之一是：**所有东西几乎都是账户**。

钱包地址是账户，程序是账户，Token 余额是账户，NFT 元数据是账户，程序状态也是账户。

一个 Solana 账户大致包含这些字段：

```text
Account {
  address: PublicKey,
  lamports: u64,
  data: Vec<u8>,
  owner: PublicKey,
  executable: bool,
}
```

其中最关键的是：

- `lamports`：账户里的 SOL 数量
- `data`：账户保存的数据
- `owner`：哪个程序有权限修改这个账户的数据
- `executable`：这个账户是否是可执行程序

可以把账户理解成链上的一块数据空间。

---

## 2. Solana 和以太坊账户模型的区别

以太坊里，智能合约通常自己拥有状态。

例如：

```text
合约地址
  ├── balance
  ├── mapping
  ├── storage
  └── code
```

合约代码和合约状态是强绑定的。

但 Solana 不一样。

Solana 通常是这样：

```text
程序账户：只存代码
数据账户：存状态
用户账户：存 SOL
Token 账户：存 Token 余额
```

程序本身通常不直接保存业务状态。业务状态会被放在单独的数据账户里，由程序读取和修改。

这就是 Solana 账户模型最核心的思想：

> **程序和状态是分离的。**

---

## 3. Account Owner：谁能修改账户数据？

每个 Solana 账户都有一个 `owner`。

这里的 owner 不是“这个账户属于哪个用户”，而是：

> **哪个程序有权修改这个账户的数据。**

例如：

```text
普通用户钱包账户
owner = System Program

Token Account
owner = SPL Token Program

Anchor 程序创建的数据账户
owner = 你的程序 ID
```

核心规则是：

> **只有账户的 owner 程序，才能修改该账户的 data。**

比如一个 Token Account 的 owner 是 SPL Token Program，那么你的程序不能直接修改它的 token 余额。

你必须通过 CPI 调用 SPL Token Program，让 SPL Token Program 自己去修改。

这是 Solana 安全模型的核心之一。

---

## 4. 钱包地址也是账户

当我们说“用户的钱包地址”，其实也是一个 Solana 账户。

例如：

```text
Alice: 7xKX...abc
```

这个账户通常由 System Program 管理。

```text
Alice Account {
  lamports: 10 SOL,
  data: [],
  owner: System Program,
  executable: false
}
```

普通钱包账户主要功能是：

- 持有 SOL
- 签名交易
- 支付交易费
- 创建其他账户

---

## 5. 程序也是账户

Solana 上的智能合约叫 Program。

Program 本质上也是一个账户，只不过它的 `executable` 字段为 `true`。

```text
Program Account {
  lamports: ...,
  data: compiled program code,
  owner: BPF Loader,
  executable: true
}
```

程序账户保存的是编译后的代码。

但程序账户通常不保存业务状态。

例如一个计数器程序：

```text
Counter Program
```

它的代码逻辑可能是：

```text
读取某个 Counter Account
把 count + 1
写回 Counter Account
```

状态不是存在 Program Account 里面，而是存在 Counter Account 里面。

---

## 6. 数据账户：Solana 程序的状态存在哪里？

假设我们要做一个简单的计数器程序。

在以太坊里，你可能会写：

```solidity
uint256 public count;
```

`count` 存在合约自己的 storage 里面。

但在 Solana 里，我们通常会创建一个单独的账户：

```text
Counter Account {
  count: 0
}
```

这个账户的 owner 是你的程序：

```text
owner = Counter Program ID
```

结构大概是：

```text
Counter Program Account
  - 保存程序代码

Counter Data Account
  - 保存 count 状态
  - owner = Counter Program
```

执行 `increment` 的时候，交易会把 Counter Data Account 传给程序。

程序检查账户权限，然后修改里面的数据。

---

## 7. 为什么交易需要显式传入账户？

Solana 程序执行时，不能像以太坊那样随便访问全局状态。

你必须在交易里明确告诉程序：

> 这次指令要读取或修改哪些账户。

例如调用 `increment`：

```text
Instruction {
  program_id: CounterProgram,
  accounts: [
    CounterAccount,
    UserSigner
  ],
  data: "increment"
}
```

这样做的好处是：

- Solana runtime 可以提前知道哪些账户会被读写
- 不冲突的交易可以并行执行
- 这是 Solana 高性能的重要原因之一

例如：

```text
交易 A 修改 CounterAccount1
交易 B 修改 CounterAccount2
```

这两个交易可以并行执行。

但如果两个交易都要修改同一个 CounterAccount，就必须排队。

---

## 8. 账户权限：signer 和 writable

在 Solana 指令里，每个账户还会标记权限：

```text
is_signer
is_writable
```

### is_signer

表示这个账户是否必须签名。

比如创建账户、转账、授权操作通常需要用户签名。

### is_writable

表示这个账户是否会被修改。

如果程序要修改账户的 lamports 或 data，这个账户必须被标记为 writable。

例如：

```text
accounts: [
  {
    pubkey: user,
    is_signer: true,
    is_writable: true
  },
  {
    pubkey: counter_account,
    is_signer: false,
    is_writable: true
  }
]
```

如果一个账户没有标记 writable，程序尝试修改它会失败。

---

## 9. Rent：账户为什么需要 SOL？

Solana 账户需要占用链上存储空间，所以账户里必须存一定数量的 SOL。

这个 SOL 不是交易手续费，而是为了让账户保持 rent-exempt，也就是免租状态。

简单理解：

```text
账户数据越大，需要存入的 SOL 越多
```

开发时经常会看到：

```rust
Rent::get()?.minimum_balance(space)
```

或者 Anchor 里：

```rust
#[account(
  init,
  payer = user,
  space = 8 + 8
)]
pub counter: Account<'info, Counter>,
```

这里的 `space` 决定账户需要多少存储空间，也影响需要多少 SOL。

---

## 10. PDA：程序派生地址

PDA 全称是 Program Derived Address。

PDA 是由程序 ID 和一组 seeds 派生出来的地址：

```text
PDA = hash(seeds + program_id)
```

例如：

```text
seeds = ["counter", user_pubkey]
program_id = CounterProgram
```

可以得到一个固定地址：

```text
Counter PDA
```

PDA 的特点：

- 地址可以被确定性计算出来
- 没有私钥
- 不能像普通钱包一样签名
- 但程序可以通过 seeds 为 PDA “签名”
- 常用来作为程序控制的数据账户地址

例如一个用户对应一个计数器：

```text
counter_pda = find_program_address(
  ["counter", user_pubkey],
  program_id
)
```

这样每个用户都有自己的 Counter Account。

---

## 11. PDA 为什么重要？

假设你做一个质押协议，需要给每个用户存一份状态：

```text
UserStakeInfo {
  amount: u64,
  reward_debt: u64,
}
```

你可以用 PDA 作为用户的状态账户：

```text
seeds = ["stake", user]
```

这样账户地址就是确定的：

```text
User A stake account = PDA("stake", User A)
User B stake account = PDA("stake", User B)
```

好处是：

- 不需要前端随机生成地址
- 程序可以验证这个账户是不是正确的 PDA
- 防止用户传入伪造账户
- 方便查找和索引

Anchor 里常见写法：

```rust
#[account(
  init,
  payer = user,
  space = 8 + UserStakeInfo::INIT_SPACE,
  seeds = [b"stake", user.key().as_ref()],
  bump
)]
pub stake_info: Account<'info, UserStakeInfo>,
```

---

## 12. 一个简单的 Anchor 账户示例

假设我们写一个计数器程序。

账户结构：

```rust
#[account]
pub struct Counter {
    pub count: u64,
}
```

初始化账户：

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 8
    )]
    pub counter: Account<'info, Counter>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

这里：

- `init`：创建账户
- `payer = user`：由 user 支付创建账户所需 SOL
- `space = 8 + 8`：账户大小

为什么是 `8 + 8`？

- 前面的 8 字节是 Anchor discriminator
- 后面的 8 字节是 `u64 count` 的大小

初始化逻辑：

```rust
pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.counter.count = 0;
    Ok(())
}
```

递增逻辑：

```rust
#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

pub fn increment(ctx: Context<Increment>) -> Result<()> {
    ctx.accounts.counter.count += 1;
    Ok(())
}
```

注意 `increment` 里的 `counter` 必须是 `mut`，因为要修改它。

---

## 13. 账户大小 space 怎么计算？

在 Solana 里，账户创建后大小通常是固定的。

所以创建账户时要提前分配空间。

常见类型大小：

```text
u8      = 1 byte
bool    = 1 byte
u16     = 2 bytes
u32     = 4 bytes
u64     = 8 bytes
u128    = 16 bytes
Pubkey  = 32 bytes
```

例如：

```rust
#[account]
pub struct UserProfile {
    pub authority: Pubkey,
    pub age: u8,
    pub score: u64,
}
```

数据大小：

```text
authority: 32
age: 1
score: 8
总计: 41
```

Anchor 账户还要加 8 字节 discriminator：

```text
space = 8 + 32 + 1 + 8
space = 49
```

如果有 String 或 Vec，要预留最大长度。

例如：

```rust
pub name: String
```

String 的空间通常按：

```text
4 + 最大字节长度
```

因为前 4 字节保存长度。

如果 name 最多 32 字节：

```text
space = 8 + 4 + 32
```

---

## 14. Token Account 也是账户

在 Solana 里，用户不是直接在钱包账户里持有 SPL Token。

例如 Alice 有 100 USDC，不是这样：

```text
Alice wallet:
  USDC = 100
```

而是这样：

```text
Alice Wallet Account
Alice USDC Token Account
```

Alice 的 USDC 余额存在一个 Token Account 里。

Token Account 里面大概保存：

```text
mint: 这个 token 是哪种 token
owner: 这个 token 账户属于哪个用户
amount: token 数量
```

这里容易混淆：Token Account 有两个 owner 概念。

第一种是 Solana 账户字段里的 owner：

```text
account.owner = SPL Token Program
```

表示这个账户的数据由 SPL Token Program 管理。

第二种是 Token Account 数据里的 owner：

```text
token_account.owner = Alice
```

表示这个 token 余额属于 Alice。

这两个不是一回事。

---

## 15. ATA：Associated Token Account

ATA 是 Associated Token Account，关联代币账户。

它是用户某种 token 的标准账户地址。

比如 Alice 的 USDC ATA：

```text
ATA = derive(Alice wallet, USDC mint)
```

所以一个用户对一种 Token 通常有一个标准 ATA：

```text
Alice + USDC Mint => Alice 的 USDC ATA
Alice + BONK Mint => Alice 的 BONK ATA
Bob + USDC Mint => Bob 的 USDC ATA
```

这让前端和程序可以很容易找到用户的 token 账户。

---

## 16. 常见账户类型总结

```text
System Account
普通钱包账户，主要持有 SOL。

Program Account
存放程序代码，executable = true。

Data Account
存放程序状态，由某个程序拥有。

PDA Account
由程序派生地址控制，常用于程序状态账户。

Token Mint Account
表示一种 SPL Token。

Token Account
表示某个用户持有某种 Token 的余额。

Associated Token Account
用户某种 Token 的标准 Token Account。
```

---

## 17. Solana 账户模型的核心原则

可以记住这几句话：

### 第一，Solana 上几乎所有东西都是账户

钱包、程序、Token、NFT、状态数据，本质上都是账户。

### 第二，程序和状态分离

程序账户存代码，数据账户存状态。

### 第三，owner 决定谁能修改账户数据

只有账户的 owner 程序才能修改账户 data。

### 第四，交易必须显式传入账户

程序只能访问交易传进来的账户。

### 第五，账户是否可写、是否签名必须提前声明

`writable` 和 `signer` 是 Solana 安全和并行执行的重要基础。

### 第六，PDA 是程序控制账户的关键工具

PDA 让程序可以安全、确定性地管理状态账户。

---

## 18. 一个完整心智模型

可以把 Solana 想象成一个巨大的账户数据库：

```text
Solana Runtime
  ├── Account A: 用户钱包
  ├── Account B: 程序代码
  ├── Account C: 程序状态
  ├── Account D: Token Mint
  ├── Account E: Token Account
  └── Account F: PDA Account
```

用户发送交易时，交易会说：

```text
我要调用哪个程序？
我要传入哪些账户？
哪些账户要签名？
哪些账户要修改？
指令数据是什么？
```

然后 Solana runtime 检查权限，执行程序，并把账户状态更新回链上。

---

## 19. 初学者最容易踩的坑

### 坑 1：以为程序可以随便访问任意账户

不可以。必须在交易里传进来。

### 坑 2：以为钱包账户直接保存 Token 余额

不对。SPL Token 余额存在 Token Account / ATA 里。

### 坑 3：混淆 owner

Solana account 的 owner 是“哪个程序能改这个账户数据”。

Token Account 数据里的 owner 是“这个 token 余额属于哪个用户”。

### 坑 4：忘记 mut

如果账户要被修改，Anchor 里必须写：

```rust
#[account(mut)]
```

### 坑 5：space 算小了

账户空间创建小了，后续写入可能失败。

String、Vec 尤其要提前规划最大长度。

### 坑 6：没有验证 PDA seeds

如果程序没验证传进来的 PDA 是否正确，用户可能传入错误账户，造成安全问题。

---

## 20. 最后总结

Solana 账户模型的关键不是“账户”两个字，而是这套设计：

```text
代码和状态分离
状态放在账户里
账户有明确 owner
交易显式声明访问哪些账户
runtime 借此实现安全和并行执行
```

如果你能理解下面这张关系图，Solana 账户模型就基本入门了：

```text
User Wallet
  |
  | signs transaction
  v

Transaction
  |
  | calls
  v

Program Account
  |
  | reads/writes
  v

Data Account / PDA / Token Account
```

一句话总结：

> **Solana 程序不是自己“拥有一堆内部状态”，而是通过交易传入账户，并在权限允许的情况下读取和修改这些账户。**
