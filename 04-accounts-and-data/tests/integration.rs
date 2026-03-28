use accounts_and_data::{
    instruction::ProfileInstruction, processor::process_instruction, state::UserProfile,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// 使用borsh进行序列化和反序列化
use borsh::{BorshDeserialize, BorshSerialize};

// 辅助函数：创建指令
fn create_instruction(
    program_id: Pubkey,
    instruction: &ProfileInstruction,
    accounts: Vec<AccountMeta>,
) -> Instruction {
    Instruction {
        program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

#[tokio::test]
async fn test_create_profile() {
    // 创建测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "accounts_and_data",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 创建新账户密钥对
    let profile_account = Keypair::new();

    // 准备创建配置文件的指令数据
    let name = "张三".to_string();
    let age = 25u8;
    let email = "zhangsan@example.com".to_string();

    let instruction_data = ProfileInstruction::CreateProfile {
        name: name.clone(),
        age,
        email: email.clone(),
    };

    // 计算所需空间和租金
    let space = UserProfile::space();
    let rent = Rent::default();
    let lamports = rent.minimum_balance(space);

    // 创建账户指令
    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &profile_account.pubkey(),
        lamports,
        space as u64,
        &program_id,
    );

    // 创建配置文件指令
    let create_profile_ix = create_instruction(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    );

    // 发送交易
    let mut transaction = Transaction::new_with_payer(
        &[create_account_ix, create_profile_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &profile_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证账户数据
    let account = banks_client
        .get_account(profile_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let profile = UserProfile::try_from_slice(&account.data).unwrap();
    assert!(profile.is_initialized);
    assert_eq!(profile.owner, payer.pubkey());
    assert_eq!(profile.get_name(), name);
    assert_eq!(profile.age, age);
    assert_eq!(profile.get_email(), email);
}

#[tokio::test]
async fn test_update_profile() {
    // 创建测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "accounts_and_data",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    let profile_account = Keypair::new();

    // 先创建配置文件
    let space = UserProfile::space();
    let rent = Rent::default();
    let lamports = rent.minimum_balance(space);

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &profile_account.pubkey(),
        lamports,
        space as u64,
        &program_id,
    );

    let create_ix = create_instruction(
        program_id,
        &ProfileInstruction::CreateProfile {
            name: "李四".to_string(),
            age: 30,
            email: "lisi@example.com".to_string(),
        },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[create_account_ix, create_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &profile_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 更新配置文件
    let update_ix = create_instruction(
        program_id,
        &ProfileInstruction::UpdateProfile {
            name: Some("李四更新".to_string()),
            age: Some(31),
            email: None,
        },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), false),
        ],
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(&[update_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证更新后的数据
    let account = banks_client
        .get_account(profile_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let profile = UserProfile::try_from_slice(&account.data).unwrap();
    assert_eq!(profile.get_name(), "李四更新");
    assert_eq!(profile.age, 31);
    assert_eq!(profile.get_email(), "lisi@example.com"); // 未更新，保持原值
}

#[tokio::test]
async fn test_close_profile() {
    // 创建测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "accounts_and_data",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    let profile_account = Keypair::new();

    // 先创建配置文件
    let space = UserProfile::space();
    let rent = Rent::default();
    let lamports = rent.minimum_balance(space);

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &profile_account.pubkey(),
        lamports,
        space as u64,
        &program_id,
    );

    let create_ix = create_instruction(
        program_id,
        &ProfileInstruction::CreateProfile {
            name: "王五".to_string(),
            age: 28,
            email: "wangwu@example.com".to_string(),
        },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[create_account_ix, create_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &profile_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 获取关闭前的余额
    let payer_balance_before = banks_client.get_balance(payer.pubkey()).await.unwrap();

    // 关闭配置文件
    let close_ix = create_instruction(
        program_id,
        &ProfileInstruction::CloseProfile,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), false),
        ],
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction = Transaction::new_with_payer(&[close_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证账户已关闭
    let account = banks_client
        .get_account(profile_account.pubkey())
        .await
        .unwrap();
    assert!(account.is_none());

    // 验证租金已返还（余额应该增加）
    let payer_balance_after = banks_client.get_balance(payer.pubkey()).await.unwrap();
    assert!(payer_balance_after > payer_balance_before);
}

#[tokio::test]
async fn test_rent_calculation() {
    // 测试租金计算是否正确
    let space = UserProfile::space();
    let rent = Rent::default();
    let lamports = rent.minimum_balance(space);

    // 验证租金豁免所需的最小余额
    assert!(lamports > 0);
    assert!(rent.is_exempt(lamports, space));
}

#[tokio::test]
async fn test_ownership_check() {
    // 测试只有owner可以更新配置文件
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "accounts_and_data",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    let profile_account = Keypair::new();
    let unauthorized_user = Keypair::new();

    // 先创建配置文件
    let space = UserProfile::space();
    let rent = Rent::default();
    let lamports = rent.minimum_balance(space);

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &profile_account.pubkey(),
        lamports,
        space as u64,
        &program_id,
    );

    let create_ix = create_instruction(
        program_id,
        &ProfileInstruction::CreateProfile {
            name: "赵六".to_string(),
            age: 35,
            email: "zhaoliu@example.com".to_string(),
        },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[create_account_ix, create_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &profile_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 尝试用未授权用户更新配置文件（应该失败）
    let update_ix = create_instruction(
        program_id,
        &ProfileInstruction::UpdateProfile {
            name: Some("未授权更新".to_string()),
            age: None,
            email: None,
        },
        vec![
            AccountMeta::new(unauthorized_user.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), false),
        ],
    );

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mut transaction =
        Transaction::new_with_payer(&[update_ix], Some(&unauthorized_user.pubkey()));
    transaction.sign(&[&unauthorized_user], recent_blockhash);

    // 这个交易应该失败
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_data_validation() {
    // 测试数据验证（名字和邮箱长度限制）
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "accounts_and_data",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    let profile_account = Keypair::new();

    // 创建一个名字过长的配置文件
    let long_name = "a".repeat(UserProfile::MAX_NAME_LEN + 1);

    let space = UserProfile::space();
    let rent = Rent::default();
    let lamports = rent.minimum_balance(space);

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &profile_account.pubkey(),
        lamports,
        space as u64,
        &program_id,
    );

    let create_ix = create_instruction(
        program_id,
        &ProfileInstruction::CreateProfile {
            name: long_name,
            age: 25,
            email: "test@example.com".to_string(),
        },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(profile_account.pubkey(), true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[create_account_ix, create_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &profile_account], recent_blockhash);

    // 这个交易应该失败
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}
