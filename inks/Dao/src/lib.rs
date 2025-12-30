#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod curve;
mod datas;
mod errors;
mod events;
mod traits;

#[ink::contract]
mod dao {
    use crate::{
        curve::{arg_to_curve, Curve, CurveArg},
        datas::*,
        errors::Error,
        events::*,
        traits::*,
    };
    use ink::{
        env::{
            call::{build_call, utils::ArgumentList, ExecutionInput},
            CallFlags,
        },
        prelude::vec::Vec,
        scale::Encode,
        storage::Mapping,
        H256, U256,
    };
    use primitives::{ensure, ok_or_err};

    /// DAO Contract Storage
    /// DAO 合约存储结构
    /// 
    /// This contract implements a decentralized autonomous organization (DAO) with governance,
    /// voting, treasury management, and ERC20-like token functionality.
    /// 该合约实现了去中心化自治组织（DAO），包含治理、投票、国库管理和类似 ERC20 的代币功能。
    #[ink(storage)]
    #[derive(Default)]
    pub struct DAO {
        /// Proposals storage / 提案存储
        proposals: Proposals,
        /// Track ID of each proposal / 每个提案的轨道 ID
        track_of_proposal: Mapping<CalllId, u16>,
        /// Caller address of each proposal / 每个提案的提交者地址
        proposal_caller: Mapping<CalllId, Address>,
        /// Deposit information: (depositor, amount, block_number) / 押金信息：(押金者, 金额, 区块号)
        deposit_of_proposal: Mapping<CalllId, (Address, U256, BlockNumber)>,
        /// Status of each proposal / 每个提案的状态
        status_of_proposal: Mapping<CalllId, PropStatus>,
        /// Votes associated with each proposal / 与每个提案关联的投票
        votes_of_proposal: VoteOfProposal,
        /// Block number when proposal was submitted / 提案提交时的区块号
        submit_block_of_proposal: Mapping<CalllId, BlockNumber>,

        /// Voting tracks configuration / 投票轨道配置
        tracks: Tracks,
        /// Track rules mapping: (contract, selector) -> track_id
        /// If selector == none, it means entire contract uses a single track
        /// 轨道规则映射：(合约地址, 选择器) -> 轨道ID
        /// 如果选择器为 None，表示整个合约使用单一轨道
        track_rules: Mapping<(Option<Address>, Option<Selector>), u16>,
        /// Default track ID for proposals / 提案的默认轨道 ID
        defalut_track: Option<u16>,

        /// All votes storage / 所有投票存储
        votes: Votes,
        /// Vote IDs of each member / 每个成员的投票 ID
        vote_of_member: VoteOfMember,
        /// Unlocked votes tracking / 已解锁投票追踪
        unlock_of_votes: Mapping<u64, ()>,

        /// Sudo call history / Sudo 调用历史
        sudo_calls: SudoCalls,
        /// Sudo account address (can execute any function without governance) / Sudo 账户地址（无需治理即可执行任何函数）
        sudo_account: Option<Address>,

        /// List of all member addresses / 所有成员地址列表
        members: Vec<Address>,
        /// Whether members can join without governance approval / 成员是否可以在无需治理批准的情况下加入
        public_join: bool,
        /// Balance of each member / 每个成员的余额
        member_balances: Mapping<Address, U256>,
        /// Locked balance of each member (used for voting) / 每个成员的锁定余额（用于投票）
        member_lock_balances: Mapping<Address, U256>,
        /// Total token issuance / 代币总发行量
        total_issuance: U256,
        /// Mapping of the token amount which an account is allowed to withdraw from another account
        /// 允许一个账户从另一个账户提取的代币数量映射
        allowances: Mapping<(Address, Address), U256>,
        /// Whether token transfer is enabled / 代币转账是否启用
        transfer: bool,

        /// Token information by token ID / 按代币 ID 的代币信息
        tokens: Mapping<u32, TokenInfo>,
        /// Token balances of each member by token ID / 每个成员按代币 ID 的代币余额
        member_tokens: Mapping<(Address, u32), U256>,

        /// Next spend ID for treasury operations / 国库操作的下一个支出 ID
        next_spend_id: u64,
        /// Treasury spend records / 国库支出记录
        spends: Mapping<u64, Spend>,
    }

    impl Member for DAO {
        /// Returns list of all members
        /// 返回所有成员列表
        /// 
        /// # Returns
        /// * `Vec<Address>` - List of member addresses / 成员地址列表
        #[ink(message)]
        fn list(&self) -> Vec<Address> {
            self.members.clone()
        }

        /// Returns whether public join is enabled
        /// 返回是否启用公开加入
        /// 
        /// # Returns
        /// * `bool` - True if public join is enabled / 如果启用公开加入则返回 true
        #[ink(message)]
        fn get_public_join(&self) -> bool {
            self.public_join
        }

