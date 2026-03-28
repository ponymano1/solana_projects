use borsh::BorshSerialize;
use cpi_basics::{instruction::TransferInstruction, state::TransferRecord};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_transfer_with_record() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "cpi_basics",
        program_id,
        processor!(cpi_basics::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 创建接收者并给予初始余额（租金豁免）
    let receiver = Keypair::new();
    let create_receiver_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &receiver.pubkey(),
        1_000_000, // 给接收者一些初始余额
        0,
        &system_program::id(),
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_receiver_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &receiver], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 创建转账记录账户
    let record_account = Keypair::new();
    let create_record_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &record_account.pubkey(),
        10_000_000, // 增加租金
        TransferRecord::space() as u64,
        &program_id,
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[create_record_account_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &record_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 获取payer初始余额
    let payer_balance_before = banks_client
        .get_balance(payer.pubkey())
        .await
        .unwrap();

    // 执行转账
    let transfer_amount = 1_000_000u64;
    let instruction_data = TransferInstruction::TransferWithRecord {
        amount: transfer_amount,
    }
    .try_to_vec()
    .unwrap();

    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(receiver.pubkey(), false),
            AccountMeta::new(record_account.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: instruction_data,
    };

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证余额变化
    let receiver_balance = banks_client
        .get_balance(receiver.pubkey())
        .await
        .unwrap();
    assert_eq!(receiver_balance, 1_000_000 + transfer_amount); // 初始余额 + 转账金额

    // 验证转账记录
    let record_account_data = banks_client
        .get_account(record_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let transfer_record: TransferRecord = {
        let mut data_slice = &record_account_data.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };

    assert!(transfer_record.is_initialized);
    assert_eq!(transfer_record.from, payer.pubkey());
    assert_eq!(transfer_record.to, receiver.pubkey());
    assert_eq!(transfer_record.amount, transfer_amount);
}

#[tokio::test]
async fn test_transfer_from_pda() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "cpi_basics",
        program_id,
        processor!(cpi_basics::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 派生PDA
    let (pda, bump) = Pubkey::find_program_address(&[b"transfer_vault"], &program_id);

    // 给PDA账户充值
    let fund_pda_ix = solana_program::system_instruction::transfer(
        &payer.pubkey(),
        &pda,
        10_000_000,
    );

    let mut transaction = Transaction::new_with_payer(&[fund_pda_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 创建接收者并给予初始余额（租金豁免）
    let receiver = Keypair::new();
    let create_receiver_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &receiver.pubkey(),
        1_000_000, // 给接收者一些初始余额
        0,
        &system_program::id(),
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_receiver_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &receiver], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 创建转账记录账户
    let record_account = Keypair::new();
    let create_record_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &record_account.pubkey(),
        10_000_000, // 增加租金
        TransferRecord::space() as u64,
        &program_id,
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[create_record_account_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &record_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 执行从PDA转账
    let transfer_amount = 2_000_000u64;
    let instruction_data = TransferInstruction::TransferFromPDA {
        amount: transfer_amount,
        bump,
    }
    .try_to_vec()
    .unwrap();

    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pda, false),
            AccountMeta::new(receiver.pubkey(), false),
            AccountMeta::new(record_account.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: instruction_data,
    };

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证余额变化
    let receiver_balance = banks_client
        .get_balance(receiver.pubkey())
        .await
        .unwrap();
    assert_eq!(receiver_balance, 1_000_000 + transfer_amount); // 初始余额 + 转账金额

    // 验证转账记录
    let record_account_data = banks_client
        .get_account(record_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let transfer_record: TransferRecord = {
        let mut data_slice = &record_account_data.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };

    assert!(transfer_record.is_initialized);
    assert_eq!(transfer_record.from, pda);
    assert_eq!(transfer_record.to, receiver.pubkey());
    assert_eq!(transfer_record.amount, transfer_amount);
}
