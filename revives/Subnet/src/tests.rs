//! Unit tests for Subnet contract. Uses off_chain Engine (wrevive_api::with_engine).

use super::*;
use wrevive_api::{with_engine, U256};

fn gov_caller() -> [u8; 20] {
    [1u8; 20]
}

#[test]
fn deploy_and_epoch_info() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
        e.set_call_data(&[]);
    });
    let _ = subnet::new();
    let info = subnet::epoch_info();
    assert_eq!(info.epoch, 0);
    assert_eq!(info.epoch_solt, 72000);
    assert_eq!(subnet::side_chain_key(), Address::zero());
}

#[test]
fn set_epoch_solt_only_by_gov() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = subnet::new();
    let _ = subnet::set_epoch_solt(36000);
    let info = subnet::epoch_info();
    assert_eq!(info.epoch_solt, 36000);

    with_engine(|e| e.set_caller([99u8; 20]));
    let res = subnet::set_epoch_solt(1);
    assert_eq!(res, Err(Error::MustCallByMainContract));
}

#[test]
fn set_region_and_region() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = subnet::new();
    let name = b"eu-west".to_vec();
    let _ = subnet::set_region(name.clone());
    assert_eq!(subnet::region(0), Some(name));
    assert_eq!(subnet::region(1), None);
}

#[test]
fn set_level_price_and_level_price() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = subnet::new();
    let price = RunPrice {
        cpu_per: 1,
        cvm_cpu_per: 2,
        memory_per: 3,
        cvm_memory_per: 4,
        disk_per: 5,
        gpu_per: 6,
    };
    let _ = subnet::set_level_price(0, price.clone());
    assert_eq!(subnet::level_price(0), Some(price));
}

#[test]
fn set_asset_and_asset() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = subnet::new();
    let info = AssetInfo::Native(alloc::vec::Vec::new());
    let price = U256::from(1000u64);
    let _ = subnet::set_asset(info.clone(), price);
    let got = subnet::asset(0).expect("asset");
    assert_eq!(got.0, info);
    assert_eq!(got.1, price);
}

#[test]
fn set_region_non_gov_reverts() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = subnet::new();
    with_engine(|e| e.set_caller([99u8; 20]));
    let res = subnet::set_region(b"x".to_vec());
    assert_eq!(res, Err(Error::MustCallByMainContract));
}

#[test]
fn get_pending_secrets_empty() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = subnet::new();
    let list = subnet::get_pending_secrets();
    assert!(list.is_empty());
}

#[test]
fn set_code_returns_err() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = subnet::new();
    let res = subnet::set_code(H256::from([0u8; 32]));
    assert_eq!(res, Err(Error::SetCodeFailed));
}
