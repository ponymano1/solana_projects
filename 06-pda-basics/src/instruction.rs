use borsh::{BorshDeserialize, BorshSerialize};

/// 投票程序支持的指令
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum VoteInstruction {
    /// 创建投票主题
    ///
    /// 账户:
    /// 0. `[signer]` 创建者账户
    /// 1. `[writable]` 投票主题PDA账户
    /// 2. `[]` 系统程序
    CreateTopic {
        /// 主题描述
        description: String,
        /// Bump seed
        bump: u8,
    },

    /// 投票
    ///
    /// 账户:
    /// 0. `[signer]` 投票者账户
    /// 1. `[writable]` 投票主题PDA账户
    /// 2. `[writable]` 用户投票记录PDA账户
    /// 3. `[]` 系统程序
    Vote {
        /// 投票选项（0=A, 1=B）
        option: u8,
        /// 用户投票记录的bump seed
        bump: u8,
    },
}
