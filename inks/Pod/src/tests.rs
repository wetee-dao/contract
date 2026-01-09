use ink::env::test::default_accounts;
use ink::AccountId;

use super::pod::*;

fn _init() -> Pod {
    let accounts = default_accounts();
    Pod::new(0, accounts.alice, accounts.alice)
}

#[ink::test]
fn test_new() {
    let accounts = default_accounts();
    let pod = Pod::new(1, accounts.bob, accounts.charlie);
    
    assert_eq!(pod.id(), 1);
    assert_eq!(pod.owner(), accounts.bob);
}

#[ink::test]
fn test_account_id() {
    let pod = _init();
    let account_id = pod.account_id();
    assert_ne!(account_id, AccountId::from([0u8; 32]));
}

#[ink::test]
fn test_charge() {
    let mut pod = _init();
    
    // 模拟充值
    ink::env::test::set_value_transferred(ink::primitives::U256::from(1000));
    pod.charge();
    
    // charge 方法只是接收转账，不返回任何内容
    // 在实际环境中，余额会通过环境自动处理
}

#[ink::test]
fn test_cloud() {
    let pod = _init();
    let cloud = pod.cloud();
    // cloud 返回创建 Pod 的调用者地址
    assert_ne!(cloud, AccountId::from([0u8; 32]));
}

#[ink::test]
fn test_id() {
    let pod = _init();
    assert_eq!(pod.id(), 0);
    
    let accounts = default_accounts();
    let pod2 = Pod::new(42, accounts.bob, accounts.charlie);
    assert_eq!(pod2.id(), 42);
}

#[ink::test]
fn test_owner() {
    let accounts = default_accounts();
    let pod = Pod::new(0, accounts.bob, accounts.charlie);
    assert_eq!(pod.owner(), accounts.bob);
}
