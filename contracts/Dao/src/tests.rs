use super::dao::*;
use crate::traits::Gov;

fn init() -> DAO {
    DAO::new(
        Vec::new(),
        None
    )
}

#[ink::test]
fn proposal_status() {
    let d = init();

    let p = d.proposal_status(0);

    assert!(p.is_err());
}
