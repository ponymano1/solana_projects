use solana_program::program_error::ProgramError;
use thiserror::Error;

/// 自定义错误类型
#[derive(Error, Debug, Copy, Clone)]
pub enum TodoError {
    /// 无效的指令
    #[error("无效的指令")]
    InvalidInstruction,

    /// 账户未初始化
    #[error("账户未初始化")]
    UninitializedAccount,

    /// 账户已初始化
    #[error("账户已初始化")]
    AlreadyInitialized,

    /// 权限不足
    #[error("权限不足")]
    Unauthorized,

    /// Todo不存在
    #[error("Todo不存在")]
    TodoNotFound,

    /// Todo列表已满
    #[error("Todo列表已满")]
    TodoListFull,

    /// 标题过长
    #[error("标题过长")]
    TitleTooLong,

    /// 描述过长
    #[error("描述过长")]
    DescriptionTooLong,
}

impl From<TodoError> for ProgramError {
    fn from(e: TodoError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
