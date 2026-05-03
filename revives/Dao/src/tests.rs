use super::*;
use wrevive_api::with_engine;

fn gov() -> [u8; 20] {
    [7u8; 20]
}

fn alice() -> Address {
    Address::from([1u8; 20])
}

fn bob() -> Address {
    Address::from([2u8; 20])
}

fn setup() {
    with_engine(|e| {
        e.reset();
        e.set_caller(gov());
    });
}

#[test]
fn deploy_and_basic_member_flow() {
    setup();
    let users = vec![(alice(), U256::from(100u64))];
    let r = dao::new_with_default_track(users, true, Some(Address::from(gov())));
    assert_eq!(r, Ok(()));
    assert_eq!(dao::get_public_join(), true);
    assert_eq!(dao::total_supply(), U256::from(100u64));
    assert_eq!(dao::balance_of(alice()), U256::from(100u64));
    assert_eq!(dao::list(), vec![alice()]);
}

#[test]
fn public_join_and_leave() {
    setup();
    let users = vec![];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    assert_eq!(dao::public_join(), Ok(()));
    assert_eq!(dao::balance_of(alice()), U256::ZERO);
    assert_eq!(dao::list(), vec![alice()]);

    assert_eq!(dao::levae(), Ok(()));
    assert_eq!(dao::list(), Vec::<Address>::new());
}

#[test]
fn public_join_not_allowed() {
    setup();
    let users = vec![];
    let _ = dao::new_with_default_track(users, false, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    assert_eq!(dao::public_join(), Err(Error::PublicJoinNotAllowed));
}

#[test]
fn join_by_gov() {
    setup();
    let users = vec![];
    let _ = dao::new_with_default_track(users, false, Some(Address::from(gov())));

    // gov-only functions require caller == contract address
    with_engine(|e| e.set_caller(e.current_contract));
    assert_eq!(dao::join(alice(), U256::from(50u64)), Ok(()));
    assert_eq!(dao::balance_of(alice()), U256::from(50u64));
    assert_eq!(dao::total_supply(), U256::from(50u64));
}

#[test]
fn transfer_between_members() {
    setup();
    let users = vec![(alice(), U256::from(100u64)), (bob(), U256::from(50u64))];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    assert_eq!(dao::transfer(bob(), U256::from(30u64)), Ok(()));
    assert_eq!(dao::balance_of(alice()), U256::from(70u64));
    assert_eq!(dao::balance_of(bob()), U256::from(80u64));
}

#[test]
fn transfer_to_non_member_adds_them() {
    setup();
    let users = vec![(alice(), U256::from(100u64))];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    let charlie = Address::from([3u8; 20]);
    assert_eq!(dao::transfer(charlie, U256::from(10u64)), Ok(()));
    assert_eq!(dao::balance_of(charlie), U256::from(10u64));
    assert!(dao::list().contains(&charlie));
}

#[test]
fn approve_and_transfer_from() {
    setup();
    let users = vec![(alice(), U256::from(100u64)), (bob(), U256::from(50u64))];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    assert_eq!(dao::approve(bob(), U256::from(40u64)), Ok(()));
    assert_eq!(dao::allowance(alice(), bob()), U256::from(40u64));

    with_engine(|e| e.set_caller([2u8; 20]));
    assert_eq!(dao::transfer_from(alice(), bob(), U256::from(25u64)), Ok(()));
    assert_eq!(dao::allowance(alice(), bob()), U256::from(15u64));
    assert_eq!(dao::balance_of(alice()), U256::from(75u64));
    assert_eq!(dao::balance_of(bob()), U256::from(75u64));
}

#[test]
fn burn_reduces_supply() {
    setup();
    let users = vec![(alice(), U256::from(100u64))];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    assert_eq!(dao::burn(U256::from(30u64)), Ok(()));
    assert_eq!(dao::balance_of(alice()), U256::from(70u64));
    assert_eq!(dao::total_supply(), U256::from(70u64));
}

#[test]
fn burn_too_much_fails() {
    setup();
    let users = vec![(alice(), U256::from(100u64))];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    assert_eq!(dao::burn(U256::from(200u64)), Err(Error::LowBalance));
}

#[test]
fn sudo_account_workflow() {
    setup();
    let users = vec![(alice(), U256::from(100u64))];
    let sudo = Address::from(gov());
    let _ = dao::new_with_default_track(users, true, Some(sudo));

    assert_eq!(dao::sudo_account(), Some(sudo));

    let call = Call {
        contract: None,
        selector: [0u8; 4],
        input: vec![],
        amount: U256::ZERO,
        ref_time_limit: u64::MAX,
        allow_reentry: false,
    };
    with_engine(|e| e.set_caller(gov()));
    let _ = dao::sudo(call);

    assert_eq!(dao::remove_sudo(), Ok(()));
    assert_eq!(dao::sudo_account(), None);
}

#[test]
fn sudo_by_non_sudo_fails() {
    setup();
    let users = vec![];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    let call = Call {
        contract: None,
        selector: [0u8; 4],
        input: vec![],
        amount: U256::ZERO,
        ref_time_limit: u64::MAX,
        allow_reentry: false,
    };
    assert_eq!(dao::sudo(call), Err(Error::MustCallByGov));
}

#[test]
fn track_management() {
    setup();
    let users = vec![];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller(e.current_contract));
    let track = Track {
        name: b"fast".to_vec(),
        prepare_period: 1,
        max_deciding: 10,
        confirm_period: 1,
        decision_period: 1,
        min_enactment_period: 1,
        decision_deposit: U256::from(1u64),
        max_balance: U256::from(1u64),
        min_approval: Curve::LinearDecreasing { begin: 10000, end: 5000, length: 30 },
        min_support: Curve::LinearDecreasing { begin: 10000, end: 50, length: 30 },
    };
    let tid = dao::add_track(track.clone()).unwrap();
    assert_eq!(dao::track(tid), Some(track.clone()));
    assert_eq!(dao::track_list(), vec![(0, dao::track(0).unwrap()), (tid, track.clone())]);

    assert_eq!(dao::set_defalut_track(tid), Ok(()));
    assert_eq!(dao::defalut_track(), Some(tid));

    with_engine(|e| e.set_caller(e.current_contract));
    let mut edited = track.clone();
    edited.name = b"slow".to_vec();
    assert_eq!(dao::edit_track(tid, edited.clone()), Ok(()));
    assert_eq!(dao::track(tid), Some(edited));
}

