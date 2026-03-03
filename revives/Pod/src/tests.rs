//! Unit tests for Pod contract. Uses off_chain Engine (wrevive_api::with_engine).

use super::*;
use wrevive_api::{with_engine, Address, U256};

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
    let _ = pod::new(id, owner, side_chain);
    assert_eq!(pod::id(), 1);
    assert_eq!(pod::owner(), owner);
    assert_eq!(pod::cloud(), Address::from(cloud_caller()));
}

#[test]
fn charge_succeeds() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new(1, Address::from([2u8; 20]), Address::from([3u8; 20]));
    let res = pod::charge();
    assert!(res.is_ok());
}

#[test]
fn account_id_off_chain() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new(1, Address::from([2u8; 20]), Address::zero());
    assert_eq!(pod::account_id(), Address::zero());
}

#[test]
fn withdraw_as_owner_insufficient_balance() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let owner = Address::from([2u8; 20]);
    let _ = pod::new(1, owner, Address::zero());
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
    let _ = pod::new(1, owner, Address::zero());
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = pod::withdraw(
        AssetInfo::Native(Default::default()),
        Address::from([9u8; 20]),
        U256::from(0u64),
    );
    assert_eq!(res, Err(Error::NotOwner));
}

#[test]
fn pay_for_woker_only_by_cloud() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new(1, Address::from([2u8; 20]), Address::zero());
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = pod::pay_for_woker(
        Address::from([5u8; 20]),
        AssetInfo::Native(Default::default()),
        U256::from(1u64),
    );
    assert_eq!(res, Err(Error::MustCallByCloudContract));
}

#[test]
fn set_code_returns_err() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new(1, Address::from([2u8; 20]), Address::zero());
    let res = pod::set_code(H256::from([0u8; 32]));
    assert_eq!(res, Err(Error::SetCodeFailed));
}

#[test]
fn set_code_non_cloud_reverts() {
    with_engine(|e| {
        e.reset();
        e.set_caller(cloud_caller());
    });
    let _ = pod::new(1, Address::from([2u8; 20]), Address::zero());
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = pod::set_code(H256::from([0u8; 32]));
    assert_eq!(res, Err(Error::MustCallByCloudContract));
}
