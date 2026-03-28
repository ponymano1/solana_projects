//! # 集成测试
//!
//! 测试环境搭建验证程序的基本功能

use solana_program_test::*;
use solana_sdk::{signature::Signer, transaction::Transaction};

/// 测试程序能否成功部署和调用
#[tokio::test]
async fn test_hello_solana() {
    // 创建程序测试环境
    let program_id = solana_program::pubkey::Pubkey::new_unique();
    let program_test = ProgramTest::new(
        "environment_setup",
        program_id,
        processor!(environment_setup::process_instruction),
    );

    // 启动测试环境
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 创建并发送交易
    let mut transaction = Transaction::new_with_payer(
        &[solana_sdk::instruction::Instruction {
            program_id,
            accounts: vec![],
            data: vec![],
        }],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    // 处理交易并验证成功
    banks_client.process_transaction(transaction).await.unwrap();
}

/// 测试程序能够接收账户信息
#[tokio::test]
async fn test_with_accounts() {
    // 创建程序测试环境
    let program_id = solana_program::pubkey::Pubkey::new_unique();
    let program_test = ProgramTest::new(
        "environment_setup",
        program_id,
        processor!(environment_setup::process_instruction),
    );

    // 启动测试环境
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 创建一个测试账户
    let test_account = solana_sdk::pubkey::Pubkey::new_unique();

    // 创建包含账户的交易
    let mut transaction = Transaction::new_with_payer(
        &[solana_sdk::instruction::Instruction {
            program_id,
            accounts: vec![solana_sdk::instruction::AccountMeta::new(
                test_account,
                false,
            )],
            data: vec![],
        }],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    // 处理交易并验证成功
    banks_client.process_transaction(transaction).await.unwrap();
}
