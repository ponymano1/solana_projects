use borsh::{BorshDeserialize, BorshSerialize};

/// 计数器状态结构
///
/// 这个结构体定义了计数器程序的状态数据
/// 使用Borsh进行序列化和反序列化
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Counter {
    /// 计数值
    pub count: u64,
    /// 是否已初始化
    /// 用于防止重复初始化和确保账户在使用前已正确设置
    pub is_initialized: bool,
}

impl Counter {
    /// Counter结构体序列化后的字节大小
    /// u64 (8字节) + bool (1字节) = 9字节
    pub const LEN: usize = 9;
}