        /// Public join DAO (anyone can join if public_join is enabled)
        /// 公开加入 DAO（如果启用了 public_join，任何人都可以加入）
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful, Error if failed
        ///   成功返回 Ok，失败返回 Error
        /// 
        /// # Errors
        /// * `PublicJoinNotAllowed` - Public join is not enabled / 未启用公开加入
        /// * `MemberExisted` - User is already a member / 用户已经是成员
        #[ink(message)]
        fn public_join(&mut self) -> Result<(), Error> {
            ensure!(self.public_join, Error::PublicJoinNotAllowed);

            let caller = self.env().caller();

            // check if user is already an member
            ensure!(!self.member_balances.contains(caller), Error::MemberExisted);

            self.member_balances.insert(caller, &U256::from(0));
            self.members.push(caller);

            self.env().emit_event(MemberAdd { user: caller });

            Ok(())
        }

        /// Set whether public join is enabled or not (governance only)
        /// 设置是否启用公开加入（仅治理）
        /// 
        /// # Arguments
        /// * `public_join` - True to enable public join, false to disable / true 启用公开加入，false 禁用
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MustCallByGov` - Must be called by governance / 必须由治理调用
        #[ink(message)]
        fn set_public_join(&mut self, public_join: bool) -> Result<(), Error> {
            self.ensure_from_gov()?;
            self.public_join = public_join;

            Ok(())
        }

        /// Join to DAO with governance approval (governance only)
        /// 通过治理批准加入 DAO（仅治理）
        /// 
        /// # Arguments
        /// * `new_user` - Address of the new member / 新成员地址
        /// * `balance` - Initial token balance for the new member / 新成员的初始代币余额
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MustCallByGov` - Must be called by governance / 必须由治理调用
        /// * `MemberExisted` - User is already a member / 用户已经是成员
        #[ink(message)]
        fn join(&mut self, new_user: Address, balance: U256) -> Result<(), Error> {
            self.ensure_from_gov()?;

            // check if user is already an member
            ensure!(
                !self.member_balances.contains(new_user),
                Error::MemberExisted
            );

            self.member_balances.insert(new_user, &balance);
            self.members.push(new_user);

            self.env().emit_event(MemberAdd { user: new_user });

            Ok(())
        }

        /// Leave DAO (only if balance is zero)
        /// 离开 DAO（仅当余额为零时）
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MemberNotExisted` - User is not a member / 用户不是成员
        /// * `MemberBalanceNotZero` - Member still has balance or locked balance / 成员仍有余额或锁定余额
        #[ink(message)]
        fn levae(&mut self) -> Result<(), Error> {
            let caller = self.env().caller();

            // check if user is already an member
            ensure!(
                self.member_balances.contains(caller),
                Error::MemberNotExisted
            );
            ensure!(
                self.member_balances.get(caller).unwrap_or(U256::from(0)) == U256::from(0),
                Error::MemberBalanceNotZero
            );
            ensure!(
                self.member_lock_balances
                    .get(caller)
                    .unwrap_or(U256::from(0))
                    == U256::from(0),
                Error::MemberBalanceNotZero
            );

            // remove user from DAO
            self.member_balances.remove(caller);
            self.member_lock_balances.remove(caller);
            self.members.retain(|x| *x != caller);

            Ok(())
        }

        /// Leave DAO and burn all balance (including locked balance)
        /// 离开 DAO 并销毁所有余额（包括锁定余额）
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MemberNotExisted` - User is not a member / 用户不是成员
        /// * `LowBalance` - Total issuance underflow / 总发行量下溢
        #[ink(message)]
        fn levae_with_burn(&mut self) -> Result<(), Error> {
            let caller = self.env().caller();

            // check if user is already an member
            ensure!(
                self.member_balances.contains(caller),
                Error::MemberNotExisted
            );

            // get amount of user
            let amount = self.member_balances.get(caller).unwrap_or(U256::from(0))
                + self
                    .member_lock_balances
                    .get(caller)
                    .unwrap_or(U256::from(0));
            ensure!(self.total_issuance >= amount, Error::LowBalance);
            self.total_issuance -= amount;

            // remove user from DAO
            self.member_balances.remove(caller);
            self.member_lock_balances.remove(caller);
            self.members.retain(|x| *x != caller);

            Ok(())
        }

