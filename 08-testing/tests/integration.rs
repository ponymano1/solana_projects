use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use testing::{instruction::CalculatorInstruction, state::CalculatorResult};

/// 辅助函数：创建计算器指令
fn create_calculator_instruction(
    program_id: &Pubkey,
    result_account: &Pubkey,
    instruction: CalculatorInstruction,
) -> Instruction {
    let data = instruction.try_to_vec().unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![AccountMeta::new(*result_account, false)],
        data,
    }
}

#[tokio::test]
async fn test_add() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "testing",
        program_id,
        processor!(testing::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 创建结果账户
    let result_account = Keypair::new();
    program_test = ProgramTest::new(
        "testing",
        program_id,
        processor!(testing::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let result_account = Keypair::new();
    let create_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &result_account.pubkey(),
        10_000_000,
        CalculatorResult::space() as u64,
        &program_id,
    );

    let mut transaction = Transaction::new_with_payer(&[create_account_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &result_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 测试加法
    let add_ix = create_calculator_instruction(
        &program_id,
        &result_account.pubkey(),
        CalculatorInstruction::Add { a: 10, b: 20 },
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(&[add_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证结果
    let account = banks_client
        .get_account(result_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let result: CalculatorResult = {
        let mut data_slice = &account.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };

    assert_eq!(result.result, 30);
    assert_eq!(result.operation_count, 1);
}

#[tokio::test]
async fn test_multiple_operations() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "testing",
        program_id,
        processor!(testing::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let result_account = Keypair::new();
    let create_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &result_account.pubkey(),
        10_000_000,
        CalculatorResult::space() as u64,
        &program_id,
    );

    let mut transaction = Transaction::new_with_payer(&[create_account_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &result_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 执行多个操作
    let operations = vec![
        CalculatorInstruction::Add { a: 10, b: 5 },      // 15
        CalculatorInstruction::Multiply { a: 15, b: 2 }, // 30
        CalculatorInstruction::Subtract { a: 30, b: 5 }, // 25
        CalculatorInstruction::Divide { a: 25, b: 5 },   // 5
    ];

    for op in operations {
        let ix = create_calculator_instruction(&program_id, &result_account.pubkey(), op);
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    // 验证最终结果
    let account = banks_client
        .get_account(result_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let result: CalculatorResult = {
        let mut data_slice = &account.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };

    assert_eq!(result.result, 5);
    assert_eq!(result.operation_count, 4);
}

#[tokio::test]
async fn test_division_by_zero() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "testing",
        program_id,
        processor!(testing::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let result_account = Keypair::new();
    let create_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &result_account.pubkey(),
        10_000_000,
        CalculatorResult::space() as u64,
        &program_id,
    );

    let mut transaction = Transaction::new_with_payer(&[create_account_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &result_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 测试除以零
    let divide_ix = create_calculator_instruction(
        &program_id,
        &result_account.pubkey(),
        CalculatorInstruction::Divide { a: 10, b: 0 },
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(&[divide_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    // 验证交易失败
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_overflow() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "testing",
        program_id,
        processor!(testing::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let result_account = Keypair::new();
    let create_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &result_account.pubkey(),
        10_000_000,
        CalculatorResult::space() as u64,
        &program_id,
    );

    let mut transaction = Transaction::new_with_payer(&[create_account_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &result_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 测试溢出
    let add_ix = create_calculator_instruction(
        &program_id,
        &result_account.pubkey(),
        CalculatorInstruction::Add {
            a: i64::MAX,
            b: 1,
        },
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(&[add_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    // 验证交易失败
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}
