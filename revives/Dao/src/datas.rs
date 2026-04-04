use parity_scale_codec::{Decode, Encode, Output};
use scale_info::TypeInfo;
use wrevive_api::{Address, BlockNumber, U256, Vec};

use crate::curve::Curve;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct TokenInfo {
    pub name: Vec<u8>,
    pub symbol: Vec<u8>,
    pub decimals: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum Opinion {
    YES = 0,
    NO,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct VoteInfo {
    pub pledge: U256,
    pub opinion: Opinion,
    pub vote_weight: U256,
    pub unlock_block: BlockNumber,
    pub call_id: CalllId,
    pub calller: Address,
    pub vote_block: BlockNumber,
    pub deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct Track {
    pub name: Vec<u8>,
    pub prepare_period: BlockNumber,
    pub decision_deposit: U256,
    pub max_deciding: BlockNumber,
    pub confirm_period: BlockNumber,
    pub decision_period: BlockNumber,
    pub min_enactment_period: BlockNumber,
    pub max_balance: U256,
    pub min_approval: Curve,
    pub min_support: Curve,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum PropStatus {
    Pending,
    Ongoing,
    Confirming,
    Approved(BlockNumber),
    Rejected(BlockNumber),
    Canceled,
}

pub type CalllId = u32;
pub type Selector = [u8; 4];

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct Call {
    pub contract: Option<Address>,
    pub selector: Selector,
    pub input: Vec<u8>,
    pub amount: U256,
    pub ref_time_limit: u64,
    pub allow_reentry: bool,
}

#[derive(Clone)]
pub struct CallInput<'a>(pub &'a [u8]);

impl Encode for CallInput<'_> {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(self.0);
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct Spend {
    pub caller: Address,
    pub to: Address,
    pub amount: U256,
    pub payout: bool,
}
