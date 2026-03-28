use borsh::BorshSerialize;
use instructions::{
    instruction::TodoInstruction,
    state::TodoList,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

/// 测试辅助函数：创建初始化指令
fn create_initialize_instruction(
    program_id: &Pubkey,
    owner: &Pubkey,
    todo_list_account: &Pubkey,
) -> Instruction {
    let data = TodoInstruction::Initialize.try_to_vec().unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*todo_list_account, false),
        ],
        data,
    }
}

/// 测试辅助函数：创建Todo指令
fn create_create_todo_instruction(
    program_id: &Pubkey,
    owner: &Pubkey,
    todo_list_account: &Pubkey,
    title: String,
    description: String,
) -> Instruction {
    let data = TodoInstruction::CreateTodo { title, description }
        .try_to_vec()
        .unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*todo_list_account, false),
        ],
        data,
    }
}

/// 测试辅助函数：创建更新Todo指令
fn create_update_todo_instruction(
    program_id: &Pubkey,
    owner: &Pubkey,
    todo_list_account: &Pubkey,
    id: u32,
    completed: bool,
) -> Instruction {
    let data = TodoInstruction::UpdateTodo { id, completed }
        .try_to_vec()
        .unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*todo_list_account, false),
        ],
        data,
    }
}

/// 测试辅助函数：创建删除Todo指令
fn create_delete_todo_instruction(
    program_id: &Pubkey,
    owner: &Pubkey,
    todo_list_account: &Pubkey,
    id: u32,
) -> Instruction {
    let data = TodoInstruction::DeleteTodo { id }
        .try_to_vec()
        .unwrap();
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*todo_list_account, false),
        ],
        data,
    }
}

#[tokio::test]
async fn test_initialize_todo_list() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "instructions",
        program_id,
        processor!(instructions::process_instruction),
    );

    // 创建Todo列表账户
    let todo_list_account = Keypair::new();
    program_test.add_account(
        todo_list_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 10_000_000,
            data: vec![0; TodoList::max_space()],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 创建并发送初始化指令
    let instruction = create_initialize_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
    );
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    // 执行交易
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证账户数据
    let account = banks_client
        .get_account(todo_list_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let todo_list: TodoList = {
        let mut data_slice = &account.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };
    assert!(todo_list.is_initialized);
    assert_eq!(todo_list.owner, payer.pubkey());
    assert_eq!(todo_list.todos.len(), 0);
    assert_eq!(todo_list.next_id, 0);
}

#[tokio::test]
async fn test_create_todo() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "instructions",
        program_id,
        processor!(instructions::process_instruction),
    );

    // 创建Todo列表账户
    let todo_list_account = Keypair::new();
    program_test.add_account(
        todo_list_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 10_000_000,
            data: vec![0; TodoList::max_space()],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 初始化Todo列表
    let init_instruction = create_initialize_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
    );
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 创建Todo
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let create_instruction = create_create_todo_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
        "学习Solana".to_string(),
        "完成第05节教程".to_string(),
    );
    let mut transaction = Transaction::new_with_payer(&[create_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证账户数据
    let account = banks_client
        .get_account(todo_list_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let todo_list: TodoList = {
        let mut data_slice = &account.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };
    assert_eq!(todo_list.todos.len(), 1);
    assert_eq!(todo_list.todos[0].id, 0);
    assert_eq!(todo_list.todos[0].title, "学习Solana");
    assert_eq!(todo_list.todos[0].description, "完成第05节教程");
    assert!(!todo_list.todos[0].completed);
    assert_eq!(todo_list.next_id, 1);
}

#[tokio::test]
async fn test_update_todo() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "instructions",
        program_id,
        processor!(instructions::process_instruction),
    );

    // 创建Todo列表账户
    let todo_list_account = Keypair::new();
    program_test.add_account(
        todo_list_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 10_000_000,
            data: vec![0; TodoList::max_space()],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 初始化并创建Todo
    let init_instruction = create_initialize_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
    );
    let create_instruction = create_create_todo_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
        "测试Todo".to_string(),
        "测试更新功能".to_string(),
    );
    let mut transaction = Transaction::new_with_payer(
        &[init_instruction, create_instruction],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 更新Todo状态
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let update_instruction = create_update_todo_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
        0,
        true,
    );
    let mut transaction = Transaction::new_with_payer(&[update_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证账户数据
    let account = banks_client
        .get_account(todo_list_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let todo_list: TodoList = {
        let mut data_slice = &account.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };
    assert_eq!(todo_list.todos.len(), 1);
    assert!(todo_list.todos[0].completed);
}

#[tokio::test]
async fn test_delete_todo() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "instructions",
        program_id,
        processor!(instructions::process_instruction),
    );

    // 创建Todo列表账户
    let todo_list_account = Keypair::new();
    program_test.add_account(
        todo_list_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 10_000_000,
            data: vec![0; TodoList::max_space()],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 初始化并创建两个Todo
    let init_instruction = create_initialize_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
    );
    let create1 = create_create_todo_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
        "Todo 1".to_string(),
        "第一个Todo".to_string(),
    );
    let create2 = create_create_todo_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
        "Todo 2".to_string(),
        "第二个Todo".to_string(),
    );
    let mut transaction = Transaction::new_with_payer(
        &[init_instruction, create1, create2],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 删除第一个Todo
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let delete_instruction = create_delete_todo_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
        0,
    );
    let mut transaction = Transaction::new_with_payer(&[delete_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 验证账户数据
    let account = banks_client
        .get_account(todo_list_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    let todo_list: TodoList = {
        let mut data_slice = &account.data[..];
        borsh::BorshDeserialize::deserialize(&mut data_slice).unwrap()
    };
    assert_eq!(todo_list.todos.len(), 1);
    assert_eq!(todo_list.todos[0].id, 1);
    assert_eq!(todo_list.todos[0].title, "Todo 2");
}

#[tokio::test]
async fn test_unauthorized_access() {
    // 创建程序测试环境
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "instructions",
        program_id,
        processor!(instructions::process_instruction),
    );

    // 创建Todo列表账户
    let todo_list_account = Keypair::new();
    program_test.add_account(
        todo_list_account.pubkey(),
        solana_sdk::account::Account {
            lamports: 10_000_000,
            data: vec![0; TodoList::max_space()],
            owner: program_id,
            ..Default::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 初始化Todo列表
    let init_instruction = create_initialize_instruction(
        &program_id,
        &payer.pubkey(),
        &todo_list_account.pubkey(),
    );
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // 尝试用另一个账户创建Todo（应该失败）
    let unauthorized_user = Keypair::new();
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let create_instruction = create_create_todo_instruction(
        &program_id,
        &unauthorized_user.pubkey(),
        &todo_list_account.pubkey(),
        "未授权Todo".to_string(),
        "这应该失败".to_string(),
    );
    let mut transaction =
        Transaction::new_with_payer(&[create_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &unauthorized_user], recent_blockhash);

    // 验证交易失败
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
}
