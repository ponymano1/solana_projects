use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// 用户配置文件数据结构
///
/// # 数据存储
/// 这个结构体会被序列化后存储在账户的data字段中：
/// Account.data = serialize(UserProfile)
///
/// # Owner字段说明
/// 这里的owner是数据层面的owner，表示这个配置文件属于哪个用户。
/// 不要与Account.owner混淆：
/// - Account.owner = program_id（账户层面，控制账户访问）
/// - UserProfile.owner = user_pubkey（数据层面，控制数据权限）
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct UserProfile {
    /// 是否已初始化
    pub is_initialized: bool,

    /// 配置文件所有者（数据层面的owner）
    /// 这个字段存储在账户data中，用于权限验证
    /// 只有这个owner才能更新/删除配置文件
    pub owner: Pubkey,

    /// 用户名（固定32字节）
    pub name: [u8; 32],
    /// 实际名字长度
    pub name_len: u8,
    /// 年龄
    pub age: u8,
    /// 邮箱地址（固定64字节）
    pub email: [u8; 64],
    /// 实际邮箱长度
    pub email_len: u8,
}

impl UserProfile {
    /// 名字最大长度
    pub const MAX_NAME_LEN: usize = 32;
    /// 邮箱最大长度
    pub const MAX_EMAIL_LEN: usize = 64;

    /// 计算账户所需空间
    ///
    /// 空间计算：
    /// - is_initialized: 1字节（bool）
    /// - owner: 32字节（Pubkey）
    /// - name: 32字节（固定数组）
    /// - name_len: 1字节（u8）
    /// - age: 1字节（u8）
    /// - email: 64字节（固定数组）
    /// - email_len: 1字节（u8）
    pub fn space() -> usize {
        1 + 32 + 32 + 1 + 1 + 64 + 1
    }

    /// 从字符串创建配置文件
    pub fn new(owner: Pubkey, name: String, age: u8, email: String) -> Result<Self, &'static str> {
        if name.len() > Self::MAX_NAME_LEN {
            return Err("名字长度超过限制");
        }
        if email.len() > Self::MAX_EMAIL_LEN {
            return Err("邮箱长度超过限制");
        }
        if name.is_empty() {
            return Err("名字不能为空");
        }
        if email.is_empty() {
            return Err("邮箱不能为空");
        }

        let mut name_bytes = [0u8; 32];
        let name_data = name.as_bytes();
        name_bytes[..name_data.len()].copy_from_slice(name_data);

        let mut email_bytes = [0u8; 64];
        let email_data = email.as_bytes();
        email_bytes[..email_data.len()].copy_from_slice(email_data);

        Ok(Self {
            is_initialized: true,
            owner,
            name: name_bytes,
            name_len: name_data.len() as u8,
            age,
            email: email_bytes,
            email_len: email_data.len() as u8,
        })
    }

    /// 获取名字字符串
    pub fn get_name(&self) -> String {
        String::from_utf8_lossy(&self.name[..self.name_len as usize]).to_string()
    }

    /// 获取邮箱字符串
    pub fn get_email(&self) -> String {
        String::from_utf8_lossy(&self.email[..self.email_len as usize]).to_string()
    }

    /// 更新名字
    pub fn set_name(&mut self, name: String) -> Result<(), &'static str> {
        if name.len() > Self::MAX_NAME_LEN {
            return Err("名字长度超过限制");
        }
        if name.is_empty() {
            return Err("名字不能为空");
        }

        self.name = [0u8; 32];
        let name_data = name.as_bytes();
        self.name[..name_data.len()].copy_from_slice(name_data);
        self.name_len = name_data.len() as u8;
        Ok(())
    }

    /// 更新邮箱
    pub fn set_email(&mut self, email: String) -> Result<(), &'static str> {
        if email.len() > Self::MAX_EMAIL_LEN {
            return Err("邮箱长度超过限制");
        }
        if email.is_empty() {
            return Err("邮箱不能为空");
        }

        self.email = [0u8; 64];
        let email_data = email.as_bytes();
        self.email[..email_data.len()].copy_from_slice(email_data);
        self.email_len = email_data.len() as u8;
        Ok(())
    }
}
