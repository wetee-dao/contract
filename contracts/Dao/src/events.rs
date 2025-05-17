use crate::errors::Error;
use ink::{prelude::vec::Vec, Address};
use primitives::CalllId;

/// The member added event.
#[ink::event]
pub struct MemberAdd {
    #[ink(topic)]
    pub user: Address,
}

/// The proposal call submission event.
#[ink::event]
pub struct ProposalSubmission {
    #[ink(topic)]
    pub proposal_id: CalllId,
}

/// The proposal call execution event.
#[ink::event]
pub struct ProposalExecution {
    #[ink(topic)]
    pub proposal_id: CalllId,
    #[ink(topic)]
    pub result: Result<Option<Vec<u8>>, Error>,
}

/// The sudo call execution event.
#[ink::event]
pub struct SudoExecution {
    #[ink(topic)]
    pub sudo_id: CalllId,
    #[ink(topic)]
    pub result: Result<Option<Vec<u8>>, Error>,
}
