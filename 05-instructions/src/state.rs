use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Todo项的最大数量
pub const MAX_TODOS: usize = 10;

/// 标题最大长度
pub const MAX_TITLE_LEN: usize = 50;

/// 描述最大长度
pub const MAX_DESCRIPTION_LEN: usize = 200;

/// Todo项
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TodoItem {
    /// Todo ID
    pub id: u32,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 是否完成
    pub completed: bool,
}

/// Todo列表账户
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TodoList {
    /// 是否已初始化
    pub is_initialized: bool,
    /// 所有者
    pub owner: Pubkey,
    /// Todo项列表
    pub todos: Vec<TodoItem>,
    /// 下一个Todo ID
    pub next_id: u32,
}

impl TodoList {
    /// 计算账户所需的最小空间（空列表）
    pub fn space() -> usize {
        // 1 (is_initialized) + 32 (owner) + 4 (vec length) + 4 (next_id)
        1 + 32 + 4 + 4
    }

    /// 计算账户所需的最大空间（满列表）
    pub fn max_space() -> usize {
        // 1 (is_initialized) + 32 (owner) + 4 (vec length) +
        // MAX_TODOS * (4 (id) + 4 (title len) + MAX_TITLE_LEN +
        //              4 (desc len) + MAX_DESCRIPTION_LEN + 1 (completed)) +
        // 4 (next_id)
        1 + 32 + 4 + MAX_TODOS * (4 + 4 + MAX_TITLE_LEN + 4 + MAX_DESCRIPTION_LEN + 1) + 4
    }
}
