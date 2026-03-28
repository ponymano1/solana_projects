use solana_program::program_error::ProgramError;
use thiserror::Error;

/// 自定义错误类型
#[derive(Error, Debug, Copy, Clone)]
pub enum VoteError {
    /// 无效的指令
    #[error("无效的指令")]
    InvalidInstruction,

    /// 账户未初始化
    #[error("账户未初始化")]
    UninitializedAccount,

    /// 账户已初始化
    #[error("账户已初始化")]
    AlreadyInitialized,

    /// PDA派生失败
    #[error("PDA派生失败")]
    InvalidPDA,

    /// 已经投过票
    #[error("已经投过票")]
    AlreadyVoted,

    /// 投票选项无效
    #[error("投票选项无效")]
    InvalidVoteOption,
}

impl From<VoteError> for ProgramError {
    fn from(e: VoteError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
