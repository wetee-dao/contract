#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod curve;
mod datas;
mod errors;
mod events;
mod traits;

#[ink::contract]
mod dao {
    use crate::{curve::{arg_to_curve, Curve, CurveArg}, datas::*, errors::Error, events::*, traits::*};
    use ink::{
        env::{
            call::{build_call, ExecutionInput},
            CallFlags,
        },
        prelude::vec::Vec,
        storage::Mapping,
        U256,
    };
    use primitives::{CallInput, ListHelper};

    #[ink(storage)]
    #[derive(Default)]
    pub struct DAO {
        /// proposals
        proposals: Mapping<CalllId, Call>,
        /// track of proposal
        track_of_proposal: Mapping<CalllId, u16>,
        /// proposals list helper
        proposals_helper: ListHelper<CalllId>,
        /// caller of proposal
        proposal_caller: Mapping<CalllId, Address>,
        /// deposit of proposal
        deposit_of_proposal: Mapping<CalllId, (Address, U256, BlockNumber)>,
        /// status of proposal
        status_of_proposal: Mapping<CalllId, PropStatus>,
        /// votes of proposal
        votes_of_proposal: Mapping<CalllId, Vec<u128>>,
        /// submit block number
        submit_block_of_proposal: Mapping<CalllId, BlockNumber>,

        /// tracks
        tracks: Mapping<u16, Track>,
        /// tracks list helper
        tracks_helper: ListHelper<u16>,

        /// track rules (If selector == none, it means entire contract uses a single track)
        track_rules: Mapping<(Option<Address>, Option<Selector>), u16>,
        /// track rules index
        track_rule_index: Mapping<u16, (Option<Address>, Option<Selector>, u16)>,
        /// track rules index helper
        track_rule_index_helper: ListHelper<u16>,
        /// default track
        defalut_track: Option<u16>,

        /// vote of proposal
        votes: Mapping<u128, VoteInfo>,
        /// proposals list helper
        votes_helper: ListHelper<u128>,
        /// votes of member
        vote_of_member: Mapping<Address, Vec<u128>>,
        /// lock of votes
        unlock_of_votes: Mapping<u128, ()>,

        /// sudo call history
        sudo_calls: Mapping<CalllId, Call>,
        /// next sudo call id
        sudo_helper: ListHelper<CalllId>,
        /// sudo account
        sudo_account: Option<Address>,

        /// members
        members: Vec<Address>,

        /// member balance
        member_balances: Mapping<Address, U256>,
        /// member lock balance
        member_lock_balances: Mapping<Address, U256>,
        /// total issuance TOKEN
        total_issuance: U256,

        /// transfer enable
        transfer: bool,
    }

    impl Member for DAO {
        /// Returns list of members.
        #[ink(message)]
        fn members(&self) -> Vec<Address> {
            self.members.clone()
        }

        /// Join to DAO
        #[ink(message)]
        fn join(&mut self, new_user: Address, balance: U256) {
            self.ensure_from_gov();

            // check if user is already an member
            assert!(!self.member_balances.contains(new_user));

            self.member_balances.insert(new_user, &balance);
            self.members.push(new_user);

            self.env().emit_event(MemberAdd { user: new_user });
        }

        // levae DAO
        #[ink(message)]
        fn levae(&mut self) {
            let caller = self.env().caller();

            // check if user is already an member
            assert!(self.member_balances.contains(caller), "member not found");
            assert!(
                self.member_balances.get(caller).unwrap_or(U256::from(0)) == U256::from(0),
                "member balance not zero"
            );
            assert!(
                self.member_lock_balances
                    .get(caller)
                    .unwrap_or(U256::from(0))
                    == U256::from(0),
                "member lock balance not zero"
            );

            // remove user from DAO
            self.member_balances.remove(caller);
            self.member_lock_balances.remove(caller);
            self.members.retain(|x| *x != caller);
        }

        // levae DAO
        #[ink(message)]
        fn levae_with_burn(&mut self) {
            let caller = self.env().caller();

            // check if user is already an member
            assert!(self.member_balances.contains(caller), "member not found");

            // get amount of user
            let amount = self.member_balances.get(caller).unwrap_or(U256::from(0))
                + self
                    .member_lock_balances
                    .get(caller)
                    .unwrap_or(U256::from(0));
            self.total_issuance -= amount;

            // remove user from DAO
            self.member_balances.remove(caller);
            self.member_lock_balances.remove(caller);
            self.members.retain(|x| *x != caller);
        }

