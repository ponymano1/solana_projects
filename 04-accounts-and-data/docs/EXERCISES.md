# 练习题 - 账户与数据存储

通过这些练习巩固对Solana账户和数据存储的理解。

## 练习1：扩展UserProfile字段

### 目标
为UserProfile添加更多字段，理解空间计算和数据布局。

### 任务
1. 添加以下字段：
   - `bio`: 个人简介（最大128字节）
   - `avatar_url`: 头像URL（最大64字节）
   - `created_at`: 创建时间（i64 Unix时间戳）
   - `updated_at`: 更新时间（i64 Unix时间戳）

2. 更新空间计算函数
3. 修改创建和更新指令
4. 添加相应的测试

### 提示
```rust
pub struct UserProfile {
    // 现有字段...
    pub bio: [u8; 128],
    pub bio_len: u8,
    pub avatar_url: [u8; 64],
    pub avatar_url_len: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

impl UserProfile {
    pub fn space() -> usize {
        // 计算新的总空间
        132 + 128 + 1 + 64 + 1 + 8 + 8 // = 342字节
    }
}
```

### 验证
- [ ] 空间计算正确
- [ ] 创建时自动设置created_at
- [ ] 更新时自动更新updated_at
- [ ] 所有测试通过

---

## 练习2：实现账户大小调整（Realloc）

### 目标
学习如何动态调整账户大小，支持数据结构升级。

### 任务
1. 实现一个新指令`ResizeProfile`
2. 允许增加bio字段的最大长度
3. 正确处理租金补充
4. 保持现有数据不丢失

### 提示
```rust
pub enum ProfileInstruction {
    // 现有指令...

    /// 调整账户大小
    /// Accounts:
    /// 0. [signer, writable] 所有者
    /// 1. [writable] 配置文件账户
    /// 2. [] 系统程序
    ResizeProfile {
        new_bio_max_len: usize,
    },
}
```

### 实现步骤
1. 计算新的空间需求
2. 计算额外租金
3. 调用`account.realloc()`
4. 从所有者账户转移额外租金
5. 更新数据结构

### 验证
- [ ] 账户大小正确增加
- [ ] 租金正确补充
- [ ] 现有数据保持完整
- [ ] 只有所有者可以调整大小

---

## 练习3：实现批量操作

### 目标
学习如何在一个交易中处理多个账户，提高效率。

### 任务
1. 实现`BatchCreateProfiles`指令
2. 在一个交易中创建多个用户配置文件
3. 优化Gas使用
4. 处理部分失败情况

### 提示
```rust
pub enum ProfileInstruction {
    // 现有指令...

    /// 批量创建配置文件
    /// Accounts:
    /// 0. [signer, writable] 付款人
    /// 1..N. [signer, writable] 新账户
    /// N+1. [] 系统程序
    BatchCreateProfiles {
        profiles: Vec<ProfileData>,
    },
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProfileData {
    pub name: String,
    pub age: u8,
    pub email: String,
}
```

### 实现要点
1. 迭代所有账户
2. 为每个账户创建和初始化
3. 使用循环处理
4. 考虑交易大小限制

### 验证
- [ ] 可以批量创建多个配置文件
- [ ] 所有配置文件正确初始化
- [ ] 租金计算正确
- [ ] 处理账户数量限制

---

## 练习4：添加访问控制

### 目标
实现更复杂的权限系统，支持多种角色。

### 任务
1. 添加角色系统：
   - `Owner`: 完全控制
   - `Admin`: 可以更新但不能删除
   - `Viewer`: 只能读取

2. 实现权限检查
3. 添加授权/撤销功能
4. 记录权限变更日志

### 数据结构
```rust
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq)]
pub enum Role {
    Owner,
    Admin,
    Viewer,
}

pub struct UserProfile {
    // 现有字段...
    pub permissions: [(Pubkey, Role); 5], // 最多5个授权用户
    pub permission_count: u8,
}
```

### 新指令
```rust
pub enum ProfileInstruction {
    // 现有指令...

    /// 授予权限
    GrantPermission {
        user: Pubkey,
        role: Role,
    },

    /// 撤销权限
    RevokePermission {
        user: Pubkey,
    },
}
```

### 实现要点
1. 在每个操作中检查权限
2. Owner可以授予/撤销权限
3. Admin可以更新但不能删除
4. Viewer只能读取（链下检查）

