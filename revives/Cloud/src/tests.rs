//! Unit tests for Cloud contract. Uses off_chain Engine (wrevive_api::with_engine).

use super::*;
use wrevive_api::{AccountId, Address, H256, U256, with_engine};

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

// ---------------------------------------------------------------------------
// Helper constants & functions for Pod lifecycle / arbitration tests
// ---------------------------------------------------------------------------

fn alice() -> Address {
    Address::from([10u8; 20])
}

fn cloud_addr() -> Address {
    Address::from([100u8; 20])
}

fn subnet_addr() -> Address {
    Address::from([1u8; 20])
}

fn side_chain_key() -> Address {
    Address::zero()
}

/// Initialize Cloud + Subnet + register dispatchers + create one worker.
fn setup_cloud_subnet_worker() {
    with_engine(|e| {
        e.reset_all();
        e.set_contract(cloud_addr());
        e.set_caller(gov_caller());
    });
    let _ = cloud::new();
    let _ = cloud::init(subnet_addr(), H256::from([2u8; 32]));

    // Register Subnet dispatcher so Cloud can call Subnet messages
    with_engine(|e| {
        e.register_contract(subnet_addr(), || subnet::call());
    });

    // Initialize Subnet contract
    with_engine(|e| {
        e.set_contract(subnet_addr());
        e.set_caller(gov_caller());
    });
    let _ = subnet::subnet::new();
    let _ = subnet::subnet::init();

    // Set up region, level price, asset, and cloud contract reference
    with_engine(|e| {
        e.set_contract(subnet_addr());
        e.set_caller(gov_caller());
    });
    let _ = subnet::subnet::set_region(b"eu".to_vec());

    with_engine(|e| {
        e.set_contract(subnet_addr());
        e.set_caller(gov_caller());
    });
    let price = RunPrice {
        cpu_per: 1,
        cvm_cpu_per: 2,
        memory_per: 3,
        cvm_memory_per: 4,
        disk_per: 5,
        gpu_per: 6,
    };
    let _ = subnet::subnet::set_level_price(1, price);

    with_engine(|e| {
        e.set_contract(subnet_addr());
        e.set_caller(gov_caller());
    });
    let _ = subnet::subnet::set_asset(AssetInfo::Native(b"TEST".to_vec()), U256::from(1000u64));

    with_engine(|e| {
        e.set_contract(subnet_addr());
        e.set_caller(gov_caller());
    });
    let _ = subnet::subnet::set_cloud_contract(cloud_addr());

    // Register a worker so create_pod has a valid target
    with_engine(|e| {
        e.set_contract(subnet_addr());
        e.set_caller(*alice().as_ref());
    });
    let p2p_id = AccountId::from([1u8; 32]);
    let ip = Ip {
        ipv4: Some(3232263885),
        ipv6: None,
        domain: None,
    };
    let _ = subnet::subnet::worker_register(
        b"worker-0".to_vec(),
        p2p_id,
        ip,
        30333,
        1,
        0,
    )
    .expect("worker_register should succeed");
}

fn create_pod_basic() {
    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let result = cloud::create_pod(
        b"test-pod".to_vec(),
        PodType::CPU,
        TEEType::SGX,
        vec![],
        0, // region_id
        1, // level
        0, // pay_asset
        0, // worker_id
        1, // duration_blocks
    );
    assert!(result.is_ok(), "create_pod failed: {:?}", result);
}

fn start_pod_basic() {
    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*side_chain_key().as_ref());
    });
    let result = cloud::start_pod(0, AccountId::from([2u8; 32]));
    assert!(result.is_ok(), "start_pod failed: {:?}", result);
}

/// Manually initialize Pod contract storage after off_chain instantiate.
/// Returns the Pod contract address.
fn init_pod_contract() -> Address {
    let pod_addr = cloud::pod(0).unwrap().0.pod_address;

    // Register Pod dispatcher so Cloud can call Pod messages
    with_engine(|e| {
        e.register_contract(pod_addr, || pod::call());
    });

    // Manually initialize Pod storage (off_chain instantiate does not call deploy)
    with_engine(|e| {
        e.set_contract(pod_addr);
        e.set_caller(*cloud_addr().as_ref());
    });
    let _ = pod::pod::new(0, alice(), side_chain_key());

    // Restore Cloud contract context
    with_engine(|e| {
        e.set_contract(cloud_addr());
    });
    pod_addr
}

fn mint_pod_basic() {
    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*side_chain_key().as_ref());
    });
    let result = cloud::mint_pod(0, H256::from([99u8; 32]));
    assert!(result.is_ok(), "mint_pod failed: {:?}", result);
}

/// Full helper: create → start → init Pod contract → mint (up to settlement)
fn create_and_settle_pod() {
    setup_cloud_subnet_worker();
    create_pod_basic();
    start_pod_basic();
    let _ = init_pod_contract();
    mint_pod_basic();
}

// ---------------------------------------------------------------------------
// Pod lifecycle tests
// ---------------------------------------------------------------------------

