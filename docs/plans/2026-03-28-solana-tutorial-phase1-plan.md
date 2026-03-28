# Solana教程阶段一实施计划（第1-8节）

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans

**Goal:** 完成Solana教程的原生开发基础部分（第1-8节），建立完整的学习框架

**Architecture:** 每节独立Rust项目，包含src/、tests/、docs/目录，所有注释中文

**Tech Stack:** Rust 1.75+, Solana CLI 1.18+, Borsh, solana-program-test

---

## 通用模板结构

每节都遵循以下标准结构：

### 目录结构
```
XX-topic-name/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── instruction.rs (可选)
│   ├── state.rs (可选)
│   └── error.rs (可选)
├── tests/
│   └── integration.rs
└── docs/
    ├── CONCEPTS.md
    └── EXERCISES.md (可选)
```

### Cargo.toml模板
```toml
[package]
name = "lesson-name"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
solana-program = "~1.18"
borsh = "0.10"
thiserror = "1.0"

[dev-dependencies]
solana-program-test = "~1.18"
solana-sdk = "~1.18"
tokio = "1"
```

### README.md模板
```markdown
# 第XX节：[标题]

## 学习目标
- 目标1
- 目标2
- 目标3

## 前置知识
- 需要完成的前置章节

## 运行步骤

### 1. 编译程序
\`\`\`bash
cargo build-bpf
\`\`\`

### 2. 运行测试
\`\`\`bash
cargo test -- --nocapture
\`\`\`

### 3. 部署到本地测试网（可选）
\`\`\`bash
solana-test-validator
solana program deploy target/deploy/program.so
\`\`\`

## 核心概念
[简要说明，详见CONCEPTS.md]

## 常见问题
Q: ...
A: ...
```

---

## Task 1: 第01节 - 环境搭建

**学习目标：**
- 安装Rust和Solana CLI
- 配置本地测试网
- 创建第一个Hello World程序
- 理解Solana程序的基本结构

**核心内容：**
1. 环境安装指南（Rust、Solana CLI、Node.js）
2. 配置本地测试网（solana-test-validator）
3. 创建最简单的程序（只打印日志）
4. 编写第一个测试

**CONCEPTS.md要点：**
- Solana账户模型 vs 以太坊账户模型
- 程序（Program）vs 智能合约（Smart Contract）
- 本地测试网的优势
- Solana CLI常用命令

**代码示例：**
```rust
// lib.rs - 最简单的Hello World
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello, Solana!");
    Ok(())
}
```

---

## Task 2: 第02节 - Solana基础概念

**学习目标：**
- 理解账户（Account）的概念
- 理解程序ID和账户所有权
- 学习Lamports和SOL的关系
- 理解租金（Rent）机制

**核心内容：**
1. 账户结构详解（data, lamports, owner, executable）
2. 程序账户 vs 数据账户
3. 系统程序（System Program）
4. 租金豁免（Rent Exemption）

**CONCEPTS.md要点：**
- Solana的账户模型详解
- 与以太坊状态存储的对比
- 为什么需要租金机制
- 如何计算租金豁免金额

**代码示例：**
```rust
// 演示如何读取账户信息
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account = &accounts[0];

    msg!("账户地址: {:?}", account.key);
    msg!("账户余额: {} lamports", account.lamports());
    msg!("账户所有者: {:?}", account.owner);
    msg!("账户数据长度: {} bytes", account.data_len());
    msg!("是否可执行: {}", account.executable);

    Ok(())
}
```

---

## Task 3: 第03节 - 第一个原生程序

**学习目标：**
- 创建可以接收和处理数据的程序
- 学习使用Borsh序列化
- 实现简单的计数器程序
- 理解账户数据的读写

**核心内容：**
1. 使用Borsh序列化/反序列化
2. 定义指令枚举
3. 读取和修改账户数据
4. 所有权检查

**代码示例：**
```rust
// instruction.rs
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    /// 初始化计数器
    Initialize,
    /// 增加计数
    Increment,
    /// 重置计数
    Reset,
}

// state.rs
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Counter {
    pub count: u64,
}
```

---

## Task 4: 第04节 - 账户与数据存储

**学习目标：**
- 深入理解账户数据存储
- 学习创建新账户
- 理解账户大小和租金计算
- 实现数据的CRUD操作

**核心内容：**
1. 使用System Program创建账户
2. 计算账户所需空间
3. 转移所有权给程序
4. 数据序列化最佳实践

**代码示例：**
```rust
// 创建账户的指令
pub fn create_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    space: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    let new_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // 计算租金
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space as usize);

    // 调用System Program创建账户
    invoke(
        &system_instruction::create_account(
            payer.key,
            new_account.key,
            lamports,
            space,
            program_id,
        ),
        &[payer.clone(), new_account.clone(), system_program.clone()],
    )?;

    Ok(())
}
```

---

## Task 5: 第05节 - 指令处理

**学习目标：**
- 设计复杂的指令结构
- 实现指令路由
- 处理指令参数
- 错误处理最佳实践

**核心内容：**
1. 指令枚举设计
2. 指令解析和分发
3. 自定义错误类型
4. 参数验证

