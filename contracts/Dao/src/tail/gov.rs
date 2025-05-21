use ink::prelude::vec::Vec;
use primitives::{Call, CalllId};

use crate::{datas::{Opinion, PropStatus}, errors::Error};

#[ink::trait_definition]
pub trait GovTrait {
    #[ink(message)]
    fn submit_proposal(&mut self, call: Call) -> CalllId;

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
    fn proposal_status(&self, proposal_id: CalllId) -> PropStatus;
}
