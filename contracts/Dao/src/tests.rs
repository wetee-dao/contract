use ink::U256;

use super::dao::*;
use crate::{curve::Curve, datas::Track, traits::Gov};

fn init() -> DAO {
    DAO::new(
        Vec::new(),
        None,
        Track {
            name: Vec::new(),
            prepare_period: 1,
            max_deciding: 1,
            confirm_period: 1,
            decision_period: 1,
            min_enactment_period: 1,
            decision_deposit: U256::from(1),
            min_approval: Curve::LinearDecreasing {
                begin: 10000,
                end: 50,
                length: 30,
            },
            min_support: Curve::LinearDecreasing {
                begin: 10000,
                end: 50,
                length: 30,
            },
            max_balance: U256::from(1),
        },
    )
}

#[ink::test]
fn proposal_status() {
    let d = init();

    let p = d.proposal_status(0);

    assert!(p.is_err());
}
