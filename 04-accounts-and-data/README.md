# 第04节 - 账户与数据存储

本节学习Solana中的账户创建、数据存储和账户生命周期管理。

## 学习目标

- 理解Solana账户模型和数据存储机制
- 学会使用System Program创建新账户
- 掌握租金计算和账户空间分配
- 实现完整的数据CRUD操作（创建、读取、更新、删除）
- 理解账户所有权和访问控制

## 前置知识

- 完成第01节：环境搭建
- 完成第02节：Solana基础概念
- 完成第03节：第一个原生程序
- 熟悉Rust基础语法

## 项目概述

本项目实现了一个用户配置文件（UserProfile）系统，演示了：
- 创建新账户并初始化数据
- 更新账户数据
- 关闭账户并返还租金
- 所有权验证和访问控制
- 数据验证和错误处理

## 运行步骤

### 1. 编译程序

```bash
cd 04-accounts-and-data
cargo build-bpf
```

### 2. 运行测试

```bash
cargo test
```

测试包括：
- `test_create_profile` - 测试创建用户配置文件
- `test_update_profile` - 测试更新配置文件
- `test_close_profile` - 测试关闭配置文件并返还租金
- `test_rent_calculation` - 测试租金计算
- `test_ownership_check` - 测试所有权验证
- `test_data_validation` - 测试数据验证

### 3. 查看测试输出

```bash
cargo test -- --nocapture
```

## 核心概念

### 账户创建流程

1. 计算所需空间
2. 计算租金豁免所需的lamports
3. 调用System Program创建账户
4. 转移所有权给程序
5. 初始化账户数据

### 租金机制

Solana使用租金机制防止状态膨胀：
- 账户必须保持足够余额以豁免租金
- 租金豁免余额 = 每字节租金 × 账户大小
- 关闭账户时可以回收租金

### 数据布局

UserProfile结构使用固定大小布局：
- `is_initialized`: 1字节
- `owner`: 32字节（Pubkey）
- `name`: 32字节（固定数组）
- `name_len`: 1字节
- `age`: 1字节
- `email`: 64字节（固定数组）
- `email_len`: 1字节
- 总计：132字节

### 账户关闭

关闭账户的步骤：
1. 验证所有者权限
2. 将账户余额转移到接收者
3. 将账户余额设为0
4. 清空账户数据

## 常见问题

### Q: 为什么需要租金？
A: 租金机制防止区块链状态无限增长。账户必须保持足够余额才能存活，否则会被清理。

### Q: 如何计算租金？
A: 使用 `Rent::default().minimum_balance(space)` 计算租金豁免所需的最小余额。

### Q: 为什么使用固定大小数组而不是String？
A: 固定大小数组确保账户空间可预测，简化空间计算和数据序列化。

### Q: 账户关闭后租金去哪了？
A: 租金返还给指定的接收者账户（通常是所有者）。

### Q: 如何防止未授权访问？
A: 在每个操作中验证签名者身份和账户所有权。

## 下一步

完成本节后，继续学习：
- 第05节：指令处理
- 第06节：PDA（程序派生地址）
- 第07节：跨程序调用（CPI）

## 参考资料

- [Solana账户模型](https://docs.solana.com/developing/programming-model/accounts)
- [租金机制](https://docs.solana.com/developing/programming-model/accounts#rent)
- [System Program](https://docs.solana.com/developing/runtime-facilities/programs#system-program)
