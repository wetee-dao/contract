use ink::{env::BlockNumber, prelude::vec::Vec, U256};
use primitives::CalllId;

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
}

/// 投票轨道
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Period {
    /// 投票轨道名
    pub name: Vec<u8>,
    /// 导入期或准备期
    /// 在这个阶段，提案人或其他人需要支付一笔 “决定押金”。
    pub prepare_period: BlockNumber,
    /// 最长决策期
    /// 就像上面说到的，如果在 28 天内的某个时间点，
    /// Approval 和 Support 均达到了通过阈值时，
    /// 公投就进入下一个 Confirming 阶段
    pub max_deciding: BlockNumber,
    /// Confirming 阶段的长度视轨道参数决定
    /// 在 Confirminig 阶段，如果 Approval 和 Support
    /// 两个比率能保持高于通过阈值 “安全” 地渡过这个时期（例如 1 天），
    /// 那么该公投就算正式通过了。
    pub confirm_period: BlockNumber,
    /// 一项公投在投票通过后，再安全地度过了执行期，与该公投相关的代码可以执行。
    pub decision_period: BlockNumber,
    /// 提案结束后多久能解锁
    pub min_enactment_period: BlockNumber,
    /// 决定押金
    pub decision_deposit: U256,
    /// 投票成功百分比
    pub min_approval: u8,
    /// 投票率
    pub min_support: u8,
    /// 最大能执行的金额
    /// 如果金额范围不合理，就无法成功执行提案
    pub max_balance: U256,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum PropStatus {
    Pending = 0,
    Ongoing,
    Approved,
    Rejected,
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