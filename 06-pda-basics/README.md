# 第06节 - PDA基础

## 学习目标

- 理解PDA（Program Derived Address）的概念和作用
- 学习如何使用find_program_address生成PDA
- 理解bump seed的作用
- 掌握使用invoke_signed创建PDA账户
- 实现基于PDA的投票系统

## 前置知识

- 完成第01-05节
- 理解账户模型和指令处理

## 项目概述

本项目实现了一个简单的投票系统，演示PDA的核心用法：
- 使用PDA存储投票主题
- 使用PDA记录用户投票
- 防止重复投票

## 运行步骤

### 1. 编译程序

```bash
cd 06-pda-basics
cargo build-bpf
```

### 2. 运行测试

```bash
cargo test
```

测试包括：
- `test_create_vote_topic` - 测试创建投票主题
- `test_vote` - 测试投票功能
- `test_cannot_vote_twice` - 测试防止重复投票

## 核心概念

### 1. 什么是PDA？

PDA（Program Derived Address）是由程序ID和种子（seeds）派生出的地址，它不在Ed25519曲线上，因此没有对应的私钥。

### 2. PDA的优势

- **确定性**：相同的种子总是生成相同的PDA
- **无需私钥**：程序可以代表PDA签名
- **唯一性**：可以为每个用户/资源生成唯一的PDA

### 3. find_program_address

```rust
let (pda, bump) = Pubkey::find_program_address(
    &[b"vote_topic", creator.as_ref()],
    program_id,
);
```

### 4. invoke_signed

使用PDA作为签名者创建账户：

```rust
invoke_signed(
    &system_instruction::create_account(...),
    &[...],
    &[&[b"vote_topic", creator.as_ref(), &[bump]]],
)?;
```

## 常见问题

### Q1: PDA和普通地址有什么区别？

A: PDA没有私钥，只能由程序代表它签名。普通地址有私钥，可以由持有私钥的人签名。

### Q2: bump seed是什么？

A: bump seed是一个u8值，用于确保派生的地址不在Ed25519曲线上。find_program_address会找到第一个有效的bump值。

### Q3: 为什么要验证PDA地址？

A: 防止恶意用户传入错误的账户地址，确保程序操作的是正确的PDA账户。

### Q4: 可以有多个PDA吗？

A: 可以。通过使用不同的种子，可以为不同的用途派生不同的PDA。

## 下一步

完成本节后，继续学习：
- 第07节：跨程序调用（CPI）
- 第08节：测试与调试

## 参考资源

- [Solana PDA文档](https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses)
- [Solana Cookbook - PDA](https://solanacookbook.com/core-concepts/pdas.html)
