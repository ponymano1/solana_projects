# 第07节 - CPI基础

## 学习目标

- 理解CPI（Cross-Program Invocation）的概念
- 学习使用invoke调用其他程序
- 掌握invoke_signed与PDA结合使用
- 实现程序间交互
- 理解CPI的权限和安全性

## 前置知识

- 完成第01-06节
- 理解PDA的概念和使用

## 项目概述

本项目实现了一个转账记录器，演示CPI的两种用法：
- 使用invoke进行普通CPI调用
- 使用invoke_signed让PDA作为签名者

## 运行步骤

### 1. 编译程序

```bash
cd 07-cpi-basics
cargo build-bpf
```

### 2. 运行测试

```bash
cargo test
```

## 核心概念

### 1. 什么是CPI？

CPI（Cross-Program Invocation）允许一个程序调用另一个程序的指令。这是Solana程序组合性的基础。

### 2. invoke vs invoke_signed

**invoke** - 普通CPI调用：
```rust
invoke(
    &system_instruction::transfer(from, to, amount),
    &[from_account, to_account, system_program],
)?;
```

**invoke_signed** - 使用PDA签名的CPI调用：
```rust
invoke_signed(
    &system_instruction::transfer(pda, to, amount),
    &[pda_account, to_account, system_program],
    &[&[b"vault", &[bump]]],
)?;
```

### 3. CPI的权限传递

- 调用程序的签名者权限会传递给被调用程序
- PDA可以通过invoke_signed作为签名者
- 被调用程序可以修改传入的可写账户

### 4. 常见的CPI场景

- 调用System Program进行转账
- 调用Token Program进行代币操作
- 调用其他自定义程序

## 常见问题

### Q1: CPI和以太坊的合约调用有什么区别？

A: Solana的CPI需要显式传递所有账户，而以太坊的调用是隐式的。Solana的方式更安全，因为所有依赖都是明确的。

### Q2: CPI有深度限制吗？

A: 是的，CPI调用深度限制为4层。这是为了防止无限递归和栈溢出。

### Q3: 为什么需要invoke_signed？

A: 因为PDA没有私钥，无法直接签名。invoke_signed允许程序代表PDA签名，只要提供正确的种子。

### Q4: CPI调用会消耗多少计算单元？

A: CPI本身消耗较少，主要消耗来自被调用程序的执行。每次CPI调用大约消耗1000个计算单元。

## 下一步

完成本节后，继续学习：
- 第08节：测试与调试

## 参考资源

- [Solana CPI文档](https://docs.solana.com/developing/programming-model/calling-between-programs)
- [Solana Cookbook - CPI](https://solanacookbook.com/references/programs.html#how-to-do-cross-program-invocation)
