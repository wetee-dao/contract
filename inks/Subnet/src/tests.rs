use super::subnet::*;

fn init() -> Subnet {
    Subnet::new()
}

#[ink::test]
fn test_secret_register() {
    use ink::env::DefaultEnvironment;

    let mut c = init();
    let alice = ink::primitives::AccountId::from([0x01; 32]);

    _ = c.secret_register(
        "node0".as_bytes().to_vec(),
        alice,
        alice,
        crate::datas::Ip {
            ipv4: None,
            ipv6: None,
            domain: None,
        },
        100,
    );

    _ = c.secret_register(
        "node1".as_bytes().to_vec(),
        alice,
        alice,
        crate::datas::Ip {
            ipv4: None,
            ipv6: None,
            domain: None,
        },
        100,
    );

    let list = c.validators();
    println!("list1 {:?}", list);

    _ = c.validator_join(1);

    let plist = c.get_pending_secrets();
    println!("plist {:?}", plist);

    ink::env::test::set_block_number::<DefaultEnvironment>(72000);

    let list2 = c.next_epoch_validators();
    println!("list2 {:?}", list2.unwrap());
}
