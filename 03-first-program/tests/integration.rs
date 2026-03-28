use borsh::BorshSerialize;
use first_program::{instruction::CounterInstruction, state::Counter};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

/// 测试辅助函数：创建初始化指令
fn create_initialize_instruction(program_id: &Pubkey, counter_account: &Pubkey) -> Instruction {
    let data = CounterInstruction::Initialize.try_to_vec().unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![AccountMeta::new(*counter_account, false)],
        data,
    }
}

/// 测试辅助函数：创建增加指令
fn create_increment_instruction(program_id: &Pubkey, counter_account: &Pubkey) -> Instruction {
    let data = CounterInstruction::Increment.try_to_vec().unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![AccountMeta::new(*counter_account, false)],
        data,
    }
}

/// 测试辅助函数：创建减少指令
fn create_decrement_instruction(program_id: &Pubkey, counter_account: &Pubkey) -> Instruction {
    let data = CounterInstruction::Decrement.try_to_vec().unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![AccountMeta::new(*counter_account, false)],
        data,
    }
}

/// 测试辅助函数：创建重置指令
fn create_reset_instruction(program_id: &Pubkey, counter_account: &Pubkey) -> Instruction {
    let data = CounterInstruction::Reset.try_to_vec().unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![AccountMeta::new(*counter_account, false)],
        data,
    }
}

#[tokio::test]
async fn test_initialize_counter() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "first_program",
        program_id,
        processor!(first_program::process_instruction),
    );

    // 创建计数器账户
    let counter_account = Keypair::new();
    program_test.add_account(
        counter_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: vec![0; 9], // Counter结构需要9字节
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 创建并发送初始化指令
    let instruction = create_initialize_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    // 执行交易
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证账户数据
    let account = banks_client
        .get_account(counter_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let counter: Counter = borsh::BorshDeserialize::try_from_slice(&account.data).unwrap();
    assert_eq!(counter.count, 0);
    assert!(counter.is_initialized);
}

#[tokio::test]
async fn test_increment_counter() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "first_program",
        program_id,
        processor!(first_program::process_instruction),
    );

    // 创建计数器账户
    let counter_account = Keypair::new();
    program_test.add_account(
        counter_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: vec![0; 9],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 先初始化
    let init_instruction = create_initialize_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 增加计数
    let increment_instruction =
        create_increment_instruction(&program_id, &counter_account.pubkey());
    let mut transaction =
        Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证计数值
    let account = banks_client
        .get_account(counter_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let counter: Counter = borsh::BorshDeserialize::try_from_slice(&account.data).unwrap();
    assert_eq!(counter.count, 1);
}

#[tokio::test]
async fn test_decrement_counter() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "first_program",
        program_id,
        processor!(first_program::process_instruction),
    );

    let counter_account = Keypair::new();
    program_test.add_account(
        counter_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: vec![0; 9],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 初始化
    let init_instruction = create_initialize_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 先增加到5（在一个交易中执行5次）
    let increment_instruction1 = create_increment_instruction(&program_id, &counter_account.pubkey());
    let increment_instruction2 = create_increment_instruction(&program_id, &counter_account.pubkey());
    let increment_instruction3 = create_increment_instruction(&program_id, &counter_account.pubkey());
    let increment_instruction4 = create_increment_instruction(&program_id, &counter_account.pubkey());
    let increment_instruction5 = create_increment_instruction(&program_id, &counter_account.pubkey());

    let mut transaction = Transaction::new_with_payer(
        &[increment_instruction1, increment_instruction2, increment_instruction3,
          increment_instruction4, increment_instruction5],
        Some(&payer.pubkey())
    );
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 减少计数
    let decrement_instruction = create_decrement_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[decrement_instruction], Some(&payer.pubkey()));
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证计数值
    let account = banks_client
        .get_account(counter_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let counter: Counter = borsh::BorshDeserialize::try_from_slice(&account.data).unwrap();
    assert_eq!(counter.count, 4);
}

#[tokio::test]
async fn test_reset_counter() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "first_program",
        program_id,
        processor!(first_program::process_instruction),
    );

    let counter_account = Keypair::new();
    program_test.add_account(
        counter_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: vec![0; 9],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 初始化
    let init_instruction = create_initialize_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 增加3次
    let increment_instruction = create_increment_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let increment_instruction = create_increment_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let increment_instruction = create_increment_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 重置计数
    let reset_instruction = create_reset_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[reset_instruction], Some(&payer.pubkey()));
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证计数值
    let account = banks_client
        .get_account(counter_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let counter: Counter = borsh::BorshDeserialize::try_from_slice(&account.data).unwrap();
    assert_eq!(counter.count, 0);
    assert!(counter.is_initialized);
}

#[tokio::test]
async fn test_uninitialized_account_error() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "first_program",
        program_id,
        processor!(first_program::process_instruction),
    );

    let counter_account = Keypair::new();
    program_test.add_account(
        counter_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: vec![0; 9],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 尝试在未初始化的账户上增加计数（应该失败）
    let increment_instruction =
        create_increment_instruction(&program_id, &counter_account.pubkey());
    let mut transaction =
        Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ownership_check() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "first_program",
        program_id,
        processor!(first_program::process_instruction),
    );

    let counter_account = Keypair::new();
    let wrong_owner = Pubkey::new_unique();

    // 创建一个不属于程序的账户
    program_test.add_account(
        counter_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: vec![0; 9],
            owner: wrong_owner, // 错误的所有者
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 尝试初始化（应该失败）
    let init_instruction = create_initialize_instruction(&program_id, &counter_account.pubkey());
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}
