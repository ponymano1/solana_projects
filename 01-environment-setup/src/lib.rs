//! # 环境搭建验证程序
//!
//! 这是一个简单的Solana程序，用于验证开发环境是否正确配置。

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello, Solana!");
    msg!("程序ID: {:?}", program_id);
    msg!("账户数量: {}", accounts.len());
    Ok(())
}
