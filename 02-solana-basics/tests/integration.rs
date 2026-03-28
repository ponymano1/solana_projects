use solana_basics::{Account, Lamports, Transaction};
use solana_sdk::pubkey::Pubkey;

/// 测试用例1: 账户创建和基本属性
#[test]
fn test_account_creation() {
    let owner = Pubkey::new_unique();
    let account = Account::new(owner, 1000);

    assert_eq!(account.owner(), &owner);
    assert_eq!(account.lamports(), 1000);
    assert!(account.data().is_empty());
    assert!(!account.is_executable());
}

/// 测试用例2: Lamports转换和计算
#[test]
fn test_lamports_conversion() {
    // 1 SOL = 1,000,000,000 lamports
    let one_sol = Lamports::from_sol(1.0);
    assert_eq!(one_sol.as_lamports(), 1_000_000_000);

    let half_sol = Lamports::from_sol(0.5);
    assert_eq!(half_sol.as_lamports(), 500_000_000);

    // 测试加法
    let total = one_sol.add(half_sol);
    assert_eq!(total.as_lamports(), 1_500_000_000);

    // 测试减法
    let remaining = one_sol.sub(half_sol).unwrap();
    assert_eq!(remaining.as_lamports(), 500_000_000);
}

/// 测试用例3: 交易创建和签名验证
#[test]
fn test_transaction_creation() {
    let payer = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();

    let tx = Transaction::new_transfer(payer, recipient, 1000);

    assert_eq!(tx.payer(), &payer);
    assert_eq!(tx.instructions().len(), 1);
    assert!(!tx.is_signed());
}

/// 测试用例4: 账户数据存储
#[test]
fn test_account_data_storage() {
    let owner = Pubkey::new_unique();
    let mut account = Account::new(owner, 5000);

    let data = vec![1, 2, 3, 4, 5];
    account.set_data(data.clone());

    assert_eq!(account.data(), &data);
    assert_eq!(account.data().len(), 5);
}

/// 测试用例5: Lamports不足时的错误处理
#[test]
fn test_lamports_insufficient_funds() {
    let small = Lamports::from_sol(0.1);
    let large = Lamports::from_sol(1.0);

    let result = small.sub(large);
    assert!(result.is_err());
}
