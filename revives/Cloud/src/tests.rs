//! Unit tests for Cloud contract. Uses off_chain Engine (wrevive_api::with_engine).

use super::*;
use wrevive_api::with_engine;

fn gov_caller() -> [u8; 20] {
    [3u8; 20]
}

#[test]
fn deploy_and_getters() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);
    assert_eq!(cloud::subnet_address(), subnet_addr);
    assert_eq!(cloud::mint_interval(), 14400);
    assert_eq!(cloud::pod_len(), 0);
    assert_eq!(cloud::pod_contract(), code_hash);
}

#[test]
fn set_mint_interval_only_by_gov() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);

    let _ = cloud::set_mint_interval(10000);
    assert_eq!(cloud::mint_interval(), 10000);

    with_engine(|e| e.set_caller([99u8; 20]));
    let res = cloud::set_mint_interval(20000);
    assert_eq!(res, Err(Error::MustCallByGovContract));
    with_engine(|e| e.set_caller(gov_caller()));
    assert_eq!(cloud::mint_interval(), 10000);
}

#[test]
fn set_pod_contract_only_by_gov() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);

    let new_hash = H256::from([5u8; 32]);
    let _ = cloud::set_pod_contract(new_hash);
    assert_eq!(cloud::pod_contract(), new_hash);

    with_engine(|e| e.set_caller([99u8; 20]));
    let res = cloud::set_pod_contract(H256::from([6u8; 32]));
    assert_eq!(res, Err(Error::MustCallByGovContract));
}

#[test]
fn create_secret_and_user_secrets() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    let alice = Address::from([10u8; 20]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);

    with_engine(|e| e.set_caller([10u8; 20]));
    let key = b"my_secret_key".to_vec();
    let hash = H256::from([1u8; 32]);
    let id = cloud::create_secret(key.clone(), hash).expect("create_secret");
    assert_eq!(id, 0);

    let s = cloud::secret(alice, 0).expect("secret");
    assert_eq!(s.k, key);
    assert_eq!(s.hash, hash);
    assert!(!s.minted);

    let list = cloud::user_secrets(alice, None, 10);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].0, 0);
}

#[test]
fn del_secret() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    let alice = Address::from([11u8; 20]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);

    with_engine(|e| e.set_caller([11u8; 20]));
    let _ = cloud::create_secret(b"k".to_vec(), H256::zero());
    let _ = cloud::del_secret(0);
    assert!(cloud::secret(alice, 0).is_none());
}

#[test]
fn charge_and_balance() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);
    let _ = cloud::charge();
    let bal = cloud::balance(AssetInfo::Native(Default::default()));
    assert_eq!(bal, U256::ZERO);
}

#[test]
fn pods_empty_and_user_pod_len() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);
    let list = cloud::pods(None, 10);
    assert!(list.is_empty());
    assert_eq!(cloud::user_pod_len(), 0);
}

#[test]
fn update_pod_contract_returns_err_when_pod_not_found() {
    let subnet_addr = Address::from([1u8; 20]);
    let code_hash = H256::from([2u8; 32]);
    with_engine(|e| {
        e.reset();
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr, code_hash);
    let res = cloud::update_pod_contract(999);
    assert_eq!(res, Err(Error::PodNotFound));
}
