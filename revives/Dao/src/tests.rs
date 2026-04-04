use super::*;
use wrevive_api::with_engine;

fn gov() -> [u8; 20] {
    [7u8; 20]
}

#[test]
fn deploy_and_basic_member_flow() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov());
    });

    let users = vec![(Address::from([1u8; 20]), U256::from(100u64))];
    let r = dao::new(users, true, Some(Address::from(gov())));
    assert_eq!(r, Ok(()));
    assert_eq!(dao::get_public_join(), true);
    assert_eq!(dao::total_supply(), U256::from(100u64));
    assert_eq!(dao::balance_of(Address::from([1u8; 20])), U256::from(100u64));
}
