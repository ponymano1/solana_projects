# 练习题

## 练习1：修改欢迎消息

**难度**: ⭐️

**目标**: 熟悉程序的基本结构和日志输出

**任务**:
1. 修改`src/lib.rs`中的程序，使其输出你自己的欢迎消息
2. 添加更多的日志信息，例如：
   - 当前时间戳（使用`Clock` sysvar）
   - 指令数据的长度
   - 第一个账户的地址（如果存在）

**提示**:
```rust
use solana_program::msg;

msg!("你的自定义消息");
msg!("指令数据长度: {}", instruction_data.len());
if !accounts.is_empty() {
    msg!("第一个账户: {:?}", accounts[0].key);
}
```

**验证**:
运行`cargo test -- --nocapture`，确认你的消息出现在日志中。

---

## 练习2：添加账户验证

**难度**: ⭐️⭐️

**目标**: 学习基本的账户验证逻辑

**任务**:
1. 修改程序，要求至少传入1个账户
2. 如果没有传入账户，返回错误
3. 验证第一个账户是否为签名者（signer）
4. 编写测试验证你的逻辑

**提示**:
```rust
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // 获取账户迭代器
    let accounts_iter = &mut accounts.iter();

    // 获取第一个账户
    let account = next_account_info(accounts_iter)?;

    // 验证是否为签名者
    if !account.is_signer {
        msg!("错误: 账户必须是签名者");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("验证通过！签名者: {:?}", account.key);
    Ok(())
}
```

**测试提示**:
```rust
use solana_sdk::instruction::AccountMeta;

// 创建签名者账户
let signer = solana_sdk::pubkey::Pubkey::new_unique();

// 在指令中标记为签名者
let instruction = solana_sdk::instruction::Instruction {
    program_id,
    accounts: vec![
        AccountMeta::new(signer, true),  // true表示是签名者
    ],
    data: vec![],
};
```

**验证**:
1. 测试传入签名者账户时程序成功
2. 测试传入非签名者账户时程序失败
3. 测试不传入账户时程序失败

---

## 练习3：解析指令数据

**难度**: ⭐️⭐️⭐️

**目标**: 学习处理和解析指令数据

**任务**:
1. 定义一个简单的指令枚举：
   - `Initialize`: 初始化操作
   - `Increment`: 增加操作
   - `Decrement`: 减少操作
2. 实现指令的序列化和反序列化
3. 根据不同的指令打印不同的消息
4. 编写测试验证每种指令

**提示**:
```rust
use solana_program::{
    msg,
    program_error::ProgramError,
};

// 定义指令枚举
pub enum MyInstruction {
    Initialize,
    Increment,
    Decrement,
}

impl MyInstruction {
    // 从字节数组解析指令
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, _rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::Increment,
            2 => Self::Decrement,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MyInstruction::unpack(instruction_data)?;

    match instruction {
        MyInstruction::Initialize => {
            msg!("执行初始化操作");
        }
        MyInstruction::Increment => {
            msg!("执行增加操作");
        }
        MyInstruction::Decrement => {
            msg!("执行减少操作");
        }
    }

    Ok(())
}
```

**测试提示**:
```rust
#[tokio::test]
async fn test_initialize_instruction() {
    // ... 设置测试环境 ...

    let instruction = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![],
        data: vec![0],  // 0 = Initialize
    };

    // ... 发送交易并验证 ...
}

#[tokio::test]
async fn test_increment_instruction() {
    // ... 设置测试环境 ...

    let instruction = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![],
        data: vec![1],  // 1 = Increment
    };

    // ... 发送交易并验证 ...
}

#[tokio::test]
async fn test_invalid_instruction() {
    // ... 设置测试环境 ...

    let instruction = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![],
        data: vec![99],  // 无效的指令
    };

    // 验证交易失败
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}
```

**验证**:
1. 测试每种指令都能正确解析和执行
2. 测试无效的指令数据会返回错误
3. 测试空的指令数据会返回错误

---

## 挑战练习：环境信息查询程序

**难度**: ⭐️⭐️⭐️⭐️

**目标**: 综合运用所学知识，创建一个实用的程序

**任务**:
创建一个程序，可以查询和显示Solana运行时的各种信息：

1. **查询时钟信息**:
   - 当前slot
   - 当前时间戳
   - 当前epoch

2. **查询租金信息**:
   - 每字节每epoch的租金
   - 租金豁免的最低余额

3. **查询账户信息**:
   - 账户余额
   - 账户所有者
   - 账户数据大小

**提示**:
```rust
use solana_program::{
    clock::Clock,
    rent::Rent,
    sysvar::Sysvar,
};

// 获取时钟信息
let clock = Clock::get()?;
msg!("当前slot: {}", clock.slot);
msg!("当前时间戳: {}", clock.unix_timestamp);
msg!("当前epoch: {}", clock.epoch);

// 获取租金信息
let rent = Rent::get()?;
msg!("每字节每epoch租金: {}", rent.lamports_per_byte_year);

// 查询账户信息
let account = next_account_info(accounts_iter)?;
msg!("账户余额: {} lamports", account.lamports());
msg!("账户所有者: {:?}", account.owner);
msg!("账户数据大小: {} bytes", account.data_len());
```

**扩展挑战**:
1. 添加指令来选择查询哪种信息
2. 计算给定数据大小的租金豁免金额
3. 验证账户是否满足租金豁免条件
4. 编写完整的测试套件

**验证**:
1. 程序能正确查询所有类型的信息
2. 所有测试通过
3. 代码有清晰的注释
4. 错误处理完善

---

## 学习建议

1. **按顺序完成**: 练习难度递增，建议按顺序完成
2. **理解原理**: 不要只是复制代码，理解每行代码的作用
3. **多写测试**: 测试是验证理解的最好方式
4. **查阅文档**: 遇到不懂的API，查阅官方文档
5. **实验探索**: 尝试修改代码，观察结果变化

## 参考答案

练习的参考答案可以在`examples/`目录中找到（如果提供）。但建议先自己尝试完成，再查看参考答案。

## 提交作业

如果你在学习小组或课程中，可以：
1. 将你的代码推送到Git仓库
2. 确保所有测试通过：`cargo test`
3. 确保代码格式正确：`cargo fmt`
4. 确保没有警告：`cargo clippy`
5. 提交Pull Request或分享你的仓库链接

## 下一步

完成这些练习后，你应该：
- ✅ 熟悉Solana程序的基本结构
- ✅ 能够处理账户和指令数据
- ✅ 会编写和运行测试
- ✅ 理解基本的错误处理

继续学习 **第02节 - Solana基础概念**，深入理解账户模型和程序架构。
