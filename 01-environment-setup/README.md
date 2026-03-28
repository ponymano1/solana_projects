# 第01节 - 环境搭建

## 学习目标

- 理解Solana开发环境的组成部分
- 掌握Rust和Solana CLI的安装配置
- 创建并运行第一个Solana程序
- 学会使用solana-program-test编写测试
- 理解Solana程序的基本结构和入口点

## 前置知识

这是教程的第一节，无需任何Solana相关的前置知识。但建议具备：
- 基本的Rust编程经验
- 了解区块链基本概念（如果有以太坊开发经验更佳）
- 熟悉命令行操作

## 环境要求

### 必需软件

1. **Rust** (推荐1.75+)
2. **Solana CLI** (本教程使用1.18版本)
3. **Node.js** (可选，后续章节需要)

### 安装步骤

#### 1. 安装Rust

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

#### 2. 安装Solana CLI

```bash
# macOS/Linux
sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"

# 将Solana添加到PATH（根据安装提示操作）
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 验证安装
solana --version
```

#### 3. 配置Solana CLI

```bash
# 设置为本地测试网（localhost）
solana config set --url localhost

# 查看当前配置
solana config get
```

## 运行步骤

### 1. 克隆或创建项目

```bash
cd 01-environment-setup
```

### 2. 构建项目

```bash
cargo build-bpf
```

### 3. 运行测试

```bash
# 运行所有测试
cargo test

# 查看详细日志
cargo test -- --nocapture
```

### 4. 预期输出

测试成功后，你应该看到类似输出：

```
running 2 tests
test test_hello_solana ... ok
test test_with_accounts ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

在详细日志中，你会看到程序打印的消息：

```
Program log: Hello, Solana!
Program log: 程序ID: [程序地址]
Program log: 账户数量: 0
```

## 核心概念简要说明

### 程序入口点（Entrypoint）

Solana程序使用`entrypoint!`宏定义入口点：

```rust
entrypoint!(process_instruction);
```

### 处理函数签名

所有Solana程序的处理函数都遵循相同的签名：

```rust
pub fn process_instruction(
    program_id: &Pubkey,      // 程序的地址
    accounts: &[AccountInfo], // 传入的账户列表
    instruction_data: &[u8],  // 指令数据
) -> ProgramResult {
    // 程序逻辑
    Ok(())
}
```

### 日志输出

使用`msg!`宏在链上打印日志：

```rust
msg!("Hello, Solana!");
```

### 测试框架

本节使用`solana-program-test`进行测试：
- 创建本地测试环境
- 模拟交易处理
- 验证程序行为

## 常见问题FAQ

### Q1: 为什么选择Rust？

Solana程序必须使用Rust编写（也支持C/C++，但Rust是主流）。Rust提供：
- 内存安全
- 高性能
- 丰富的工具链

### Q2: 本地测试网 vs Devnet vs Mainnet？

- **本地测试网（localhost）**: 完全在本地运行，速度快，免费
- **Devnet**: Solana官方测试网，可获取免费测试代币
- **Mainnet**: 主网，使用真实SOL代币

本教程主要使用本地测试网和Rust测试框架。

### Q3: 测试失败怎么办？

常见问题：
1. 确保Rust版本 >= 1.75
2. 检查Cargo.toml中的依赖版本
3. 运行`cargo clean`后重新构建
4. 查看详细错误信息：`cargo test -- --nocapture`

### Q4: 为什么使用Rust测试而不是TypeScript？

前8节使用Rust测试（solana-program-test）的原因：
- 更接近程序本身的语言
- 更快的测试执行速度
- 更好的类型安全
- 第9节引入Anchor后会切换到TypeScript测试

### Q5: `msg!`宏的日志在哪里查看？

- 在测试中：使用`cargo test -- --nocapture`
- 在本地测试网：查看`solana-test-validator`的输出
- 在Devnet/Mainnet：使用Solana Explorer查看交易日志

## 下一步

完成本节后，你已经：
- ✅ 配置好Solana开发环境
- ✅ 创建并运行了第一个程序
- ✅ 理解了程序的基本结构
- ✅ 学会了编写和运行测试

继续学习 **第02节 - Solana基础概念**，深入理解Solana的账户模型和运行时。

## 参考资料

- [Solana官方文档](https://docs.solana.com/)
- [Rust官方教程](https://doc.rust-lang.org/book/)
- [solana-program-test文档](https://docs.rs/solana-program-test/)

