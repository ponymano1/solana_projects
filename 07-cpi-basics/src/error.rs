use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum TransferError {
    #[error("无效的指令")]
    InvalidInstruction,

    #[error("账户未初始化")]
    UninitializedAccount,

    #[error("金额无效")]
    InvalidAmount,
}

impl From<TransferError> for ProgramError {
    fn from(e: TransferError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
