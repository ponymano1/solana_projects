use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod error;
pub mod instruction;
pub mod state;

use error::CalculatorError;
use instruction::CalculatorInstruction;
use state::CalculatorResult;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CalculatorInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        CalculatorInstruction::Add { a, b } => {
            msg!("指令: 加法 {} + {}", a, b);
            process_add(program_id, accounts, a, b)
        }
        CalculatorInstruction::Subtract { a, b } => {
            msg!("指令: 减法 {} - {}", a, b);
            process_subtract(program_id, accounts, a, b)
        }
        CalculatorInstruction::Multiply { a, b } => {
            msg!("指令: 乘法 {} * {}", a, b);
            process_multiply(program_id, accounts, a, b)
        }
        CalculatorInstruction::Divide { a, b } => {
            msg!("指令: 除法 {} / {}", a, b);
            process_divide(program_id, accounts, a, b)
        }
    }
}

fn process_add(program_id: &Pubkey, accounts: &[AccountInfo], a: i64, b: i64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let result_account = next_account_info(accounts_iter)?;

    if result_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let result = a.checked_add(b).ok_or(CalculatorError::Overflow)?;

    let mut calculator_result = get_or_create_result(result_account)?;
    calculator_result.result = result;
    calculator_result.operation_count += 1;

    calculator_result.serialize(&mut &mut result_account.data.borrow_mut()[..])?;

    msg!("结果: {}", result);
    Ok(())
}

fn process_subtract(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    a: i64,
    b: i64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let result_account = next_account_info(accounts_iter)?;

    if result_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let result = a.checked_sub(b).ok_or(CalculatorError::Overflow)?;

    let mut calculator_result = get_or_create_result(result_account)?;
    calculator_result.result = result;
    calculator_result.operation_count += 1;

    calculator_result.serialize(&mut &mut result_account.data.borrow_mut()[..])?;

    msg!("结果: {}", result);
    Ok(())
}

fn process_multiply(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    a: i64,
    b: i64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let result_account = next_account_info(accounts_iter)?;

    if result_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let result = a.checked_mul(b).ok_or(CalculatorError::Overflow)?;

    let mut calculator_result = get_or_create_result(result_account)?;
    calculator_result.result = result;
    calculator_result.operation_count += 1;

    calculator_result.serialize(&mut &mut result_account.data.borrow_mut()[..])?;

    msg!("结果: {}", result);
    Ok(())
}

fn process_divide(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    a: i64,
    b: i64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let result_account = next_account_info(accounts_iter)?;

    if result_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if b == 0 {
        return Err(CalculatorError::DivisionByZero.into());
    }

    let result = a.checked_div(b).ok_or(CalculatorError::Overflow)?;

    let mut calculator_result = get_or_create_result(result_account)?;
    calculator_result.result = result;
    calculator_result.operation_count += 1;

    calculator_result.serialize(&mut &mut result_account.data.borrow_mut()[..])?;

    msg!("结果: {}", result);
    Ok(())
}

fn get_or_create_result(account: &AccountInfo) -> Result<CalculatorResult, ProgramError> {
    if account.data_len() > 0 {
        let data = account.data.borrow();
        let mut data_slice = &data[..];
        CalculatorResult::deserialize(&mut data_slice).map_err(|_| ProgramError::InvalidAccountData)
    } else {
        Ok(CalculatorResult {
            result: 0,
            operation_count: 0,
        })
    }
}
