use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// 投票主题
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VoteTopic {
    /// 是否已初始化
    pub is_initialized: bool,
    /// 主题创建者
    pub creator: Pubkey,
    /// 主题描述
    pub description: String,
    /// 选项A的票数
    pub option_a_votes: u64,
    /// 选项B的票数
    pub option_b_votes: u64,
    /// Bump seed（用于PDA）
    pub bump: u8,
}

impl VoteTopic {
    /// 计算账户所需空间
    pub fn space(description_len: usize) -> usize {
        // 1 (is_initialized) + 32 (creator) + 4 (string len) + description_len +
        // 8 (option_a_votes) + 8 (option_b_votes) + 1 (bump)
        1 + 32 + 4 + description_len + 8 + 8 + 1
    }

    /// 最大描述长度
    pub const MAX_DESCRIPTION_LEN: usize = 200;
}

/// 用户投票记录
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserVote {
    /// 是否已初始化
    pub is_initialized: bool,
    /// 投票的主题
    pub topic: Pubkey,
    /// 投票者
    pub voter: Pubkey,
    /// 投票选项（0=A, 1=B）
    pub vote_option: u8,
    /// Bump seed（用于PDA）
    pub bump: u8,
}

impl UserVote {
    /// 计算账户所需空间
    pub fn space() -> usize {
        // 1 (is_initialized) + 32 (topic) + 32 (voter) + 1 (vote_option) + 1 (bump)
        1 + 32 + 32 + 1 + 1
    }
}
