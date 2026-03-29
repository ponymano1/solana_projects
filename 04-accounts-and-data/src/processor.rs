use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::ProfileInstruction, state::UserProfile};

/// 处理指令
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ProfileInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        ProfileInstruction::CreateProfile { name, age, email } => {
            msg!("指令: 创建配置文件");
            process_create_profile(program_id, accounts, name, age, email)
        }
        ProfileInstruction::UpdateProfile { name, age, email } => {
            msg!("指令: 更新配置文件");
            process_update_profile(accounts, name, age, email)
        }
        ProfileInstruction::CloseProfile => {
            msg!("指令: 关闭配置文件");
            process_close_profile(accounts)
        }
    }
}

/// 处理创建配置文件指令
///
/// # 数据存储位置
/// 数据存储在账户的data字段中：profile_info.data
///
/// # 两层Owner概念
/// 1. 账户层面的owner (Account.owner)：
///    - 这是Solana账户模型的owner字段
///    - 表示哪个程序拥有这个账户的控制权
///    - 只有owner程序才能修改账户的data
///    - 在create时必须验证：profile_info.owner == program_id
///
/// 2. 数据层面的owner (UserProfile.owner)：
///    - 这是业务逻辑中的owner字段
///    - 表示这个配置文件属于哪个用户
///    - 用于权限控制：只有数据owner才能更新/删除
///    - 存储在账户data中的UserProfile结构里
fn process_create_profile(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    age: u8,
    email: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let profile_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    // 验证付款人是签名者
    if !payer_info.is_signer {
        msg!("错误: 付款人必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 验证配置文件账户是签名者
    if !profile_info.is_signer {
        msg!("错误: 配置文件账户必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 【账户层面的owner验证】
    // 验证这个账户的owner字段是否为当前程序
    // 这是必须的，因为只有owner程序才能写入账户的data字段
    // 此时账户已经通过System Program创建，owner应该已经设置为program_id
    if profile_info.owner != program_id {
        msg!("错误: 配置文件账户所有者必须是程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 【数据层面的owner设置】
    // 创建UserProfile数据结构，将payer设置为数据的owner
    // 这个owner存储在账户data中，用于后续的权限验证
    let profile = UserProfile::new(*payer_info.key, name, age, email).map_err(|e| {
        msg!("数据验证失败: {}", e);
        ProgramError::InvalidInstructionData
    })?;

    // 【数据存储】
    // 将UserProfile序列化后写入账户的data字段
    // 这就是数据的实际存储位置：profile_info.data
    profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;

    msg!("配置文件创建成功");
    Ok(())
}

/// 处理更新配置文件指令
///
/// # 为什么这里验证数据owner而不是账户owner？
/// 因为：
/// 1. 账户层面的owner肯定是program_id（否则我们无法读取data）
/// 2. 这里需要验证的是业务权限：谁有权修改这个配置文件？
/// 3. 答案是：UserProfile.owner（数据的创建者）
///
/// # 两层owner的作用
/// - Account.owner = program_id  → 程序控制账户
/// - UserProfile.owner = user_pubkey → 用户控制数据
fn process_update_profile(
    accounts: &[AccountInfo],
    name: Option<String>,
    age: Option<u8>,
    email: Option<String>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let profile_info = next_account_info(account_info_iter)?;

    // 验证所有者是签名者
    if !owner_info.is_signer {
        msg!("错误: 所有者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 【从账户data中读取数据】
    // 反序列化存储在profile_info.data中的UserProfile数据
    let mut profile = UserProfile::try_from_slice(&profile_info.data.borrow())?;

    // 验证配置文件已初始化
    if !profile.is_initialized {
        msg!("错误: 配置文件未初始化");
        return Err(ProgramError::UninitializedAccount);
    }

    // 【数据层面的owner验证】
    // 验证调用者是否是UserProfile的owner（数据创建者）
    // 这是业务逻辑层面的权限控制
    if profile.owner != *owner_info.key {
        msg!("错误: 只有所有者可以更新配置文件");
        return Err(ProgramError::IllegalOwner);
    }

    // 更新字段
    if let Some(new_name) = name {
        profile.set_name(new_name).map_err(|e| {
            msg!("名字更新失败: {}", e);
            ProgramError::InvalidInstructionData
        })?;
    }
    if let Some(new_age) = age {
        profile.age = new_age;
    }
    if let Some(new_email) = email {
        profile.set_email(new_email).map_err(|e| {
            msg!("邮箱更新失败: {}", e);
            ProgramError::InvalidInstructionData
        })?;
    }

    // 序列化并写入账户
    profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;

    msg!("配置文件更新成功");
    Ok(())
}

/// 处理关闭配置文件指令
fn process_close_profile(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let profile_info = next_account_info(account_info_iter)?;

    // 验证所有者是签名者
    if !owner_info.is_signer {
        msg!("错误: 所有者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 反序列化配置文件数据
    let profile = UserProfile::try_from_slice(&profile_info.data.borrow())?;

    // 验证配置文件已初始化
    if !profile.is_initialized {
        msg!("错误: 配置文件未初始化");
        return Err(ProgramError::UninitializedAccount);
    }

    // 验证所有者权限
    if profile.owner != *owner_info.key {
        msg!("错误: 只有所有者可以关闭配置文件");
        return Err(ProgramError::IllegalOwner);
    }

    // 转移lamports到所有者账户
    let dest_starting_lamports = owner_info.lamports();
    **owner_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(profile_info.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // 清空配置文件账户
    **profile_info.lamports.borrow_mut() = 0;

    // 清空数据
    let mut data = profile_info.data.borrow_mut();
    data.fill(0);

    msg!("配置文件关闭成功，租金已返还");
    Ok(())
}