        /// Delete member from DAO and burn all balance (governance only)
        /// 从 DAO 删除成员并销毁所有余额（仅治理）
        /// 
        /// # Arguments
        /// * `user` - Address of the member to delete / 要删除的成员地址
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MustCallByGov` - Must be called by governance / 必须由治理调用
        /// * `MemberNotExisted` - User is not a member / 用户不是成员
        /// * `LowBalance` - Total issuance underflow / 总发行量下溢
        #[ink(message)]
        fn delete(&mut self, user: Address) -> Result<(), Error> {
            self.ensure_from_gov()?;

            // check if user is an member
            ensure!(self.member_balances.contains(user), Error::MemberNotExisted);

            // get amount of user
            let amount = self.member_balances.get(user).unwrap_or(U256::from(0))
                + self.member_lock_balances.get(user).unwrap_or(U256::from(0));
            ensure!(self.total_issuance >= amount, Error::LowBalance);
            self.total_issuance -= amount;

            // remove user from DAO
            self.member_balances.remove(user);
            self.member_lock_balances.remove(user);
            self.members.retain(|x| *x != user);

            Ok(())
        }
    }

    impl Erc20 for DAO {
        /// Returns the token name (ERC20 standard)
        /// 返回代币名称（ERC20 标准）
        /// 
        /// # Returns
        /// * `Vec<u8>` - Token name / 代币名称
        #[ink(message)]
        fn name(&self) -> Vec<u8> {
            return Vec::new();
        }

        /// Returns the token symbol (ERC20 standard)
        /// 返回代币符号（ERC20 标准）
        /// 
        /// # Returns
        /// * `Vec<u8>` - Token symbol / 代币符号
        #[ink(message)]
        fn symbol(&self) -> Vec<u8> {
            return Vec::new();
        }

        /// Returns the number of decimals (ERC20 standard)
        /// 返回小数位数（ERC20 标准）
        /// 
        /// # Returns
        /// * `u8` - Number of decimals (12) / 小数位数（12）
        #[ink(message)]
        fn decimals(&self) -> u8 {
            12
        }

        /// Returns the total token supply (ERC20 standard)
        /// 返回代币总供应量（ERC20 标准）
        /// 
        /// # Returns
        /// * `U256` - Total token supply / 代币总供应量
        #[ink(message)]
        fn total_supply(&self) -> U256 {
            self.total_issuance
        }

        /// Returns the free (unlocked) balance of an account (ERC20 standard)
        /// 返回账户的可用（未锁定）余额（ERC20 标准）
        /// 
        /// # Arguments
        /// * `owner` - Address of the account / 账户地址
        /// 
        /// # Returns
        /// * `U256` - Free balance (total balance - locked balance) / 可用余额（总余额 - 锁定余额）
        #[ink(message)]
        fn balance_of(&self, owner: Address) -> U256 {
            self.free_balance(owner)
        }

        /// Transfer tokens from caller to another address (ERC20 standard)
        /// 从调用者向另一个地址转账代币（ERC20 标准）
        /// 
        /// # Arguments
        /// * `to` - Recipient address / 接收者地址
        /// * `value` - Amount to transfer / 转账金额
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `TransferDisable` - Transfer is disabled / 转账已禁用
        /// * `MemberNotExisted` - Caller or recipient is not a member / 调用者或接收者不是成员
        /// * `LowBalance` - Insufficient free balance / 可用余额不足
        #[ink(message)]
        fn transfer(&mut self, to: Address, value: U256) -> Result<(), Error> {
            ensure!(self.transfer, Error::TransferDisable);

            let caller = self.env().caller();

            // check if user is an member
            ensure!(
                self.member_balances.contains(caller),
                Error::MemberNotExisted
            );
            ensure!(self.member_balances.contains(to), Error::MemberNotExisted);

            let free = self.free_balance(caller);
            ensure!(free >= value, Error::LowBalance);

            let caller_balance = self.member_balances.get(caller).unwrap();
            let to_balance = self.member_balances.get(to).unwrap_or(U256::from(0));
            
            self.member_balances.insert(caller, &(caller_balance - value));
            self.member_balances.insert(to, &(to_balance + value));

            Ok(())
        }

        /// Transfer tokens from one address to another using allowance (ERC20 standard)
        /// 使用授权额度从一个地址向另一个地址转账代币（ERC20 标准）
        /// 
        /// # Arguments
        /// * `from` - Address to transfer from / 转出地址
        /// * `to` - Address to transfer to / 转入地址
        /// * `value` - Amount to transfer / 转账金额
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `TransferDisable` - Transfer is disabled / 转账已禁用
        /// * `InsufficientAllowance` - Insufficient allowance / 授权额度不足
        /// * `MemberNotExisted` - From or to address is not a member / 转出或转入地址不是成员
        /// * `LowBalance` - Insufficient free balance / 可用余额不足
        #[ink(message)]
        fn transfer_from(&mut self, from: Address, to: Address, value: U256) -> Result<(), Error> {
            ensure!(self.transfer, Error::TransferDisable);

            let spender = self.env().caller();

            let allowance = self.allowances.get((from, spender)).unwrap_or_default();
            ensure!(allowance >= value, Error::InsufficientAllowance);

            let free = self.free_balance(from);
            ensure!(free >= value, Error::LowBalance);

            let from_balance = self.member_balances.get(from).unwrap();
            let to_balance = self.member_balances.get(to).unwrap_or(U256::from(0));
            
            self.member_balances.insert(from, &(from_balance - value));
            self.member_balances.insert(to, &(to_balance + value));
            self.allowances
                .insert((from, spender), &(allowance - value));

            Ok(())
        }

        /// Approve spender to transfer tokens on behalf of caller (ERC20 standard)
        /// 授权支出者代表调用者转账代币（ERC20 标准）
        /// 
        /// # Arguments
        /// * `spender` - Address to approve / 被授权的地址
        /// * `value` - Amount to approve / 授权金额
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        #[ink(message)]
        fn approve(&mut self, spender: Address, value: U256) -> Result<(), Error> {
            let caller = self.env().caller();

            self.allowances.insert((caller, spender), &value);
            Ok(())
        }

        /// Returns the amount of tokens that spender is allowed to transfer from owner (ERC20 standard)
        /// 返回支出者被允许从所有者转账的代币数量（ERC20 标准）
        /// 
        /// # Arguments
        /// * `owner` - Token owner address / 代币所有者地址
        /// * `spender` - Spender address / 支出者地址
        /// 
        /// # Returns
        /// * `U256` - Allowance amount / 授权额度
        #[ink(message)]
        fn allowance(&mut self, owner: Address, spender: Address) -> U256 {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Burn tokens from caller's balance (reduces total supply)
        /// 销毁调用者余额中的代币（减少总供应量）
        /// 
        /// # Arguments
        /// * `amount` - Amount of tokens to burn / 要销毁的代币数量
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `LowBalance` - Insufficient free balance / 可用余额不足
        #[ink(message)]
        fn burn(&mut self, amount: U256) -> Result<(), Error> {
            let caller = self.env().caller();
            let free = self.free_balance(caller);

            ensure!(free >= amount, Error::LowBalance);

            let total = self.member_balances.get(caller).unwrap();
            self.member_balances.insert(caller, &(total - amount));
            self.total_issuance -= amount;

            Ok(())
        }

        /// Returns the locked balance of an account (used for voting)
        /// 返回账户的锁定余额（用于投票）
        /// 
        /// # Arguments
        /// * `owner` - Address of the account / 账户地址
        /// 
        /// # Returns
        /// * `U256` - Locked balance / 锁定余额
        #[ink(message)]
        fn lock_balance_of(&self, owner: Address) -> U256 {
            self.member_lock_balances
                .get(owner)
                .unwrap_or(U256::from(0))
        }
    }

    impl Sudo for DAO {
        /// Execute a call with sudo privileges (sudo account only)
        /// 使用 sudo 权限执行调用（仅 sudo 账户）
        /// 
        /// If sudo is enabled, sudo account can call any function without governance approval.
        /// 如果启用了 sudo，sudo 账户可以在无需治理批准的情况下调用任何函数。
        /// 
        /// # Arguments
        /// * `call` - Call to execute / 要执行的调用
        /// 
        /// # Returns
        /// * `Result<Vec<u8>, Error>` - Execution result / 执行结果
        /// 
        /// # Errors
        /// * `CallFailed` - Not sudo account or call execution failed / 不是 sudo 账户或调用执行失败
        #[ink(message)]
        fn sudo(&mut self, call: Call) -> Result<Vec<u8>, Error> {
            let caller = self.env().caller();

            // Only sudo account can call sudo
            if self.sudo_account.is_none() || self.sudo_account.unwrap() != caller {
                return Err(Error::CallFailed);
            }

            // Insert call into sudo history
            let call_id = self.sudo_calls.insert(&call.clone());
            let result = self.exec_call(call);
            self.env().emit_event(SudoExecution {
                sudo_id: call_id,
                result: result.clone().map(Some),
            });

            result
        }

        /// Remove sudo account (governance only)
        /// 删除 sudo 账户（仅治理）
        /// 
        /// After ensuring stable operation of DAO, delete sudo to make it fully decentralized.
        /// 在确保 DAO 稳定运行后，删除 sudo 以使其完全去中心化。
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MustCallByGov` - Must be called by governance / 必须由治理调用
        #[ink(message)]
        fn remove_sudo(&mut self) -> Result<(), Error> {
            self.ensure_from_gov()?;

            self.sudo_account = None;

            Ok(())
        }
    }

