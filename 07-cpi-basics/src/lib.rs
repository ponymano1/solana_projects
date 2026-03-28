use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::Sysvar,
};

pub mod error;
pub mod instruction;
pub mod state;

use error::TransferError;
use instruction::TransferInstruction;
use state::TransferRecord;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TransferInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TransferInstruction::TransferWithRecord { amount } => {
            msg!("指令: 通过CPI转账并记录");
            process_transfer_with_record(program_id, accounts, amount)
        }
        TransferInstruction::TransferFromPDA { amount, bump } => {
            msg!("指令: 使用PDA签名的CPI转账");
            process_transfer_from_pda(program_id, accounts, amount, bump)
        }
    }
}

/// 处理普通CPI转账
fn process_transfer_with_record(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let from_account = next_account_info(accounts_iter)?;
    let to_account = next_account_info(accounts_iter)?;
    let record_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // 检查发送者是否签名
    if !from_account.is_signer {
        msg!("错误: 发送者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 验证金额
    if amount == 0 {
        msg!("错误: 金额必须大于0");
        return Err(TransferError::InvalidAmount.into());
    }

    // 检查记录账户所有权
    if record_account.owner != program_id {
        msg!("错误: 记录账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 使用invoke进行CPI调用System Program
    msg!("执行CPI: 从 {:?} 转账 {} lamports 到 {:?}",
        from_account.key, amount, to_account.key);

    invoke(
        &system_instruction::transfer(from_account.key, to_account.key, amount),
        &[
            from_account.clone(),
            to_account.clone(),
            system_program.clone(),
        ],
    )?;

    msg!("CPI转账成功");

    // 记录转账信息
    let clock = Clock::get()?;
    let transfer_record = TransferRecord {
        is_initialized: true,
        from: *from_account.key,
        to: *to_account.key,
        amount,
        timestamp: clock.slot,
    };

    transfer_record.serialize(&mut &mut record_account.data.borrow_mut()[..])?;
    msg!("转账记录已保存");

    Ok(())
}

/// 处理使用PDA签名的CPI转账
fn process_transfer_from_pda(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let pda_account = next_account_info(accounts_iter)?;
    let to_account = next_account_info(accounts_iter)?;
    let record_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // 验证金额
    if amount == 0 {
        msg!("错误: 金额必须大于0");
        return Err(TransferError::InvalidAmount.into());
    }

    // 验证PDA
    let (expected_pda, expected_bump) = Pubkey::find_program_address(
        &[b"transfer_vault"],
        program_id,
    );

    if pda_account.key != &expected_pda {
        msg!("错误: PDA地址不匹配");
        return Err(ProgramError::InvalidSeeds);
    }

    if bump != expected_bump {
        msg!("错误: Bump seed不匹配");
        return Err(ProgramError::InvalidSeeds);
    }

    // 检查记录账户所有权
    if record_account.owner != program_id {
        msg!("错误: 记录账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 使用invoke_signed进行CPI调用，PDA作为签名者
    msg!("执行CPI: 从PDA {:?} 转账 {} lamports 到 {:?}",
        pda_account.key, amount, to_account.key);

    invoke_signed(
        &system_instruction::transfer(pda_account.key, to_account.key, amount),
        &[
            pda_account.clone(),
            to_account.clone(),
            system_program.clone(),
        ],
        &[&[b"transfer_vault", &[bump]]],
    )?;

    msg!("CPI转账成功（使用PDA签名）");

    // 记录转账信息
    let clock = Clock::get()?;
    let transfer_record = TransferRecord {
        is_initialized: true,
        from: *pda_account.key,
        to: *to_account.key,
        amount,
        timestamp: clock.slot,
    };

    transfer_record.serialize(&mut &mut record_account.data.borrow_mut()[..])?;
    msg!("转账记录已保存");

    Ok(())
}