### 验证
- [ ] Owner有完全控制权
- [ ] Admin可以更新但不能删除
- [ ] 可以授予和撤销权限
- [ ] 权限列表正确维护
- [ ] 未授权用户无法操作

---

## 练习5：实现数据迁移

### 目标
学习如何安全地升级数据结构，保持向后兼容。

### 任务
1. 创建UserProfileV2结构
2. 实现从V1到V2的迁移
3. 支持两个版本共存
4. 提供迁移工具

### 版本控制
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub enum ProfileVersion {
    V1(UserProfileV1),
    V2(UserProfileV2),
}

pub struct UserProfileV1 {
    pub version: u8, // = 1
    // V1字段...
}

pub struct UserProfileV2 {
    pub version: u8, // = 2
    // V2字段（包含V1所有字段 + 新字段）
}
```

### 迁移指令
```rust
pub enum ProfileInstruction {
    // 现有指令...

    /// 迁移到V2
    /// Accounts:
    /// 0. [signer, writable] 所有者
    /// 1. [writable] 配置文件账户
    MigrateToV2,
}
```

### 实现步骤
1. 读取当前版本
2. 检查是否需要迁移
3. 转换数据结构
4. 调整账户大小（如果需要）
5. 写入新版本数据

### 验证
- [ ] V1账户可以正常使用
- [ ] V2账户可以正常使用
- [ ] 迁移过程不丢失数据
- [ ] 迁移后功能正常
- [ ] 可以回滚（可选）

---

## 练习6：实现数据压缩

### 目标
学习如何使用压缩减少存储成本。

### 任务
1. 使用位标志压缩布尔字段
2. 使用枚举压缩状态
3. 实现自定义压缩算法
4. 对比压缩前后的空间

### 压缩技术
```rust
// 使用位标志
pub struct ProfileFlags {
    // 8个布尔值压缩到1个字节
    flags: u8,
}

impl ProfileFlags {
    const IS_INITIALIZED: u8 = 1 << 0;
    const IS_VERIFIED: u8 = 1 << 1;
    const IS_PREMIUM: u8 = 1 << 2;
    // ...

    pub fn is_initialized(&self) -> bool {
        self.flags & Self::IS_INITIALIZED != 0
    }

    pub fn set_initialized(&mut self, value: bool) {
        if value {
            self.flags |= Self::IS_INITIALIZED;
        } else {
            self.flags &= !Self::IS_INITIALIZED;
        }
    }
}
```

### 验证
- [ ] 空间使用减少
- [ ] 功能保持不变
- [ ] 性能没有明显下降
- [ ] 代码可读性可接受

---

## 挑战练习：实现完整的社交网络配置文件

### 目标
综合运用所有学到的知识，构建一个复杂的系统。

### 功能需求
1. 用户配置文件（基础信息）
2. 好友列表（使用PDA）
3. 帖子系统（独立账户）
4. 点赞和评论
5. 隐私设置
6. 数据导出

### 架构设计
```
UserProfile (主账户)
├── FriendList (PDA)
├── Posts[] (多个账户)
│   ├── Post1
│   │   ├── Likes (PDA)
│   │   └── Comments[] (多个账户)
│   └── Post2
└── Settings (PDA)
```

### 技术要点
- 使用PDA管理关联数据
- 实现分页查询
- 优化存储成本
- 考虑隐私和安全
- 提供良好的用户体验

### 验证
- [ ] 所有功能正常工作
- [ ] 性能可接受
- [ ] 存储成本合理
- [ ] 安全性良好
- [ ] 代码质量高

---

## 学习资源

### 官方文档
- [Solana账户模型](https://docs.solana.com/developing/programming-model/accounts)
- [租金机制](https://docs.solana.com/developing/programming-model/accounts#rent)
- [Borsh序列化](https://borsh.io/)

### 示例项目
- [Solana Program Library](https://github.com/solana-labs/solana-program-library)
- [Anchor Examples](https://github.com/coral-xyz/anchor/tree/master/examples)

### 社区
- [Solana Stack Exchange](https://solana.stackexchange.com/)
- [Solana Discord](https://discord.gg/solana)

---

## 提交作业

完成练习后，请：
1. 确保所有测试通过
2. 运行`cargo fmt`和`cargo clippy`
3. 编写清晰的文档
4. 提交代码到GitHub
5. 在README中说明完成了哪些练习

祝学习愉快！
