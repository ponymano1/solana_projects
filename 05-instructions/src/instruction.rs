use borsh::{BorshDeserialize, BorshSerialize};

/// Todo程序支持的指令
///
/// # Borsh序列化格式
/// 枚举按顺序序列化，每个变体有不同的格式：
///
/// - Initialize: `[0]`
/// - CreateTodo: `[1, title_len(4), title_bytes, desc_len(4), desc_bytes]`
/// - UpdateTodo: `[2, id(4), completed(1)]`
/// - DeleteTodo: `[3, id(4)]`
///
/// # 客户端调用示例
/// ```typescript
/// // CreateTodo指令
/// const titleBytes = Buffer.from('学习Solana', 'utf-8');
/// const descBytes = Buffer.from('完成第05节', 'utf-8');
/// const data = Buffer.concat([
///   Buffer.from([1]),                          // 指令索引
///   Buffer.from([titleBytes.length, 0, 0, 0]), // title长度
///   titleBytes,
///   Buffer.from([descBytes.length, 0, 0, 0]),  // desc长度
///   descBytes,
/// ]);
/// ```
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TodoInstruction {
    /// 初始化Todo列表
    ///
    /// 设置所有者并标记为已初始化。
    ///
    /// # 账户要求
    /// 0. `[writable, signer]` 所有者账户 - 将成为Todo列表的owner
    /// 1. `[writable]` Todo列表账户 - 将被初始化
    ///
    /// # 客户端调用
    /// ```typescript
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: owner.publicKey, isSigner: true, isWritable: true },
    ///     { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: Buffer.from([0]), // Initialize指令索引
    /// });
    /// ```
    Initialize,

    /// 创建新的Todo
    ///
    /// 添加一个新的Todo项到列表中。
    ///
    /// # 账户要求
    /// 0. `[signer]` 所有者账户 - 必须是Todo列表的owner
    /// 1. `[writable]` Todo列表账户 - 必须已初始化
    ///
    /// # 参数
    /// - `title`: Todo标题（最大50字节）
    /// - `description`: Todo描述（最大200字节）
    ///
    /// # 客户端调用
    /// ```typescript
    /// // 序列化函数
    /// function serializeCreateTodo(title: string, description: string): Buffer {
    ///   const titleBytes = Buffer.from(title, 'utf-8');
    ///   const descBytes = Buffer.from(description, 'utf-8');
    ///   return Buffer.concat([
    ///     Buffer.from([1]),                              // 指令索引
    ///     Buffer.from([titleBytes.length, 0, 0, 0]),     // title长度(小端序)
    ///     titleBytes,
    ///     Buffer.from([descBytes.length, 0, 0, 0]),      // desc长度(小端序)
    ///     descBytes,
    ///   ]);
    /// }
    ///
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: owner.publicKey, isSigner: true, isWritable: true },
    ///     { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: serializeCreateTodo('学习Solana', '完成第05节'),
    /// });
    /// ```
    CreateTodo {
        /// Todo标题
        title: String,
        /// Todo描述
        description: String,
    },

    /// 更新Todo状态
    ///
    /// 标记Todo为完成或未完成。
    ///
    /// # 账户要求
    /// 0. `[signer]` 所有者账户 - 必须是Todo列表的owner
    /// 1. `[writable]` Todo列表账户 - 必须已初始化
    ///
    /// # 参数
    /// - `id`: Todo ID
    /// - `completed`: 是否完成
    ///
    /// # 客户端调用
    /// ```typescript
    /// function serializeUpdateTodo(id: number, completed: boolean): Buffer {
    ///   const buffer = Buffer.alloc(6);
    ///   buffer[0] = 2;                    // 指令索引
    ///   buffer.writeUInt32LE(id, 1);      // id (小端序)
    ///   buffer[5] = completed ? 1 : 0;    // completed
    ///   return buffer;
    /// }
    ///
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: owner.publicKey, isSigner: true, isWritable: true },
    ///     { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: serializeUpdateTodo(0, true), // 标记第一个Todo为完成
    /// });
    /// ```
    UpdateTodo {
        /// Todo ID
        id: u32,
        /// 是否完成
        completed: bool,
    },

    /// 删除Todo
    ///
    /// 从列表中移除指定的Todo项。
    ///
    /// # 账户要求
    /// 0. `[signer]` 所有者账户 - 必须是Todo列表的owner
    /// 1. `[writable]` Todo列表账户 - 必须已初始化
    ///
    /// # 参数
    /// - `id`: 要删除的Todo ID
    ///
    /// # 客户端调用
    /// ```typescript
    /// function serializeDeleteTodo(id: number): Buffer {
    ///   const buffer = Buffer.alloc(5);
    ///   buffer[0] = 3;                    // 指令索引
    ///   buffer.writeUInt32LE(id, 1);      // id (小端序)
    ///   return buffer;
    /// }
    ///
    /// const instruction = new TransactionInstruction({
    ///   keys: [
    ///     { pubkey: owner.publicKey, isSigner: true, isWritable: true },
    ///     { pubkey: todoListAccount.publicKey, isSigner: false, isWritable: true },
    ///   ],
    ///   programId: programId,
    ///   data: serializeDeleteTodo(0), // 删除第一个Todo
    /// });
    /// ```
    DeleteTodo {
        /// Todo ID
        id: u32,
    },
}
