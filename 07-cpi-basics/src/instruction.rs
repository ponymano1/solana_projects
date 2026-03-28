use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TransferInstruction {
    /// 通过CPI转账并记录
    ///
    /// 账户:
    /// 0. `[signer, writable]` 发送者
    /// 1. `[writable]` 接收者
    /// 2. `[writable]` 转账记录账户
    /// 3. `[]` 系统程序
    TransferWithRecord {
        /// 转账金额
        amount: u64,
    },

    /// 使用PDA签名的CPI转账
    ///
    /// 账户:
    /// 0. `[writable]` PDA账户（发送者）
    /// 1. `[writable]` 接收者
    /// 2. `[writable]` 转账记录账户
    /// 3. `[]` 系统程序
    TransferFromPDA {
        /// 转账金额
        amount: u64,
        /// PDA的bump seed
        bump: u8,
    },
}
