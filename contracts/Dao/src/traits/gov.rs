use crate::{curve::CurveArg, datas::CalllId};
use ink::{env::BlockNumber, prelude::vec::Vec, U256};

use crate::{
    datas::{Call, Opinion, PropStatus},
    errors::Error,
};

#[ink::trait_definition]
pub trait Gov {
    #[ink(message)]
    fn set_defalut_track(&mut self, id: u16) -> Result<(), Error>;

    #[ink(message)]
    fn add_track(
        &mut self,
        name: Vec<u8>,
        prepare_period: BlockNumber,
        decision_deposit: U256,
        max_deciding: BlockNumber,
        confirm_period: BlockNumber,
        decision_period: BlockNumber,
        min_enactment_period: BlockNumber,
        max_balance: U256,
        min_approval: CurveArg,
        min_support: CurveArg,
    ) -> Result<(), Error>;

    #[ink(message)]
    fn edit_track(
        &mut self,
        id: u16,
        name: Vec<u8>,
        prepare_period: BlockNumber,
        decision_deposit: U256,
        max_deciding: BlockNumber,
        confirm_period: BlockNumber,
        decision_period: BlockNumber,
        min_enactment_period: BlockNumber,
        max_balance: U256,
        min_approval: CurveArg,
        min_support: CurveArg,
    ) -> Result<(), Error>;

    #[ink(message)]
    fn submit_proposal(&mut self, call: Call) -> Result<CalllId, Error>;

    #[ink(message)]
    fn cancel_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error>;

    #[ink(message, payable)]
    fn deposit_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error>;

    #[ink(message)]
    fn vote(&mut self, proposal_id: CalllId, opinion: Opinion) -> Result<(), Error>;

    #[ink(message)]
    fn cancel_vote(&mut self, proposal_id: u128) -> Result<(), Error>;

    #[ink(message)]
    fn unlock(&mut self, vote_id: u128) -> Result<(), Error>;

    #[ink(message)]
    fn exec_proposal(&mut self, proposal_id: CalllId) -> Result<Vec<u8>, Error>;

    #[ink(message)]
    fn proposal_status(&self, proposal_id: CalllId) -> Result<PropStatus, Error>;
}