        /// Delete member from DAO
        #[ink(message)]
        fn delete_member(&mut self, user: Address) {
            self.ensure_from_gov();

            // check if user is an member
            assert!(self.member_balances.contains(user));

            // get amount of user
            let amount = self.member_balances.get(user).unwrap_or(U256::from(0))
                + self.member_lock_balances.get(user).unwrap_or(U256::from(0));
            self.total_issuance -= amount;

            // remove user from DAO
            self.member_balances.remove(user);
            self.member_lock_balances.remove(user);
            self.members.retain(|x| *x != user);
        }
    }

    impl Erc20 for DAO {
        #[ink(message)]
        fn balance_of(&self, user: Address) -> (U256, U256) {
            let balance = self.member_balances.get(user).unwrap_or(U256::from(0));
            let lock = self.member_lock_balances.get(user).unwrap_or(U256::from(0));

            (balance, lock)
        }

        /// Enable transfer
        #[ink(message)]
        fn enable_transfer(&mut self) {
            self.ensure_from_gov();

            if self.transfer {
                return;
            }

            self.transfer = true;
        }

        /// Transfer TOKEN to user
        #[ink(message)]
        fn transfer(&mut self, to: Address, amount: U256) {
            assert!(self.transfer);

            let caller = self.env().caller();

            // check if user is an member
            assert!(self.member_balances.contains(caller));
            assert!(self.member_balances.contains(to));

            let total = self.member_balances.get(caller).unwrap_or(U256::from(0));
            let lock = self
                .member_lock_balances
                .get(caller)
                .unwrap_or(U256::from(0));
            assert!(total - lock >= amount);

            self.member_balances.insert(caller, &(total - amount));
            self.member_balances.insert(
                to,
                &(self.member_balances.get(to).unwrap_or(U256::from(0)) + amount),
            );
        }

        /// Burn tokens from caller's balance.
        #[ink(message)]
        fn burn(&mut self, amount: U256) {
            let caller = self.env().caller();

            // check if user is an member
            assert!(self.member_balances.contains(caller));

            let total = self.member_balances.get(caller).unwrap();
            assert!(total >= amount);

            self.member_balances.insert(caller, &(total - amount));
            self.total_issuance -= amount;
        }
    }

    impl Sudo for DAO {
        /// If sudo is enabled, sudo account can call any function without gov
        #[ink(message)]
        fn sudo(&mut self, call: Call) -> Result<Vec<u8>, Error> {
            let caller = self.env().caller();

            // Only sudo account can call sudo
            if self.sudo_account.is_none() || self.sudo_account.unwrap() != caller {
                return Err(Error::CallFailed);
            }

            // Insert call into sudo history
            let call_id = self.sudo_helper.next_id;
            self.sudo_helper.next_id = call_id.checked_add(1).expect("call id overflow");
            self.sudo_calls.insert(call_id, &call);

            let result = self.exec_call(call);
            self.env().emit_event(SudoExecution {
                sudo_id: call_id,
                result: result.clone().map(Some),
            });

            result
        }

        /// After ensuring stable operation of DAO, delete sudo.
        #[ink(message)]
        fn remove_sudo(&mut self) {
            self.ensure_from_gov();

            self.sudo_account = None;
        }
    }

    impl Gov for DAO {
        #[ink(message)]
        fn set_defalut_track(&mut self, id: u16) -> Result<(), Error> {
            self.ensure_from_gov();

            assert!(self.tracks.contains(&id));

            self.defalut_track = Some(id);

            Ok(())
        }

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
            self.ensure_from_gov();

            let id = self.tracks_helper.next_id;
            let approval = arg_to_curve(min_approval);
            let support = arg_to_curve(min_support);
            self.tracks.insert(
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
            self.tracks_helper.next_id = id.checked_add(1).expect("track id overflow");
            self.tracks_helper.list.push(id);
            
            Ok(())
        }

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
            self.ensure_from_gov();

            assert!(self.tracks.contains(&id));