#[test]
fn pod_lifecycle_create_start_stop() {
    setup_cloud_subnet_worker();

    // Alice creates a pod (empty containers => zero cost)
    create_pod_basic();

    // Verify pod was created
    assert_eq!(cloud::pod_len(), 1);
    let pod_info = cloud::pod(0).expect("pod should exist");
    assert_eq!(pod_info.0.owner, alice());
    assert_eq!(pod_info.3, 0); // status = 0 (created)
    assert!(!pod_info.0.is_settled);

    // Start pod as side-chain
    start_pod_basic();
    let pod_info = cloud::pod(0).expect("pod should exist");
    assert_eq!(pod_info.3, 1); // status = 1 (running)

    // Stop pod as owner
    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let result = cloud::stop_pod(0);
    assert!(result.is_ok(), "stop_pod failed: {:?}", result);

    let pod_info = cloud::pod(0).expect("pod should exist");
    assert_eq!(pod_info.3, 3); // status = 3 (stopped)
}

#[test]
fn pod_mint_marks_settled() {
    setup_cloud_subnet_worker();
    create_pod_basic();
    start_pod_basic();
    let _ = init_pod_contract();

    // Before mint: not settled
    let pod = cloud::pod(0).unwrap().0;
    assert!(!pod.is_settled);

    // Mint as side-chain
    mint_pod_basic();

    // After mint: settled
    let pod = cloud::pod(0).unwrap().0;
    assert!(pod.is_settled);
    assert_eq!(pod.settled_amount, pod.prepaid_amount);
}

#[test]
fn pod_renew_fails_when_settled() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
        e.value_transferred = U256::from(100u64);
    });
    let result = cloud::renew_pod(0, 10);
    assert_eq!(result, Err(Error::PodAlreadySettled));
}

#[test]
fn pod_restart_fails_when_settled() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let result = cloud::restart_pod(0);
    assert_eq!(result, Err(Error::PodAlreadySettled));
}

#[test]
fn pod_start_fails_when_settled() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*side_chain_key().as_ref());
    });
    let result = cloud::start_pod(0, AccountId::from([2u8; 32]));
    assert_eq!(result, Err(Error::PodAlreadySettled));
}

#[test]
fn pod_stop_allows_settled_pod() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let result = cloud::stop_pod(0);
    assert!(result.is_ok(), "stop_pod should succeed even after settlement");
    let pod_info = cloud::pod(0).unwrap();
    assert_eq!(pod_info.3, 3); // status = stopped
}

// ---------------------------------------------------------------------------
// Arbitration tests
// ---------------------------------------------------------------------------

#[test]
fn pod_arbitration_submit_and_resolve_approved() {
    create_and_settle_pod();

    // Submit arbitration as owner
    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let arb_id = cloud::submit_arbitration(0, U256::from(50u64), b"bad service".to_vec()).unwrap();
    assert_eq!(arb_id, 0);

    let arb = cloud::arbitration(0).expect("arbitration should exist");
    assert_eq!(arb.status, ArbitrationStatus::Pending);
    assert_eq!(arb.pod_id, 0);
    assert_eq!(arb.claimant, alice());
    assert_eq!(arb.amount, U256::from(50u64));

    // Resolve as side-chain (approved, zero deduction to avoid slash logic)
    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*side_chain_key().as_ref());
    });
    let result = cloud::resolve_arbitration(0, true, U256::ZERO);
    assert!(result.is_ok(), "resolve_arbitration failed: {:?}", result);

    let arb = cloud::arbitration(0).unwrap();
    assert_eq!(arb.status, ArbitrationStatus::Approved);
    assert_eq!(arb.result_amount, U256::ZERO);
    assert!(arb.resolved_at.is_some());

    // Pod arbitration list
    let arbs = cloud::pod_arbitrations(0, None, 10);
    assert_eq!(arbs.len(), 1);
    assert_eq!(arbs[0].0, 0);
}

#[test]
fn pod_arbitration_rejected() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let _ = cloud::submit_arbitration(0, U256::from(50u64), b"bad service".to_vec());

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*side_chain_key().as_ref());
    });
    let result = cloud::resolve_arbitration(0, false, U256::ZERO);
    assert!(result.is_ok());

    let arb = cloud::arbitration(0).unwrap();
    assert_eq!(arb.status, ArbitrationStatus::Rejected);
}

#[test]
fn pod_arbitration_non_owner_fails() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller([99u8; 20]);
    });
    let result = cloud::submit_arbitration(0, U256::from(50u64), b"bad service".to_vec());
    assert_eq!(result, Err(Error::NotPodOwner));
}

#[test]
fn pod_resolve_arbitration_non_side_chain_fails() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let _ = cloud::submit_arbitration(0, U256::from(50u64), b"bad service".to_vec());

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller([99u8; 20]);
    });
    let result = cloud::resolve_arbitration(0, true, U256::ZERO);
    assert_eq!(result, Err(Error::InvalidSideChainCaller));
}

#[test]
fn pod_resolve_arbitration_already_resolved_fails() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*alice().as_ref());
    });
    let _ = cloud::submit_arbitration(0, U256::from(50u64), b"bad service".to_vec());

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*side_chain_key().as_ref());
    });
    let _ = cloud::resolve_arbitration(0, true, U256::ZERO);

    // Try to resolve again
    let result = cloud::resolve_arbitration(0, true, U256::ZERO);
    assert_eq!(result, Err(Error::ArbitrationAlreadyResolved));
}

#[test]
fn pod_arbitration_not_found_fails() {
    create_and_settle_pod();

    with_engine(|e| {
        e.set_contract(cloud_addr());
        e.set_caller(*side_chain_key().as_ref());
    });
    let result = cloud::resolve_arbitration(999, true, U256::ZERO);
    assert_eq!(result, Err(Error::ArbitrationNotFound));
}
