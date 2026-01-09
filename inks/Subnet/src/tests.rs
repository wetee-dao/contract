use super::subnet::*;
use crate::datas::*;
use ink::env::test::default_accounts;

fn init() -> Subnet {
    Subnet::new()
}

#[ink::test]
fn test_new() {
    let subnet = init();
    // 新创建的 Subnet 应该没有验证者
    let validators = subnet.validators();
    assert_eq!(validators.len(), 0);
}

#[ink::test]
fn test_secret_register() {
    use ink::env::DefaultEnvironment;

    let mut c = init();
    let accounts = default_accounts();
    let alice = accounts.alice;

    // 注册第一个节点
    let result1 = c.secret_register(
        "node0".as_bytes().to_vec(),
        alice,
        alice,
        Ip {
            ipv4: None,
            ipv6: None,
            domain: None,
        },
        100,
    );
    assert!(result1.is_ok());

    // 注册第二个节点
    let result2 = c.secret_register(
        "node1".as_bytes().to_vec(),
        alice,
        alice,
        Ip {
            ipv4: None,
            ipv6: None,
            domain: None,
        },
        100,
    );
    assert!(result2.is_ok());

    let list = c.validators();
    assert_eq!(list.len(), 0); // 刚注册时还不是验证者

    // 节点加入验证者集合
    let join_result = c.validator_join(0);
    assert!(join_result.is_ok());

    let plist = c.get_pending_secrets();
    assert!(plist.len() > 0);

    // 设置区块号到下一个 epoch
    ink::env::test::set_block_number::<DefaultEnvironment>(72000);

    let list2 = c.next_epoch_validators();
    assert!(list2.is_some());
    let validators = list2.unwrap();
    assert!(validators.len() > 0);
}

#[ink::test]
fn test_worker_register() {
    let mut subnet = init();
    let accounts = default_accounts();
    
    // 先创建区域
    ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
    let _ = subnet.set_region("region1".as_bytes().to_vec());
    
    // 注册工作节点
    ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
    let result = subnet.worker_register(
        "worker1".as_bytes().to_vec(),
        accounts.bob,
        Ip {
            ipv4: Some("192.168.1.1".as_bytes().to_vec()),
            ipv6: None,
            domain: None,
        },
        100,
        1,  // level
        0,  // region_id
    );
    
    assert!(result.is_ok());
}

#[ink::test]
fn test_validator_join() {
    let mut subnet = init();
    let accounts = default_accounts();
    
    // 先注册节点
    let result1 = subnet.secret_register(
        "node0".as_bytes().to_vec(),
        accounts.alice,
        accounts.alice,
        Ip {
            ipv4: None,
            ipv6: None,
            domain: None,
        },
        100,
    );
    assert!(result1.is_ok());
    
    // 节点加入验证者集合
    let result = subnet.validator_join(0);
    assert!(result.is_ok());
    
    let pending = subnet.get_pending_secrets();
    assert!(pending.len() > 0);
}

#[ink::test]
fn test_validators() {
    let subnet = init();
    let validators = subnet.validators();
    // 初始状态应该没有验证者
    assert_eq!(validators.len(), 0);
}

#[ink::test]
fn test_get_pending_secrets() {
    let subnet = init();
    let pending = subnet.get_pending_secrets();
    // 初始状态应该没有待处理的密钥
    assert_eq!(pending.len(), 0);
}
