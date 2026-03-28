use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod instruction;
pub mod state;

use instruction::CounterInstruction;
use state::Counter;

// 声明程序入口点
entrypoint!(process_instruction);

/// 程序入口点
///
/// 所有对程序的调用都会通过这个函数
///
/// # 参数
/// * `program_id` - 当前程序的公钥
/// * `accounts` - 指令需要的账户列表
/// * `instruction_data` - 序列化的指令数据
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 反序列化指令数据
    let instruction = CounterInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // 根据指令类型分发处理
    match instruction {
        CounterInstruction::Initialize => {
            msg!("指令: 初始化计数器");
            process_initialize(program_id, accounts)
        }
        CounterInstruction::Increment => {
            msg!("指令: 增加计数");
            process_increment(program_id, accounts)
        }
        CounterInstruction::Decrement => {
            msg!("指令: 减少计数");
            process_decrement(program_id, accounts)
        }
        CounterInstruction::Reset => {
            msg!("指令: 重置计数");
            process_reset(program_id, accounts)
        }
    }
}

/// 处理初始化指令
fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // 检查账户所有权
    if counter_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut counter = Counter::try_from_slice(&counter_account.data.borrow())?;

    // 检查是否已初始化
    if counter.is_initialized {
        msg!("错误: 账户已经初始化");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // 初始化计数器
    counter.count = 0;
    counter.is_initialized = true;

    // 序列化并保存数据
    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;

    msg!("计数器初始化成功");
    Ok(())
}

/// 处理增加指令
fn process_increment(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // 检查账户所有权
    if counter_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut counter = Counter::try_from_slice(&counter_account.data.borrow())?;

    // 检查是否已初始化
    if !counter.is_initialized {
        msg!("错误: 账户未初始化");
        return Err(ProgramError::UninitializedAccount);
    }

    // 增加计数
    counter.count = counter
        .count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    // 序列化并保存数据
    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;

    msg!("计数增加到: {}", counter.count);
    Ok(())
}

/// 处理减少指令
fn process_decrement(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // 检查账户所有权
    if counter_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut counter = Counter::try_from_slice(&counter_account.data.borrow())?;

    // 检查是否已初始化
    if !counter.is_initialized {
        msg!("错误: 账户未初始化");
        return Err(ProgramError::UninitializedAccount);
    }

    // 减少计数（防止下溢）
    counter.count = counter
        .count
        .checked_sub(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    // 序列化并保存数据
    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;

    msg!("计数减少到: {}", counter.count);
    Ok(())
}

/// 处理重置指令
fn process_reset(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // 检查账户所有权
    if counter_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut counter = Counter::try_from_slice(&counter_account.data.borrow())?;

    // 检查是否已初始化
    if !counter.is_initialized {
        msg!("错误: 账户未初始化");
        return Err(ProgramError::UninitializedAccount);
    }

    // 重置计数
    counter.count = 0;

    // 序列化并保存数据
    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;

    msg!("计数器已重置");
    Ok(())
}
