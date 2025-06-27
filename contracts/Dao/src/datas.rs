use ink::{env::BlockNumber, prelude::vec::Vec, scale::Output, Address, U256};

use crate::curve::Curve;

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct TokenInfo {
    pub name: Vec<u8>,
    pub symbol: Vec<u8>,
    pub decimals: u8,
}

/// vote yes or no
/// 投票
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Opinion {
    /// Agree.
    YES = 0,
    /// Reject.
    NO,
}

/// Information about votes.
/// 投票信息
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct VoteInfo {
    /// The specific thing that the vote pledged.
    /// 抵押
    pub pledge: U256,
    /// Object or agree.
    /// 是否同意
    pub opinion: Opinion,
    /// voting weight.
    /// 投票权重
    pub vote_weight: U256,
    /// Block height that can be unlocked.
    /// 投票解锁阶段
    pub unlock_block: BlockNumber,
    /// The prop id corresponding to the vote.
    /// 投票的公投
    pub call_id: CalllId,
    /// 投票人
    pub calller: Address,
    /// vote time
    pub vote_block: BlockNumber,
    /// is deleted
    pub deleted: bool,
}

/// 投票轨道
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Track {
    /// 投票轨道名
    pub name: Vec<u8>,

    /// 导入期或准备期
    /// 在这个阶段，提案人或其他人需要支付一笔 “决定押金”。
    pub prepare_period: BlockNumber,
    /// 决定押金 (Native DOT)
    /// 质押后进入投票阶段
    pub decision_deposit: U256,
    /// 投票阶段 => 最长决策期
    /// 如果在 28 天内的某个时间点，Approval 和 Support 均达到了通过阈值时，公投就进入下一个 Confirming 阶段
    pub max_deciding: BlockNumber,
    /// Confirminig 阶段
    /// 如果 Approval 和 Support 两个比率能保持高于通过阈值 “安全” 地渡过这个时期（例如 1 天），该公投就算正式通过了。
    pub confirm_period: BlockNumber,
    /// decision 阶段
    /// 一项公投在投票通过后，再安全地度过了执行期，与该公投相关的代码可以执行。
    pub decision_period: BlockNumber,

    /// 提案结束后多久能解锁
    pub min_enactment_period: BlockNumber,

    /// 最大能执行的金额  用于国库支出相关提案
    pub max_balance: U256,

    /// 投票成功百分比
    pub min_approval: Curve,
    /// 投票率
    pub min_support: Curve,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum PropStatus {
    Pending,
    Ongoing,
    Confirming,
    Approved(BlockNumber),
    Rejected(BlockNumber),
    Canceled,
}

/// Voting Statistics.
/// 投票数据统计
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Tally {
    /// The number of yes votes
    /// 同意的数量
    pub yes: U256,
    /// The number of no votes
    /// 不同意的数量
    pub no: U256,
}

pub type CalllId = u32;
pub type Selector = [u8; 4];

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Call {
    /// The address of the contract that is call in this proposal.
    pub contract: Option<Address>,
    /// The selector bytes that identifies the function of the contract that should be call.
    pub selector: Selector,
    /// The SCALE encoded parameters that are passed to the call function.
    pub input: Vec<u8>,
    /// The amount of chain balance that is transferred to the Proposalee.
    pub amount: U256,
    /// Gas limit for the execution of the call.
    pub ref_time_limit: u64,
    /// If set to true the transaction will be allowed to re-enter the multisig
    /// contract. Re-entrancy can lead to vulnerabilities. Use at your own risk.
    pub allow_reentry: bool,
}

#[derive(Clone)]
pub struct CallInput<'a>(pub &'a [u8]);
impl ink::scale::Encode for CallInput<'_> {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(self.0);
    }
}

#[derive(Clone, Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Spend {
    /// caller address
    pub caller: Address,
    /// pay to address
    pub to: Address,
    /// amount
    pub amount: U256,
    /// is payout
    pub payout: bool,
}

primitives::define_map!(Proposals, CalllId, Call);

primitives::define_map!(Tracks, u16, Track);

primitives::define_map!(Votes, u64, VoteInfo);

primitives::define_map!(SudoCalls, CalllId, Call);

primitives::define_double_map!(VoteOfProposal, CalllId, u64);

primitives::define_double_map!(VoteOfMember, Address, u64);