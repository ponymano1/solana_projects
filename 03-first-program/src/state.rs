use borsh::{BorshDeserialize, BorshSerialize};

/// 计数器状态结构
///
/// # 数据存储
/// 这个结构体序列化后存储在账户的data字段中：
/// Account.data = serialize(Counter)
///
/// # 内存布局
/// 字节偏移:  0  1  2  3  4  5  6  7  8
/// 字段:     [     count (u64)      ][i]
/// 其中 i = is_initialized (bool)
///
/// # 为什么需要is_initialized？
/// - 防止重复初始化
/// - 区分"未初始化"和"初始化为0"
/// - 确保账户在使用前已正确设置
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Counter {
    /// 计数值（8字节，小端序）
    pub count: u64,

    /// 是否已初始化（1字节，0=false, 1=true）
    /// 用于防止重复初始化和确保账户在使用前已正确设置
    pub is_initialized: bool,
}

impl Counter {
    /// Counter结构体序列化后的字节大小
    ///
    /// 计算：
    /// - u64: 8字节
    /// - bool: 1字节
    /// - 总计: 9字节
    pub const LEN: usize = 9;
}