#[test]
fn set_track_rule_and_lookup() {
    setup();
    let users = vec![];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller(e.current_contract));
    let track = Track {
        name: b"rule".to_vec(),
        prepare_period: 1,
        max_deciding: 1,
        confirm_period: 1,
        decision_period: 1,
        min_enactment_period: 1,
        decision_deposit: U256::from(1u64),
        max_balance: U256::from(1u64),
        min_approval: Curve::LinearDecreasing { begin: 10000, end: 5000, length: 30 },
        min_support: Curve::LinearDecreasing { begin: 10000, end: 50, length: 30 },
    };
    let tid = dao::add_track(track).unwrap();
    let contract = Some(Address::from([9u8; 20]));
    let selector = Some([0x12u8, 0x34u8, 0x56u8, 0x78u8]);
    assert_eq!(dao::set_track_rule(contract, selector, tid), Ok(()));
}

#[test]
fn token_info_exists() {
    setup();
    let users = vec![(alice(), U256::from(100u64))];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    let token = dao::token(0).expect("token 0 should exist");
    assert_eq!(token.name, b"WeTEE DAO".to_vec());
    assert_eq!(token.symbol, b"DAO".to_vec());
    assert_eq!(token.decimals, 18);
}

#[test]
fn member_not_existed_for_leave() {
    setup();
    let users = vec![];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller([1u8; 20]));
    assert_eq!(dao::levae(), Err(Error::MemberNotExisted));
}

#[test]
fn delete_by_gov() {
    setup();
    let users = vec![(alice(), U256::from(100u64))];
    let _ = dao::new_with_default_track(users, true, Some(Address::from(gov())));

    with_engine(|e| e.set_caller(e.current_contract));
    assert_eq!(dao::delete(alice()), Ok(()));
    assert_eq!(dao::balance_of(alice()), U256::ZERO);
    assert_eq!(dao::total_supply(), U256::ZERO);
}