    impl Gov for DAO {
        /// set default track, all gov proposal will use this track if no track is specified
        #[ink(message)]
        fn set_defalut_track(&mut self, id: u16) -> Result<(), Error> {
            self.ensure_from_gov()?;

            ensure!(self.tracks.contains(&id), Error::NoTrack);

            self.defalut_track = Some(id);

            Ok(())
        }

        #[ink(message)]
        fn defalut_track(&self) -> Option<u16> {
            self.defalut_track
        }

        #[ink(message)]
        fn track_list(&self, page: u16, size: u16) -> Vec<Track> {
            let mut list = Vec::new();
            let start = (page - 1) * size;
            for i in start..start + size {
                if let Some(track) = self.tracks.get(i) {
                    list.push(track);
                }
            }
            list
        }

        #[ink(message)]
        fn track(&self, id: u16) -> Option<Track> {
            self.tracks.get(id)
        }

        /// add a new vote track
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
        ) -> Result<(), Error> {
            self.ensure_from_gov()?;

            // let id = self.tracks_helper.next_id;
            let approval = arg_to_curve(min_approval);
            let support = arg_to_curve(min_support);
            self.tracks.insert(&Track {
                name,
                prepare_period,
                decision_deposit,
                max_deciding,
                confirm_period,
                decision_period,
                min_enactment_period,
                max_balance,
                min_approval: approval,
                min_support: support,
            });

            Ok(())
        }

