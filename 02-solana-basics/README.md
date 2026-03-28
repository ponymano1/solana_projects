# 第02节 - Solana基础概念

本节介绍Solana区块链的核心概念，包括账户、Lamports和交易。

## 学习目标

- 理解Solana账户模型
- 掌握Lamports的概念和转换
- 了解交易的基本结构
- 学习如何在账户之间转移资产

## 核心概念

### 1. Account（账户）

账户是Solana中存储数据和SOL的基本单位。每个账户包含：

- **owner**：账户所有者（程序ID）
- **lamports**：账户余额
- **data**：账户数据
- **executable**：是否可执行

### 2. Lamports

Lamports是SOL的最小单位：
- 1 SOL = 1,000,000,000 lamports
- 类似于比特币的satoshi或以太坊的wei

### 3. Transaction（交易）

交易用于修改区块链状态，包含：
- 付款人（payer）
- 一个或多个指令（instructions）
- 签名信息

## 运行测试

```bash
cargo test
```

## 代码格式化

```bash
cargo fmt
cargo clippy
```

## 相关文档

- [CONCEPTS.md](docs/CONCEPTS.md) - 详细概念说明
- [EXERCISES.md](docs/EXERCISES.md) - 练习题
