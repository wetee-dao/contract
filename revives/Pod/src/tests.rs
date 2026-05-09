//! Unit tests for Pod contract. Uses off_chain Engine (wrevive_api::with_engine).

use super::*;
use wrevive_api::{Address, H256, U256, with_engine};

fn cloud_caller() -> [u8; 20] {
    [1u8; 20]
}

#[test]
fn deploy_and_getters() {
    let id = 1u64;
    let owner = Address::from([2u8; 20]);
    let side_chain = Address::from([3u8; 20]);
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let _ = pod::initialize(id, owner, side_chain);
    assert_eq!(pod::id(), 1);
    assert_eq!(pod::owner(), owner);
    assert_eq!(pod::cloud(), Address::from(cloud_caller()));
}

#[test]
fn account_id_off_chain() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let _ = pod::initialize(1, Address::from([2u8; 20]), Address::zero());
    assert_eq!(pod::account_id(), Address::zero());
}

#[test]
fn withdraw_as_owner_insufficient_balance() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let owner = Address::from([2u8; 20]);
    let _ = pod::new();
    let _ = pod::initialize(1, owner, Address::zero());
    // 标记已结算，否则 withdraw 会返回 NotSettled
    let _ = pod::mark_settled();
    with_engine(|e| e.set_caller([2u8; 20]));
    let res = pod::withdraw(
        AssetInfo::Native(Default::default()),
        Address::from([9u8; 20]),
        U256::from(100u64),
    );
    assert_eq!(res, Err(Error::InsufficientBalance));
}

#[test]
fn withdraw_as_non_owner_reverts() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let owner = Address::from([2u8; 20]);
    let _ = pod::new();
    let _ = pod::initialize(1, owner, Address::zero());
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = pod::withdraw(
        AssetInfo::Native(Default::default()),
        Address::from([9u8; 20]),
        U256::from(0u64),
    );
    assert_eq!(res, Err(Error::NotOwner));
}

#[test]
fn pay_for_worker_only_by_cloud() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let _ = pod::initialize(1, Address::from([2u8; 20]), Address::zero());
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = pod::pay_for_worker(
        Address::from([5u8; 20]),
        AssetInfo::Native(Default::default()),
        U256::from(1u64),
    );
    assert_eq!(res, Err(Error::MustCallByCloudContract));
}

#[test]
fn set_code_returns_err_when_upgrade_needed() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let _ = pod::initialize(1, Address::from([2u8; 20]), Address::zero());
    // 链下 code_hash 恒为 0；传入非零哈希表示"要升级到别的代码"，当前无 host 则失败
    let res = pod::set_code(H256::from([1u8; 32]));
    assert_eq!(res, Err(Error::CodeUpgradeNotSupported));
}

#[test]
fn set_code_noop_when_hash_matches_current() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let _ = pod::initialize(1, Address::from([2u8; 20]), Address::zero());
    let res = pod::set_code(H256::zero());
    assert_eq!(res, Ok(()));
}

#[test]
fn set_code_non_cloud_reverts() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let _ = pod::initialize(1, Address::from([2u8; 20]), Address::zero());
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = pod::set_code(H256::from([1u8; 32]));
    assert_eq!(res, Err(Error::MustCallByCloudContract));
}

#[test]
fn initialize_twice_fails() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let owner = Address::from([2u8; 20]);
    let side_chain = Address::from([3u8; 20]);
    let _ = pod::initialize(1, owner, side_chain);
    // 第二次调用应该失败
    let res = pod::initialize(2, owner, side_chain);
    assert_eq!(res, Err(Error::AlreadyInitialized));
}

#[test]
fn mark_settled_only_by_cloud() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new();
    let _ = pod::initialize(1, Address::from([2u8; 20]), Address::zero());
    // Cloud 调用 mark_settled 成功
    let res = pod::mark_settled();
    assert_eq!(res, Ok(()));
    // 非 Cloud 调用失败
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = pod::mark_settled();
    assert_eq!(res, Err(Error::MustCallByCloudContract));
}

#[test]
fn withdraw_before_settled_fails_with_not_settled() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let owner = Address::from([2u8; 20]);
    let _ = pod::new();
    let _ = pod::initialize(1, owner, Address::zero());
    // 切换到 owner，但未调用 mark_settled，withdraw 应返回 NotSettled
    with_engine(|e| e.set_caller([2u8; 20]));
    let res = pod::withdraw(
        AssetInfo::Native(Default::default()),
        Address::from([9u8; 20]),
        U256::from(1u64),
    );
    assert_eq!(res, Err(Error::NotSettled));
}
