use solana_sdk::pubkey::Pubkey;
use std::fmt;
use thiserror::Error;

/// Solana基础概念库
///
/// 本模块实现了Solana区块链的核心概念：
/// - Account（账户）：存储数据和SOL的基本单位
/// - Lamports：SOL的最小单位（1 SOL = 10^9 lamports）
/// - Transaction（交易）：在账户之间转移资产或调用程序
///
/// # 错误类型定义
#[derive(Error, Debug)]
pub enum SolanaError {
    #[error("资金不足：需要 {required}，但只有 {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("无效的操作：{0}")]
    InvalidOperation(String),
}

/// Lamports - SOL的最小单位
/// 1 SOL = 1,000,000,000 lamports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lamports(u64);

impl Lamports {
    /// 从SOL数量创建Lamports
    pub fn from_sol(sol: f64) -> Self {
        Self((sol * 1_000_000_000.0) as u64)
    }

    /// 获取lamports数量
    pub fn as_lamports(&self) -> u64 {
        self.0
    }

    /// 加法操作
    pub fn add(&self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// 减法操作（带错误检查）
    pub fn sub(&self, other: Self) -> Result<Self, SolanaError> {
        if self.0 < other.0 {
            return Err(SolanaError::InsufficientFunds {
                required: other.0,
                available: self.0,
            });
        }
        Ok(Self(self.0 - other.0))
    }
}

impl fmt::Display for Lamports {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} lamports", self.0)
    }
}

/// Account - Solana账户
/// 账户是Solana中存储数据和SOL的基本单位
#[derive(Debug, Clone)]
pub struct Account {
    /// 账户所有者（程序ID）
    owner: Pubkey,
    /// 账户余额（lamports）
    lamports: u64,
    /// 账户数据
    data: Vec<u8>,
    /// 是否可执行
    executable: bool,
}

impl Account {
    /// 创建新账户
    pub fn new(owner: Pubkey, lamports: u64) -> Self {
        Self {
            owner,
            lamports,
            data: Vec::new(),
            executable: false,
        }
    }

    /// 获取账户所有者
    pub fn owner(&self) -> &Pubkey {
        &self.owner
    }

    /// 获取账户余额
    pub fn lamports(&self) -> u64 {
        self.lamports
    }

    /// 获取账户数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 设置账户数据
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    /// 检查账户是否可执行
    pub fn is_executable(&self) -> bool {
        self.executable
    }
}

/// Instruction - 交易指令
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Instruction {
    /// 程序ID
    program_id: Pubkey,
    /// 账户列表
    accounts: Vec<Pubkey>,
    /// 指令数据
    data: Vec<u8>,
}

/// Transaction - 交易
/// 交易包含一个或多个指令，用于修改区块链状态
#[derive(Debug, Clone)]
pub struct Transaction {
    /// 交易付款人
    payer: Pubkey,
    /// 指令列表
    instructions: Vec<Instruction>,
    /// 是否已签名
    signed: bool,
}

impl Transaction {
    /// 创建转账交易
    pub fn new_transfer(from: Pubkey, to: Pubkey, lamports: u64) -> Self {
        let instruction = Instruction {
            program_id: solana_sdk::system_program::id(),
            accounts: vec![from, to],
            data: lamports.to_le_bytes().to_vec(),
        };

        Self {
            payer: from,
            instructions: vec![instruction],
            signed: false,
        }
    }

    /// 获取交易付款人
    pub fn payer(&self) -> &Pubkey {
        &self.payer
    }

    /// 获取指令列表
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// 检查是否已签名
    pub fn is_signed(&self) -> bool {
        self.signed
    }

    /// 签名交易
    pub fn sign(&mut self) {
        self.signed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamports_basic() {
        let lamports = Lamports::from_sol(1.0);
        assert_eq!(lamports.as_lamports(), 1_000_000_000);
    }

    #[test]
    fn test_account_basic() {
        let owner = Pubkey::new_unique();
        let account = Account::new(owner, 1000);
        assert_eq!(account.lamports(), 1000);
    }
}
