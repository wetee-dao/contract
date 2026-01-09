use super::dao::*;
use crate::{datas::*, errors::Error, traits::*};
use ink::env::test::default_accounts;
use ink::{prelude::vec::Vec, U256};

fn init() -> DAO {
    let accounts = default_accounts();
    DAO::new_with_default_track(Vec::new(), true, Some(accounts.alice))
}

#[ink::test]
fn test_proposal_status() {
    let d = init();
    let p = d.proposal_status(0);
    assert!(p.is_err());
}

#[ink::test]
fn test_public_join() {
    let mut dao = DAO::new_with_default_track(Vec::new(), true, None);
    let accounts = default_accounts();
    
    // 测试公开加入
    let result = dao.public_join();
    assert!(result.is_ok());
    
    // 测试重复加入应该失败
    let result2 = dao.public_join();
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), Error::MemberExisted);
}

#[ink::test]
fn test_proposal_status_invalid() {
    let d = init();
    let p = d.proposal_status(999);
    assert!(p.is_err());
}