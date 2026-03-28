use borsh::{BorshDeserialize, BorshSerialize};

/// 配置文件程序指令
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum ProfileInstruction {
    /// 创建用户配置文件
    ///
    /// 账户：
    /// 0. [signer, writable] 付款人（支付租金）
    /// 1. [signer, writable] 新账户（配置文件账户）
    /// 2. [] 系统程序
    CreateProfile {
        /// 用户名
        name: String,
        /// 年龄
        age: u8,
        /// 邮箱地址
        email: String,
    },

    /// 更新用户配置文件
    ///
    /// 账户：
    /// 0. [signer] 所有者（必须是配置文件的owner）
    /// 1. [writable] 配置文件账户
    UpdateProfile {
        /// 新的用户名（None表示不更新）
        name: Option<String>,
        /// 新的年龄（None表示不更新）
        age: Option<u8>,
        /// 新的邮箱地址（None表示不更新）
        email: Option<String>,
    },

    /// 关闭用户配置文件（返还租金）
    ///
    /// 账户：
    /// 0. [signer, writable] 所有者（接收返还的租金）
    /// 1. [writable] 配置文件账户（将被关闭）
    CloseProfile,
}