            let approval = arg_to_curve(min_approval);
            let support = arg_to_curve(min_support);
            self.tracks.insert(
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

        /// Submit a proposal to DAO
        #[ink(message)]
        fn submit_proposal(&mut self, call: Call) -> Result<CalllId, Error> {
            let caller = self.env().caller();

            // check if user is an member
            assert!(self.member_balances.contains(caller));

            //  get track of call
            let track_wrap = self.get_track_id(&call);
            if track_wrap.is_none() {
                return Err(Error::NoTrack);
            }
            let track = track_wrap.unwrap();
            
            // save proposal
            let call_id = self.proposals_helper.next_id;
            self.proposals_helper.next_id = call_id.checked_add(1).expect("proposal id overflow");
            self.proposals_helper.list.push(call_id);
            self.proposals.insert(call_id, &call);

            // set caller of proposal
            self.proposal_caller.insert(call_id, &caller);

            // set track for proposal
            self.track_of_proposal.insert(call_id, &track);

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

            assert!(
                self.status_of_proposal.get(proposal_id) == Some(PropStatus::Pending),
                "proposal is started, cannot cancel"
            );

            assert!(
                self.proposal_caller.get(proposal_id).unwrap_or_default() == caller,
                "only caller can cancel proposal"
            );

            self.status_of_proposal
                .insert(proposal_id, &PropStatus::Canceled);

            Ok(())
        }

        /// Confirm a proposal with deposit TOKEN.
        #[ink(message, payable)]
        fn deposit_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            let caller = self.env().caller();
            let payvalue = self.env().transferred_value();

            // check status
            let status = self.status_of_proposal.get(proposal_id).unwrap();
            if status != PropStatus::Pending {
                return Err(Error::InvalidProposalStatus);
            }

            let deposit = self.deposit_of_proposal.get(proposal_id).unwrap();

            // check track
            let track = self.get_track(proposal_id);
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

        /// Vote for a proposal
        #[ink(message)]
        fn vote(&mut self, proposal_id: CalllId, opinion: Opinion) -> Result<(), Error> {
            let caller = self.env().caller();

            // check token
            let payvalue = self.env().transferred_value();
            let total = self.member_balances.get(caller).unwrap();
            let lock = self
                .member_lock_balances
                .get(caller)
                .unwrap_or(U256::from(0));
            if total - lock < payvalue {
                return Err(Error::LowBalance);
            }

            // check status
            if self.status_of_proposal.get(proposal_id).unwrap() != PropStatus::Ongoing {
                return Err(Error::PropNotOngoing);
            }

            // check time
            let deposit_block = self.submit_block_of_proposal.get(proposal_id).unwrap();
            let track = self.get_track(proposal_id);
            let now = self.env().block_number();
            if now > deposit_block + track.max_deciding {
                return Err(Error::InvalidVoteTime);
            }

            let vid = self.votes_helper.next_id;
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
            let mut votes = self.vote_of_member.get(caller).unwrap_or_default();
            votes.push(vid);
            self.vote_of_member.insert(caller, &votes);

            let mut votes_of_proposal = self.votes_of_proposal.get(proposal_id).unwrap_or_default();
            votes_of_proposal.push(vid);
            self.votes_of_proposal
                .insert(proposal_id, &votes_of_proposal);

            self.votes.insert(vid, &vote);
            self.votes_helper.list.push(vid);
            self.votes_helper.next_id = vid + 1;

            Ok(())
        }

        /// Cancel vote before proposal is executed or rejected
        #[ink(message)]
        fn cancel_vote(&mut self, vote_id: u128) -> Result<(), Error> {
            let caller = self.env().caller();

            let mut vote = self.votes.get(vote_id).unwrap();

            // check vote user
            if vote.calller != caller {
                return Err(Error::InvalidVoteUser);
            }

            // check proposal status
            let proposal_id = self.votes.get(vote_id).unwrap().call_id;
            if self.status_of_proposal.get(proposal_id).unwrap() != PropStatus::Ongoing {
                return Err(Error::PropNotOngoing);
            }

            vote.deleted = true;
            self.votes.insert(vote_id, &vote);

            // unlock token
            let lock = self
                .member_lock_balances
                .get(caller)
                .unwrap_or(U256::from(0));
            self.member_lock_balances
                .insert(caller, &(lock - vote.pledge));

            Ok(())
        }

        /// Unlock tokens after proposal is executed or rejected
        #[ink(message)]
        fn unlock(&mut self, vote_id: u128) -> Result<(), Error> {
            let caller = self.env().caller();

            // check vote unlock status
            if self.unlock_of_votes.contains(vote_id) {
                return Err(Error::VoteAlreadyUnlocked);
            }

            let vote = self.votes.get(vote_id).unwrap();

            // check vote status
            if vote.deleted {
                return Err(Error::InvalidVoteStatus);
            }

            // check vote user
            if vote.calller != caller {
                return Err(Error::InvalidVoteUser);
            }

            // check vote unlock time
            if vote.unlock_block > self.env().block_number() {
                return Err(Error::InvalidVoteUnlockTime);
            }

            // check proposal status
            let proposal_id = self.votes.get(vote_id).unwrap().call_id;
            let status = self.status_of_proposal.get(proposal_id).unwrap();
            if status != PropStatus::Approved && status != PropStatus::Rejected {
                return Err(Error::PropNotOngoing);
            }

            // unlock token
            let lock = self
                .member_lock_balances
                .get(caller)
                .unwrap_or(U256::from(0));
            self.member_lock_balances
                .insert(caller, &(lock - vote.pledge));
            self.unlock_of_votes.insert(vote_id, &());

            Ok(())
        }

        /// Execute proposal after vote is passed
        #[ink(message)]
        fn exec_proposal(&mut self, proposal_id: CalllId) -> Result<Vec<u8>, Error> {
            let call = self.proposals.get(proposal_id).expect("proposal not found");

            // check status
            if self.status_of_proposal.get(proposal_id).unwrap() != PropStatus::Ongoing {
                return Err(Error::PropNotOngoing);
            }

            let (is_confirm, end) = self.calculate_proposal_status(proposal_id);
            if !is_confirm {
                let now = self.env().block_number();
                if now > end {
                    self.status_of_proposal
                        .insert(proposal_id, &PropStatus::Rejected);
                }
                return Err(Error::ProposalNotConfirmed);
            }

            // Return the deposit amount.
            let deposit = self.deposit_of_proposal.get(proposal_id).unwrap();
            let result = self.env().transfer(deposit.0, deposit.1);
            if result.is_err() {
                return Err(Error::TransferFailed);
            }

            //  Set the status to approved.
            self.status_of_proposal
                .insert(proposal_id, &PropStatus::Approved);

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
            let status = self.status_of_proposal.get(proposal_id).unwrap();
            if status != PropStatus::Ongoing {
                return Ok(status);
            }

            let (is_confirm, end) = self.calculate_proposal_status(proposal_id);
            if !is_confirm {
                let now = self.env().block_number();
                if now > end {
                    return Ok(PropStatus::Rejected);
                }
                return Ok(PropStatus::Ongoing);
            }

            return Ok(PropStatus::Approved);
        }
    }

