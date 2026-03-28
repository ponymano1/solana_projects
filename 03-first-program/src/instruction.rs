use borsh::{BorshDeserialize, BorshSerialize};

/// 计数器程序支持的指令枚举
///
/// 这个枚举定义了程序可以处理的所有指令类型
/// 使用Borsh进行序列化，以便在链上传输
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    /// 初始化计数器
    ///
    /// 账户要求：
    /// 0. `[writable]` 计数器账户 - 将被初始化
    Initialize,

    /// 增加计数
    ///
    /// 账户要求：
    /// 0. `[writable]` 计数器账户 - 必须已初始化
    Increment,

    /// 减少计数
    ///
    /// 账户要求：
    /// 0. `[writable]` 计数器账户 - 必须已初始化
    Decrement,

    /// 重置计数
    ///
    /// 账户要求：
    /// 0. `[writable]` 计数器账户 - 必须已初始化
    Reset,
}
