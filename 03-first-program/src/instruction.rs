use borsh::{BorshDeserialize, BorshSerialize};

/// 计数器程序支持的指令枚举
///
/// # Borsh序列化
/// 枚举按顺序序列化为单字节索引：
/// - Initialize → [0]
/// - Increment → [1]
/// - Decrement → [2]
/// - Reset → [3]
///
/// # 客户端调用
/// 客户端发送指令时，需要将枚举序列化为字节数组：
/// ```typescript
/// // Initialize指令
/// const data = Buffer.from([0]);
///
/// // Increment指令
/// const data = Buffer.from([1]);
/// ```
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    /// 初始化计数器
    ///
    /// 将计数器设置为0并标记为已初始化。
    ///
    /// # 账户要求
    /// 0. `[writable]` 计数器账户 - 将被初始化
    ///
    /// # 客户端调用
    /// ```typescript
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: counterAccount, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: Buffer.from([0]), // Initialize指令索引
    /// });
    /// ```
    Initialize,

    /// 增加计数
    ///
    /// 将计数值加1（使用checked_add防止溢出）。
    ///
    /// # 账户要求
    /// 0. `[writable]` 计数器账户 - 必须已初始化
    ///
    /// # 客户端调用
    /// ```typescript
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: counterAccount, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: Buffer.from([1]), // Increment指令索引
    /// });
    /// ```
    Increment,

    /// 减少计数
    ///
    /// 将计数值减1（使用checked_sub防止下溢）。
    ///
    /// # 账户要求
    /// 0. `[writable]` 计数器账户 - 必须已初始化
    ///
    /// # 客户端调用
    /// ```typescript
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: counterAccount, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: Buffer.from([2]), // Decrement指令索引
    /// });
    /// ```
    Decrement,

    /// 重置计数
    ///
    /// 将计数值重置为0。
    ///
    /// # 账户要求
    /// 0. `[writable]` 计数器账户 - 必须已初始化
    ///
    /// # 客户端调用
    /// ```typescript
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: counterAccount, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: Buffer.from([3]), // Reset指令索引
    /// });
    /// ```
    Reset,
}
