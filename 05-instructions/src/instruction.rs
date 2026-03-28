use borsh::{BorshDeserialize, BorshSerialize};

/// Todo程序支持的指令
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TodoInstruction {
    /// 初始化Todo列表
    ///
    /// 账户:
    /// 0. `[writable, signer]` 所有者账户
    /// 1. `[writable]` Todo列表账户
    Initialize,

    /// 创建新的Todo
    ///
    /// 账户:
    /// 0. `[signer]` 所有者账户
    /// 1. `[writable]` Todo列表账户
    CreateTodo {
        /// Todo标题
        title: String,
        /// Todo描述
        description: String,
    },

    /// 更新Todo状态
    ///
    /// 账户:
    /// 0. `[signer]` 所有者账户
    /// 1. `[writable]` Todo列表账户
    UpdateTodo {
        /// Todo ID
        id: u32,
        /// 是否完成
        completed: bool,
    },

    /// 删除Todo
    ///
    /// 账户:
    /// 0. `[signer]` 所有者账户
    /// 1. `[writable]` Todo列表账户
    DeleteTodo {
        /// Todo ID
        id: u32,
    },
}