    impl DAO {
        /// create a new dao
        #[ink(constructor)]
        pub fn new(users: Vec<(Address, U256)>, sudo_account: Option<Address>) -> Self {
            let mut dao = DAO::default();
            let mut members = Vec::new();
            let mut member_balances = Mapping::default();
            let mut total_issuance = U256::from(0);

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

            dao
        }

        /// create a new dao with track
        #[ink(constructor)]
        pub fn new_with_track(
            users: Vec<(Address, U256)>,
            sudo_account: Option<Address>,
            track: Track,
        ) -> Self {
            let mut dao = DAO::default();
            let mut members = Vec::new();
            let mut member_balances = Mapping::default();
            let mut total_issuance = U256::from(0);
            let mut tracks = Mapping::default();
            let mut tracks_helper = ListHelper::<u16>::default();

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

            // init vote track
            tracks_helper.next_id = 1;
            tracks_helper.list.push(0);
            tracks.insert(0, &track);
            dao.tracks = tracks;
            dao.tracks_helper = tracks_helper;
            dao.defalut_track = Some(0);

            dao
        }

        #[ink(constructor)]
        pub fn new_with_default_track(
            users: Vec<(Address, U256)>,
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

            DAO::new_with_track(users, sudo_account, track)
        }

        /// Gov call only call from contract
        fn ensure_from_gov(&self) {
            assert_eq!(self.env().caller(), self.env().address());
        }

        /// Get track rule of call
        fn get_track_id(&self, call: &Call) -> Option<u16> {
            let mut index = self
                .track_rules
                .get((call.contract.clone(), Some(call.selector.clone())));
            if index.is_some() {
                return index;
            }

            index = self
                .track_rules
                .get((call.contract.clone(), None::<Selector>));

            if index.is_some() {
                return index;
            }

            if self.defalut_track.is_some() {
                return self.defalut_track;
            }

            return None;
        }

        /// Get track of call
        fn get_track(&self, proposal_id: CalllId) -> Track {
            let track_id = self.track_of_proposal.get(proposal_id).unwrap();
            let track = self.tracks.get(track_id).unwrap();

            track
        }

        fn calculate_proposal_status(&self, proposal_id: CalllId) -> (bool, BlockNumber) {
            // get votes
            let vote_ids = self.votes_of_proposal.get(proposal_id).unwrap();
            let mut votes = Vec::new();
            for id in vote_ids {
                let vote = self.votes.get(id).unwrap();
                votes.push(vote);
            }

            // get track
            let track = self.get_track(proposal_id);

            // get vote begin and end time
            let (_, _, begin) = self.deposit_of_proposal.get(proposal_id).unwrap();
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

                if yes * 10000 / no >= min_approval && support * 10000 / all >= min_support {
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

            (is_confirm, end)
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
    }
}

#[cfg(test)]
mod tests;
