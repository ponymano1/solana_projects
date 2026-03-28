use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// 转账记录
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TransferRecord {
    /// 是否已初始化
    pub is_initialized: bool,
    /// 发送者
    pub from: Pubkey,
    /// 接收者
    pub to: Pubkey,
    /// 金额
    pub amount: u64,
    /// 时间戳（slot）
    pub timestamp: u64,
}

impl TransferRecord {
    pub fn space() -> usize {
        // 1 + 32 + 32 + 8 + 8
        81
    }
}