**代码示例：**
```rust
// error.rs
#[derive(Error, Debug, Copy, Clone)]
pub enum TodoError {
    #[error("无效的指令")]
    InvalidInstruction,
    #[error("账户未初始化")]
    UninitializedAccount,
    #[error("权限不足")]
    Unauthorized,
}

// instruction.rs
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TodoInstruction {
    CreateTodo { title: String, description: String },
    UpdateTodo { id: u32, completed: bool },
    DeleteTodo { id: u32 },
}
```

---

## Task 6: 第06节 - PDA基础

**学习目标：**
- 理解PDA（Program Derived Address）概念
- 学习如何生成PDA
- 理解bump seed的作用
- 实现基于PDA的账户管理

**核心内容：**
1. PDA vs 普通地址
2. find_program_address函数
3. PDA作为签名者
4. PDA的实际应用场景

**CONCEPTS.md要点：**
- 为什么需要PDA
- PDA如何解决签名问题
- 与以太坊合约地址的对比
- PDA的安全性考虑

**代码示例：**
```rust
// 生成PDA
pub fn find_user_pda(
    user: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user", user.as_ref()],
        program_id,
    )
}

// 验证PDA
pub fn verify_pda(
    pda: &Pubkey,
    seeds: &[&[u8]],
    bump: u8,
    program_id: &Pubkey,
) -> ProgramResult {
    let expected_pda = Pubkey::create_program_address(
        &[seeds, &[bump]].concat(),
        program_id,
    )?;

    if expected_pda != *pda {
        return Err(ProgramError::InvalidSeeds);
    }

    Ok(())
}
```

---

## Task 7: 第07节 - CPI基础

**学习目标：**
- 理解CPI（Cross-Program Invocation）
- 学习调用其他程序
- 理解invoke和invoke_signed的区别
- 实现程序间交互

**核心内容：**
1. CPI的工作原理
2. invoke函数使用
3. invoke_signed与PDA结合
4. CPI权限和安全性

**CONCEPTS.md要点：**
- CPI vs 以太坊的合约调用
- 为什么需要invoke_signed
- CPI的深度限制
- 常见的CPI模式

**代码示例：**
```rust
// 调用System Program转账
pub fn transfer_lamports(
    from: &AccountInfo,
    to: &AccountInfo,
    system_program: &AccountInfo,
    amount: u64,
) -> ProgramResult {
    invoke(
        &system_instruction::transfer(from.key, to.key, amount),
        &[from.clone(), to.clone(), system_program.clone()],
    )?;
    Ok(())
}

// 使用PDA签名的CPI
pub fn transfer_with_pda(
    pda: &AccountInfo,
    to: &AccountInfo,
    system_program: &AccountInfo,
    amount: u64,
    seeds: &[&[u8]],
    bump: u8,
) -> ProgramResult {
    invoke_signed(
        &system_instruction::transfer(pda.key, to.key, amount),
        &[pda.clone(), to.clone(), system_program.clone()],
        &[&[seeds, &[bump]].concat()],
    )?;
    Ok(())
}
```

---

## Task 8: 第08节 - 测试与调试

**学习目标：**
- 掌握solana-program-test框架
- 编写完整的集成测试
- 学习调试技巧
- 理解测试最佳实践

**核心内容：**
1. ProgramTest环境搭建
2. BanksClient使用
3. 测试账户和交易
4. 日志和错误调试

**代码示例：**
```rust
#[tokio::test]
async fn test_counter_program() {
    // 1. 设置测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "counter_program",
        program_id,
        processor!(process_instruction),
    );

    // 2. 启动测试环境
    let (mut banks_client, payer, recent_blockhash) =
        program_test.start().await;

    // 3. 创建测试账户
    let counter_account = Keypair::new();

    // 4. 构建并发送初始化指令
    let mut transaction = Transaction::new_with_payer(
        &[initialize_instruction(
            &program_id,
            &counter_account.pubkey(),
            &payer.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &counter_account], recent_blockhash);

    // 5. 验证交易成功
    banks_client.process_transaction(transaction).await.unwrap();

    // 6. 验证账户状态
    let account = banks_client
        .get_account(counter_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let counter_data = Counter::try_from_slice(&account.data).unwrap();
    assert_eq!(counter_data.count, 0);
}
```

---

## 实施步骤

### 准备工作
1. 确保开发环境已安装（Rust、Solana CLI）
2. 创建项目根目录结构
3. 初始化git仓库

### 执行顺序
按照Task 1-8的顺序依次实施，每完成一个Task：
1. 创建目录结构
2. 编写所有文件（Cargo.toml、源码、测试、文档）
3. 编译并运行测试
4. 验证所有测试通过
5. 提交到git

### 质量检查
每节完成后检查：
- [ ] 所有注释都是中文
- [ ] README.md包含完整的运行步骤
- [ ] CONCEPTS.md详细讲解核心概念
- [ ] 至少2-3个测试用例
- [ ] 所有测试通过
- [ ] 代码可以独立运行

---

## 后续阶段

完成阶段一后，继续实施：
- **阶段二：** 第9-11节（Anchor框架）
- **阶段三：** 第12-18节（实战应用）
- **阶段四：** 第19-25节（进阶主题）

每个阶段完成后创建对应的实施计划文档。