        /// edit a track
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
        ) -> Result<(), Error> {
            self.ensure_from_gov()?;

            ensure!(self.tracks.contains(&id), Error::NoTrack);

            let approval = arg_to_curve(min_approval);
            let support = arg_to_curve(min_support);
            self.tracks.update(
                id,
                &Track {
                    name,
                    prepare_period,
                    decision_deposit,
                    max_deciding,
                    confirm_period,
                    decision_period,
                    min_enactment_period,
                    max_balance,
                    min_approval: approval,
                    min_support: support,
                },
            );

            Ok(())
        }

        #[ink(message)]
        fn proposals(&self, page: u16, size: u16) -> Vec<Call> {
            let mut list = Vec::new();
            let start = ((page - 1) * size) as u32;
            for i in start..start + size as u32 {
                if let Some(proposal) = self.proposals.get(i as u32) {
                    list.push(proposal);
                }
            }
            list
        }

        #[ink(message)]
        fn proposal(&self, id: u32) -> Option<Call> {
            self.proposals.get(id)
        }

        /// Submit a proposal to DAO for governance voting
        /// 向 DAO 提交提案以供治理投票
        /// 
        /// # Arguments
        /// * `call` - Call to execute if proposal passes / 提案通过后要执行的调用
        /// * `track_id` - Voting track ID to use / 要使用的投票轨道 ID
        /// 
        /// # Returns
        /// * `Result<CalllId, Error>` - Proposal ID if successful / 成功返回提案 ID
        /// 
        /// # Errors
        /// * `MemberNotExisted` - Caller is not a member / 调用者不是成员
        /// * `NoTrack` - Track ID is invalid / 轨道 ID 无效
        /// * `MaxBalanceOverflow` - Call amount exceeds track max balance / 调用金额超过轨道最大余额
        #[ink(message)]
        fn submit_proposal(&mut self, call: Call, track_id: u16) -> Result<CalllId, Error> {
            let caller = self.env().caller();

            // check if user is an member
            ensure!(
                self.member_balances.contains(caller),
                Error::MemberNotExisted
            );

            //  check track of call
            let tracks = self.get_track_id(&call);
            ensure!(tracks.contains(&track_id), Error::NoTrack);

            // check call amount
            let track = self.tracks.get(track_id).unwrap();
            ensure!(call.amount <= track.max_balance, Error::MaxBalanceOverflow);

            // save proposal
            let call_id = self.proposals.insert(&call);

            // set caller of proposal
            self.proposal_caller.insert(call_id, &caller);

            // set track for proposal
            self.track_of_proposal.insert(call_id, &track_id);

            // set proposal status
            self.status_of_proposal
                .insert(call_id, &PropStatus::Pending);

            // set submit block number for proposal
            self.submit_block_of_proposal
                .insert(call_id, &self.env().block_number());

            // emit event
            self.env().emit_event(ProposalSubmission {
                proposal_id: call_id,
            });

            Ok(call_id)
        }

        /// Cancel a proposal
        #[ink(message)]
        fn cancel_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            let caller = self.env().caller();

            ensure!(
                self.status_of_proposal.get(proposal_id) == Some(PropStatus::Pending),
                Error::InvalidProposalStatus
            );

            ensure!(
                self.proposal_caller.get(proposal_id).unwrap_or_default() == caller,
                Error::InvalidProposalCaller
            );

            self.status_of_proposal
                .insert(proposal_id, &PropStatus::Canceled);

            Ok(())
        }

        /// Confirm a proposal by depositing tokens (payable)
        /// 通过存入代币确认提案（可支付）
        /// 
        /// This moves the proposal from Pending to Ongoing status and starts the voting period.
        /// 这将提案从待处理状态移动到进行中状态，并开始投票期。
        /// 
        /// # Arguments
        /// * `proposal_id` - ID of the proposal to confirm / 要确认的提案 ID
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `InvalidProposalStatus` - Proposal is not in Pending status / 提案不在待处理状态
        /// * `InvalidDepositTime` - Too early to deposit (prepare period not passed) / 存入时间过早（准备期未过）
        /// * `InvalidDeposit` - Deposit amount is less than required / 存入金额少于要求
        #[ink(message, payable)]
        fn deposit_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            let caller = self.env().caller();
            let payvalue = self.env().transferred_value();

            // check status
            let status = self
                .status_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            if status != PropStatus::Pending {
                return Err(Error::InvalidProposalStatus);
            }

            let deposit = self
                .deposit_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;

            // check track
            let track = self.get_track(proposal_id)?;
            let now = self.env().block_number();
            if now < deposit.2 + track.prepare_period {
                return Err(Error::InvalidDepositTime);
            }

            // check payvalue
            if payvalue < track.decision_deposit {
                return Err(Error::InvalidDeposit);
            }

            // save deposit
            self.deposit_of_proposal
                .insert(proposal_id, &(caller, payvalue, now));

            // save status
            self.status_of_proposal
                .insert(proposal_id, &PropStatus::Ongoing);

            Ok(())
        }

        #[ink(message)]
        fn vote_list(&self, proposal_id: CalllId) -> Vec<VoteInfo> {
            let ids = self.votes_of_proposal.desc_list(proposal_id, None, 1000);
            let mut list = Vec::new();
            for id in ids {
                let info = self.votes.get(id.1);
                if info.is_none() {
                    continue;
                }
                list.push(info.unwrap());
            }

            list
        }

        #[ink(message)]
        fn vote(&mut self, vote_id: u64) -> Option<VoteInfo> {
            self.votes.get(vote_id)
        }

        /// Vote for a proposal (payable - the transferred value is locked as voting weight)
        /// 对提案投票（可支付 - 转账的金额将作为投票权重被锁定）
        /// 
        /// # Arguments
        /// * `proposal_id` - ID of the proposal to vote on / 要投票的提案 ID
        /// * `opinion` - Vote opinion (YES or NO) / 投票意见（是或否）
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MemberNotExisted` - Caller is not a member / 调用者不是成员
        /// * `LowBalance` - Insufficient free balance / 可用余额不足
        /// * `InvalidProposalStatus` - Proposal is not in Ongoing status / 提案不在进行中状态
        /// * `PropNotOngoing` - Proposal is not ongoing / 提案未在进行中
        /// * `InvalidVoteTime` - Voting period has ended / 投票期已结束
        #[ink(message)]
        fn submit_vote(&mut self, proposal_id: CalllId, opinion: Opinion) -> Result<(), Error> {
            let caller = self.env().caller();

            // check token
            let payvalue = self.env().transferred_value();
            let total = self
                .member_balances
                .get(caller)
                .ok_or(Error::MemberNotExisted)?;
            let lock = self
                .member_lock_balances
                .get(caller)
                .unwrap_or(U256::from(0));
            if total - lock < payvalue {
                return Err(Error::LowBalance);
            }

            // check status
            let status = self
                .status_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            if status != PropStatus::Ongoing {
                return Err(Error::PropNotOngoing);
            }

            // check time
            let deposit_block = self
                .submit_block_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            let track = self.get_track(proposal_id)?;
            let now = self.env().block_number();
            if now > deposit_block + track.max_deciding {
                return Err(Error::InvalidVoteTime);
            }

            let vid = self.votes.next_id();
            let vote = VoteInfo {
                pledge: payvalue,
                opinion,
                vote_weight: 1u32.into(),
                unlock_block: 1u32.into(),
                call_id: proposal_id,
                calller: caller,
                vote_block: now,
                deleted: false,
            };

            // lock token
            self.member_lock_balances.insert(caller, &(lock + payvalue));

            // save vote
            self.vote_of_member.insert(caller, &vid);
            self.votes_of_proposal.insert(proposal_id, &vid);

            self.votes.insert(&vote);

            Ok(())
        }

        /// Cancel vote before proposal is executed or rejected
        #[ink(message)]
        fn cancel_vote(&mut self, vote_id: u64) -> Result<(), Error> {
            let caller = self.env().caller();

            let mut vote = self.votes.get(vote_id).ok_or(Error::InvalidVote)?;

            // check vote user
            if vote.calller != caller {
                return Err(Error::InvalidVoteUser);
            }

            // check proposal status
            let proposal_id = vote.call_id;
            let status = self
                .status_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            if status != PropStatus::Ongoing {
                return Err(Error::PropNotOngoing);
            }

            vote.deleted = true;
            self.votes.update(vote_id, &vote);

            // unlock token
            let lock = self
                .member_lock_balances
                .get(caller)
                .unwrap_or(U256::from(0));
            ensure!(lock >= vote.pledge, Error::LowBalance);
            self.member_lock_balances
                .insert(caller, &(lock - vote.pledge));

            Ok(())
        }

        /// Unlock tokens after proposal is executed or rejected
        #[ink(message)]
        fn unlock(&mut self, vote_id: u64) -> Result<(), Error> {
            let caller = self.env().caller();

            // check vote unlock status
            if self.unlock_of_votes.contains(vote_id) {
                return Err(Error::VoteAlreadyUnlocked);
            }

            let vote = self.votes.get(vote_id).ok_or(Error::InvalidVote)?;

            // check vote status
            if vote.deleted {
                return Err(Error::InvalidVoteStatus);
            }

            // check vote user
            if vote.calller != caller {
                return Err(Error::InvalidVoteUser);
            }

            let proposal_id = vote.call_id;

            // check vote unlock time
            let end_block = self.calculate_proposal_end_block(proposal_id)?;
            let now = self.env().block_number();
            if now < end_block + vote.unlock_block {
                return Err(Error::InvalidVoteUnlockTime);
            }

            // unlock token
            let lock = self
                .member_lock_balances
                .get(caller)
                .unwrap_or(U256::from(0));
            ensure!(lock >= vote.pledge, Error::LowBalance);
            self.member_lock_balances
                .insert(caller, &(lock - vote.pledge));
            self.unlock_of_votes.insert(vote_id, &());

            Ok(())
        }

        /// Execute proposal after vote is passed
        #[ink(message)]
        fn exec_proposal(&mut self, proposal_id: CalllId) -> Result<Vec<u8>, Error> {
            let call = self
                .proposals
                .get(proposal_id)
                .ok_or(Error::InvalidProposal)?;

            // check status
            let status = self
                .status_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            if status != PropStatus::Ongoing {
                return Err(Error::PropNotOngoing);
            }

            let (is_confirm, end, track) = self.calculate_proposal_status(proposal_id)?;
            let now = self.env().block_number();
            if !is_confirm {
                if now > end {
                    self.status_of_proposal
                        .insert(proposal_id, &PropStatus::Rejected(end));
                }
                return Err(Error::ProposalNotConfirmed);
            }

            if now < end + track.decision_period {
                return Err(Error::ProposalInDecision);
            }

            //  Set the status to approved first (CEI pattern: Checks-Effects-Interactions)
            self.status_of_proposal
                .insert(proposal_id, &PropStatus::Approved(now));

            // Return the deposit amount.
            let deposit = self.deposit_of_proposal.get(proposal_id).unwrap();
            let result = self.env().transfer(deposit.0, deposit.1);
            if result.is_err() {
                return Err(Error::TransferFailed);
            }

            let result = self.exec_call(call);
            self.env().emit_event(ProposalExecution {
                proposal_id,
                result: result.clone().map(Some),
            });

            result
        }

        /// Calculate proposal status
        #[ink(message)]
        fn proposal_status(&self, proposal_id: CalllId) -> Result<PropStatus, Error> {
            if !self.status_of_proposal.contains(proposal_id) {
                return Err(Error::InvalidProposal);
            }

            // check status
            let status = self
                .status_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            if status != PropStatus::Ongoing {
                return Ok(status);
            }

            let (is_confirm, end_block, _) = self.calculate_proposal_status(proposal_id)?;
            if !is_confirm {
                let now = self.env().block_number();
                if now > end_block {
                    return Ok(PropStatus::Rejected(end_block));
                }
                return Ok(PropStatus::Ongoing);
            }

            return Ok(PropStatus::Approved(0));
        }
    }

    impl Treasury for DAO {
        #[ink(message)]
        fn spend(
            &mut self,
            track_id: u16,
            to: Address,
            _assert_id: u32,
            amount: U256,
        ) -> Result<u64, Error> {
            let caller = self.env().caller();

            // check if user is an member
            ensure!(
                self.member_balances.contains(caller),
                Error::MemberNotExisted
            );

            let id = self.next_spend_id;
            self.spends.insert(
                id,
                &Spend {
                    caller,
                    amount,
                    to: to,
                    payout: false,
                },
            );

            let call_args = ArgumentList::empty().push_arg(&id);

            let call = Call {
                contract: None,
                selector: [159, 160, 185, 236],
                input: call_args.encode(),
                amount: amount,
                ref_time_limit: u64::MAX,
                allow_reentry: false,
            };

            self.submit_proposal(call, track_id)?;
            self.next_spend_id += 1;

            Ok(id)
        }

        #[ink(message)]
        fn payout(&mut self, spend_index: u64) -> Result<(), Error> {
            self.ensure_from_gov()?;

            // check spend
            let mut spend = self.spends.get(spend_index).ok_or(Error::SpendNotFound)?;
            ensure!(!spend.payout, Error::SpendAlreadyExecuted);

            // transfer token
            ok_or_err!(
                self.env().transfer(spend.to, spend.amount),
                Error::SpendTransferError
            );

            // save state
            spend.payout = true;
            self.spends.insert(spend_index, &spend);
            Ok(())
        }
    }

    impl DAO {
        /// create a new dao
        #[ink(constructor)]
        pub fn new(
            users: Vec<(Address, U256)>,
            public_join: bool,
            sudo_account: Option<Address>,
            track: Option<Track>,
        ) -> Self {
            let mut dao = DAO::default();
            let mut members = Vec::new();
            let mut member_balances = Mapping::default();
            let mut total_issuance = U256::from(0);
            let mut tracks: Tracks = Default::default();

            // init members balances
            for (user, balance) in users.iter() {
                member_balances.insert(*user, balance);
                members.push(*user);
                total_issuance = total_issuance
                    .checked_add(*balance)
                    .expect("issuance overflow");
            }

            // init DAO
            dao.members = members;
            dao.member_balances = member_balances;
            dao.total_issuance = total_issuance;
            dao.sudo_account = sudo_account;
            dao.public_join = public_join;

            if track.is_some() {
                // init vote track
                tracks.insert(&track.unwrap());
                dao.tracks = tracks;
                dao.defalut_track = Some(0);
            }

            dao
        }

        /// create a new dao with gov track
        #[ink(constructor)]
        pub fn new_with_track(
            users: Vec<(Address, U256)>,
            public_join: bool,
            sudo_account: Option<Address>,
            track: Track,
        ) -> Self {
            DAO::new(users, public_join, sudo_account, Some(track))
        }

        /// create a new dao with default gov track
        #[ink(constructor)]
        pub fn new_with_default_track(
            users: Vec<(Address, U256)>,
            public_join: bool,
            sudo_account: Option<Address>,
        ) -> Self {
            let track = Track {
                name: Vec::new(),
                prepare_period: 1,
                max_deciding: 1,
                confirm_period: 1,
                decision_period: 1,
                min_enactment_period: 1,
                decision_deposit: U256::from(1),
                max_balance: U256::from(1),
                min_approval: Curve::LinearDecreasing {
                    begin: 10000,
                    end: 5000,
                    length: 30,
                },
                min_support: Curve::LinearDecreasing {
                    begin: 10000,
                    end: 50,
                    length: 30,
                },
            };

            DAO::new_with_track(users, public_join, sudo_account, track)
        }

        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_gov()?;
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        /// Gov call only call from contract
        fn ensure_from_gov(&self) -> Result<(), Error> {
            ensure!(
                self.env().caller() == self.env().address(),
                Error::MustCallByGov
            );

            Ok(())
        }

        /// Get track rule of call
        fn get_track_id(&self, call: &Call) -> Vec<u16> {
            let mut ids = Vec::new();

            let id = self
                .track_rules
                .get((call.contract.clone(), Some(call.selector.clone())));
            if id.is_some() {
                ids.push(id.unwrap());
            }

            let contract_id = self
                .track_rules
                .get((call.contract.clone(), None::<Selector>));

            if contract_id.is_some() {
                ids.push(contract_id.unwrap());
            }

            if self.defalut_track.is_some() {
                ids.push(self.defalut_track.unwrap());
            }

            return ids;
        }

        /// Get track of call
        fn get_track(&self, proposal_id: CalllId) -> Result<Track, Error> {
            let track_id = self
                .track_of_proposal
                .get(proposal_id)
                .ok_or(Error::NoTrack)?;
            let track = self.tracks.get(track_id).ok_or(Error::NoTrack)?;

            Ok(track)
        }

        /// Calculate proposal status
        fn calculate_proposal_status(
            &self,
            proposal_id: CalllId,
        ) -> Result<(bool, BlockNumber, Track), Error> {
            // get votes
            let vote_ids = self.votes_of_proposal.list(proposal_id, 1, 10000000);
            let mut votes = Vec::new();
            for id in vote_ids {
                if let Some(vote) = self.votes.get(id.1) {
                    votes.push(vote);
                }
            }

            // get track
            let track = self.get_track(proposal_id)?;

            // get vote begin and end time
            let (_, _, begin) = self.deposit_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            let end = begin + track.max_deciding;
            let confirm_period = track.confirm_period;
            let all = self.total_issuance;

            // statistical results
            let mut yes = U256::from(0);
            let mut no = U256::from(0);
            let mut support = U256::from(0);
            let mut is_confirm = false;
            let mut last_achieve_block: BlockNumber = 0;

            for vote in votes {
                if vote.deleted {
                    continue;
                }

                // calculate min
                let min_approval = U256::from(track.min_approval.y(vote.vote_block));
                let min_support = U256::from(track.min_support.y(vote.vote_block));

                // calculate vote info
                support += vote.pledge;
                match vote.opinion {
                    Opinion::YES => {
                        yes += vote.pledge * vote.vote_weight;
                    }
                    Opinion::NO => {
                        no += vote.pledge * vote.vote_weight;
                    }
                }

                // 防止除零错误
                if no > U256::from(0) && all > U256::from(0) {
                    let approval_ratio = yes * 10000 / no;
                    let support_ratio = support * 10000 / all;
                    
                    if approval_ratio >= min_approval && support_ratio >= min_support {
                        if vote.vote_block - last_achieve_block > confirm_period {
                            is_confirm = true;
                            break;
                        }

                        // 记录上次成功的投票块
                        if last_achieve_block == 0 {
                            last_achieve_block = vote.vote_block;
                        }
                    } else {
                        last_achieve_block = 0;
                    }
                }
            }

            Ok((is_confirm, end, track))
        }

        /// Calculate proposal end block
        fn calculate_proposal_end_block(&self, proposal_id: CalllId) -> Result<BlockNumber, Error> {
            let status = self
                .status_of_proposal
                .get(proposal_id)
                .ok_or(Error::InvalidProposalStatus)?;
            match status {
                PropStatus::Ongoing => {
                    let (is_confirm, end, _) = self.calculate_proposal_status(proposal_id)?;
                    if !is_confirm {
                        let now = self.env().block_number();
                        if now > end {
                            return Ok(end);
                        }
                    }
                    return Err(Error::InvalidProposalStatus);
                }
                PropStatus::Rejected(b) => {
                    return Ok(b);
                }
                PropStatus::Approved(b) => {
                    return Ok(b);
                }
                _ => {
                    return Err(Error::InvalidProposalStatus);
                }
            }
        }

        /// Run call
        pub fn exec_call(&mut self, call: Call) -> Result<Vec<u8>, Error> {
            let call_flags = if call.allow_reentry {
                CallFlags::ALLOW_REENTRY
            } else {
                CallFlags::empty()
            };

            let result = build_call::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(call.contract.unwrap_or(self.env().address()))
                .ref_time_limit(call.ref_time_limit)
                .transferred_value(call.amount)
                .call_flags(call_flags)
                .exec_input(
                    ExecutionInput::new(call.selector.into()).push_arg(CallInput(&call.input)),
                )
                .returns::<Vec<u8>>()
                .try_invoke();

            match result {
                Ok(Ok(v)) => Ok(v),
                _ => Err(Error::CallFailed),
            }
        }

        // get free balance
        fn free_balance(&self, owner: Address) -> U256 {
            let balance = self.member_balances.get(owner).unwrap_or(U256::from(0));
            let lock = self
                .member_lock_balances
                .get(owner)
                .unwrap_or(U256::from(0));

            balance - lock
        }
    }
}

#[cfg(test)]
mod tests;

// #[cfg(all(test, feature = "e2e-tests"))]
#[cfg(test)]
mod e2e_tests;
