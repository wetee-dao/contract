use crate::errors::Error;
use ink::{prelude::vec::Vec, Address};
use crate::datas::CalllId;

/// Event emitted when a new member is added to the DAO
/// 当新成员加入 DAO 时发出的事件
#[ink::event]
pub struct MemberAdd {
    /// Address of the new member / 新成员地址
    #[ink(topic)]
    pub user: Address,
}

/// Event emitted when a proposal is submitted
/// 当提案被提交时发出的事件
#[ink::event]
pub struct ProposalSubmission {
    /// ID of the submitted proposal / 提交的提案 ID
    #[ink(topic)]
    pub proposal_id: CalllId,
}

/// Event emitted when a proposal is executed
/// 当提案被执行时发出的事件
#[ink::event]
pub struct ProposalExecution {
    /// ID of the executed proposal / 被执行的提案 ID
    #[ink(topic)]
    pub proposal_id: CalllId,
    /// Execution result (Ok contains optional return data, Err contains error)
    /// 执行结果（Ok 包含可选的返回数据，Err 包含错误）
    pub result: Result<Option<Vec<u8>>, Error>,
}

/// Event emitted when a sudo call is executed
/// 当 sudo 调用被执行时发出的事件
#[ink::event]
pub struct SudoExecution {
    /// ID of the sudo call / Sudo 调用 ID
    #[ink(topic)]
    pub sudo_id: CalllId,
    /// Execution result (Ok contains optional return data, Err contains error)
    /// 执行结果（Ok 包含可选的返回数据，Err 包含错误）
    pub result: Result<Option<Vec<u8>>, Error>,
}
