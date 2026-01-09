use ink::{env::BlockNumber, prelude::vec::Vec, scale::Output, Address, U256};

use crate::curve::Curve;

/// Token information structure
/// 代币信息结构
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct TokenInfo {
    /// Token name / 代币名称
    pub name: Vec<u8>,
    /// Token symbol / 代币符号
    pub symbol: Vec<u8>,
    /// Number of decimals / 小数位数
    pub decimals: u8,
}

/// Vote opinion: Yes or No
/// 投票意见：同意或拒绝
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Opinion {
    /// Agree / 同意
    YES = 0,
    /// Reject / 拒绝
    NO,
}

/// Information about a vote
/// 投票信息
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct VoteInfo {
    /// The amount of tokens pledged for this vote / 本次投票抵押的代币数量
    pub pledge: U256,
    /// Vote opinion (YES or NO) / 投票意见（同意或拒绝）
    pub opinion: Opinion,
    /// Voting weight multiplier / 投票权重倍数
    pub vote_weight: U256,
    /// Block number when tokens can be unlocked / 代币可以解锁的区块号
    pub unlock_block: BlockNumber,
    /// The proposal ID this vote corresponds to / 本次投票对应的提案 ID
    pub call_id: CalllId,
    /// Address of the voter / 投票人地址
    pub calller: Address,
    /// Block number when the vote was cast / 投票时的区块号
    pub vote_block: BlockNumber,
    /// Whether this vote has been deleted / 本次投票是否已被删除
    pub deleted: bool,
}

/// Voting track configuration
/// 投票轨道配置
/// 
/// A track defines the voting rules and timeline for proposals.
/// 轨道定义了提案的投票规则和时间线。
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Track {
    /// Track name / 投票轨道名称
    pub name: Vec<u8>,

    /// Preparation period (blocks)
    /// In this stage, the proposer or others need to pay a "decision deposit".
    /// 准备期（区块数）
    /// 在这个阶段，提案人或其他人需要支付一笔"决定押金"。
    pub prepare_period: BlockNumber,
    /// Decision deposit amount (native tokens)
    /// After depositing, the proposal enters the voting stage.
    /// 决定押金金额（原生代币）
    /// 质押后进入投票阶段。
    pub decision_deposit: U256,
    /// Maximum deciding period (blocks) - voting stage
    /// If Approval and Support both reach the threshold at some point within this period,
    /// the referendum enters the Confirming stage.
    /// 最长决策期（区块数）- 投票阶段
    /// 如果在此期间的某个时间点，Approval 和 Support 均达到通过阈值，公投就进入确认阶段。
    pub max_deciding: BlockNumber,
    /// Confirming period (blocks)
    /// If Approval and Support ratios remain above the threshold for this period (e.g., 1 day),
    /// the referendum is officially passed.
    /// 确认期（区块数）
    /// 如果 Approval 和 Support 两个比率能保持高于通过阈值"安全"地渡过这个时期（例如 1 天），该公投就算正式通过了。
    pub confirm_period: BlockNumber,
    /// Decision period (blocks)
    /// After a referendum passes and safely completes this execution period,
    /// the code related to the referendum can be executed.
    /// 决策期（区块数）
    /// 一项公投在投票通过后，再安全地度过了执行期，与该公投相关的代码可以执行。
    pub decision_period: BlockNumber,

    /// Minimum period before tokens can be unlocked after proposal ends (blocks)
    /// 提案结束后多久能解锁代币（区块数）
    pub min_enactment_period: BlockNumber,

    /// Maximum executable amount (for treasury spending proposals)
    /// 最大能执行的金额（用于国库支出相关提案）
    pub max_balance: U256,

    /// Minimum approval percentage curve (voting success threshold)
    /// 投票成功百分比曲线（投票通过阈值）
    pub min_approval: Curve,
    /// Minimum support percentage curve (voting participation threshold)
    /// 投票率曲线（投票参与阈值）
    pub min_support: Curve,
}

/// Proposal status
/// 提案状态
#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum PropStatus {
    /// Proposal is pending (waiting for deposit) / 提案待处理（等待押金）
    Pending,
    /// Proposal is ongoing (voting period) / 提案进行中（投票期）
    Ongoing,
    /// Proposal is in confirming stage / 提案在确认阶段
    Confirming,
    /// Proposal is approved (with block number when approved) / 提案已批准（包含批准时的区块号）
    Approved(BlockNumber),
    /// Proposal is rejected (with block number when rejected) / 提案已拒绝（包含拒绝时的区块号）
    Rejected(BlockNumber),
    /// Proposal is canceled / 提案已取消
    Canceled,
}


/// Proposal ID type / 提案 ID 类型
pub type CalllId = u32;
/// Function selector type (4 bytes) / 函数选择器类型（4 字节）
pub type Selector = [u8; 4];

/// Call structure for proposals
/// 提案调用结构
/// 
/// Represents a call that will be executed if the proposal passes.
/// 表示提案通过后将执行的调用。
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Call {
    /// The address of the contract to call (None means self) / 要调用的合约地址（None 表示自身）
    pub contract: Option<Address>,
    /// The selector bytes that identifies the function to call / 标识要调用的函数的选择器字节
    pub selector: Selector,
    /// The SCALE encoded parameters passed to the call function / 传递给调用函数的 SCALE 编码参数
    pub input: Vec<u8>,
    /// The amount of native tokens transferred with the call / 随调用一起转账的原生代币数量
    pub amount: U256,
    /// Reference time limit (gas limit) for the execution of the call / 调用执行的参考时间限制（gas 限制）
    pub ref_time_limit: u64,
    /// If true, the transaction will be allowed to re-enter the contract.
    /// Re-entrancy can lead to vulnerabilities. Use at your own risk.
    /// 如果为 true，将允许交易重入合约。
    /// 重入可能导致漏洞。使用需自担风险。
    pub allow_reentry: bool,
}

/// Call input wrapper for SCALE encoding
/// 用于 SCALE 编码的调用输入包装器
#[derive(Clone)]
pub struct CallInput<'a>(pub &'a [u8]);

impl ink::scale::Encode for CallInput<'_> {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(self.0);
    }
}

/// Treasury spend record
/// 国库支出记录
#[derive(Clone, Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Spend {
    /// Address of the spender (who requested the spend) / 支出者地址（请求支出的人）
    pub caller: Address,
    /// Address to pay to / 支付目标地址
    pub to: Address,
    /// Amount to spend / 支出金额
    pub amount: U256,
    /// Whether the spend has been paid out / 是否已支付
    pub payout: bool,
}

// Proposals storage mapping / 提案存储映射
primitives::define_map!(Proposals, CalllId, Call);

// Tracks storage mapping / 轨道存储映射
primitives::define_map!(Tracks, u16, Track);

// Votes storage mapping / 投票存储映射
primitives::define_map!(Votes, u64, VoteInfo);

// Sudo calls storage mapping / Sudo 调用存储映射
primitives::define_map!(SudoCalls, CalllId, Call);

// Vote IDs of proposals (double mapping) / 提案的投票 ID（双重映射）
primitives::double_u32_map!(VoteOfProposal, CalllId, u64);

// Vote IDs of members (double mapping) / 成员的投票 ID（双重映射）
primitives::double_u32_map!(VoteOfMember, Address, u64);