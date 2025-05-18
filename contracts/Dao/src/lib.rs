#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;
mod events;

#[ink::contract]
mod dao {
    use crate::{datas::*, errors::Error, events::*};
    use ink::{
        env::{
            call::{build_call, ExecutionInput},
            CallFlags,
        },
        prelude::vec::Vec,
        storage::Mapping,
        U256,
    };
    use primitives::{Call, CallInput, CalllId, ListHelper, Selector};

    #[ink(storage)]
    #[derive(Default)]
    pub struct DAO {
        /// proposals
        proposals: Mapping<CalllId, Call>,
        /// period of proposal
        period_of_proposal: Mapping<CalllId, u16>,
        /// proposals list helper
        proposals_helper: ListHelper<CalllId>,
        /// caller of proposal
        proposal_caller: Mapping<CalllId, Address>,
        /// deposit of proposal
        deposit_of_proposal: Mapping<CalllId, (Address, U256)>,
        /// status of proposal
        status_of_proposal: Mapping<CalllId, PropStatus>,

        /// periods
        periods: Mapping<u16, Period>,
        /// periods list helper
        periods_helper: ListHelper<u16>,

        /// period rules (If selector == none, it means entire contract uses a single track)
        period_rules: Mapping<(Option<Address>, Option<Selector>), u16>,
        /// period rules index
        period_rule_index: Mapping<u16, (Option<Address>, Option<Selector>, u16)>,
        /// period rules index helper
        period_rule_index_helper: ListHelper<u16>,

        /// vote of proposal
        votes: Mapping<u128, VoteInfo>,
        /// proposals list helper
        votes_helper: ListHelper<u128>,
        /// votes of member
        vote_of_member: Mapping<Address, Vec<u128>>,

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

    impl DAO {
        /// create a new dao
        #[ink(constructor)]
        pub fn new(
            args: Vec<(Address, U256)>,
            sudo_account: Option<Address>,
            period: Period,
        ) -> Self {
            let mut dao = DAO::default();
            let mut members = Vec::new();
            let mut member_balances = Mapping::default();
            let mut total_issuance = U256::from(0);
            let mut periods = Mapping::default();
            let mut periods_helper = ListHelper::<u16>::default();

            // Init members balances
            for (user, balance) in args.iter() {
                member_balances.insert(*user, balance);
                members.push(*user);
                total_issuance = total_issuance
                    .checked_add(*balance)
                    .expect("issuance overflow");
            }

            dao.members = members;
            dao.member_balances = member_balances;
            dao.total_issuance = total_issuance;
            dao.sudo_account = sudo_account;

            periods_helper.next_id = 1;
            periods_helper.list.push(0);
            periods.insert(0, &period);
            dao.periods = periods;
            dao.periods_helper = periods_helper;

            dao
        }

        /// Returns list of members.
        #[ink(message)]
        pub fn members(&self) -> Vec<Address> {
            self.members.clone()
        }

        /// Join to DAO
        #[ink(message)]
        pub fn join(&mut self, new_user: Address, balance: U256) {
            self.ensure_from_gov();

            // check if user is already an member
            assert!(!self.member_balances.contains(new_user));

            self.member_balances.insert(new_user, &balance);
            self.members.push(new_user);

            self.env().emit_event(MemberAdd { user: new_user });
        }

        // levae DAO
        #[ink(message)]
        pub fn levae(&mut self) {
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
        pub fn levae_with_burn(&mut self) {
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
        pub fn delete_member(&mut self, user: Address) {
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

        /// Enable transfer
        #[ink(message)]
        pub fn enable_transfer(&mut self) {
            self.ensure_from_gov();

            if self.transfer {
                return;
            }

            self.transfer = true;
        }

        /// Transfer TOKEN to user
        #[ink(message)]
        pub fn transfer(&mut self, to: Address, amount: U256) {
            assert!(self.transfer);

            let caller = self.env().caller();

            // check if user is an member
            assert!(self.member_balances.contains(caller));
            assert!(self.member_balances.contains(to));

            let total = self.member_balances.get(caller).unwrap_or(U256::from(0));
            assert!(total >= amount);

            self.member_balances.insert(caller, &(total - amount));
            self.member_balances.insert(
                to,
                &(self.member_balances.get(to).unwrap_or(U256::from(0)) + amount),
            );
        }

        /// Burn tokens from caller's balance.
        #[ink(message)]
        pub fn burn(&mut self, amount: U256) {
            let caller = self.env().caller();

            // check if user is an member
            assert!(self.member_balances.contains(caller));

            let total = self.member_balances.get(caller).unwrap();
            assert!(total >= amount);

            self.member_balances.insert(caller, &(total - amount));
            self.total_issuance -= amount;
        }

        /// If sudo is enabled, sudo account can call any function without gov
        #[ink(message)]
        pub fn sudo(&mut self, call: Call) -> Result<Vec<u8>, Error> {
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
        pub fn remove_sudo(&mut self) {
            self.ensure_from_gov();

            self.sudo_account = None;
        }

        /// Submit a proposal to DAO
        #[ink(message)]
        pub fn submit_proposal(&mut self, call: Call) -> CalllId {
            let caller = self.env().caller();

            // check if user is an member
            assert!(self.member_balances.contains(caller));

            //  get period of call
            let period = self.get_period(&call);

            // save proposal
            let call_id = self.proposals_helper.next_id;
            self.proposals_helper.next_id = call_id.checked_add(1).expect("proposal id overflow");
            self.proposals_helper.list.push(call_id);
            self.proposals.insert(call_id, &call);

            // set caller of proposal
            self.proposal_caller.insert(call_id, &caller);

            // set period for proposal
            self.period_of_proposal.insert(call_id, &period);

            // set proposal status
            self.status_of_proposal
                .insert(call_id, &PropStatus::Pending);

            // emit event
            self.env().emit_event(ProposalSubmission {
                proposal_id: call_id,
            });

            call_id
        }

        /// Cancel a proposal
        #[ink(message)]
        pub fn cancel_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error> {
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
        pub fn deposit_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            let caller = self.env().caller();
            let payvalue = self.env().transferred_value();

            // check status
            let status = self.status_of_proposal.get(proposal_id).unwrap();
            if status != PropStatus::Pending {
                return Err(Error::InvalidProposalStatus);
            }

            // check period
            let period_id = self.period_of_proposal.get(proposal_id).unwrap();
            let period = self.periods.get(period_id).unwrap();

            // check payvalue
            if payvalue < period.decision_deposit {
                return Err(Error::InvalidDeposit);
            }

            // save deposit
            self.deposit_of_proposal.insert(proposal_id, &(caller,payvalue));

            // save status
            self.status_of_proposal
                .insert(proposal_id, &PropStatus::Ongoing);

            Ok(())
        }

        /// Vote for a proposal
        #[ink(message)]
        pub fn vote_for_prop(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            Ok(())
        }

        /// Cancel vote before proposal is executed or rejected
        #[ink(message)]
        pub fn cancel_vote(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            
            Ok(())
        }

        /// Unlock tokens after proposal is executed or rejected
        #[ink(message)]
        pub fn unlock(&mut self, vote_id: CalllId) -> Result<(), Error> {
        
            Ok(())
        }

        /// Execute proposal after vote is passed
        #[ink(message)]
        pub fn exec_proposal(&mut self, proposal_id: CalllId) -> Result<Vec<u8>, Error> {
            let call = self.take_proposal(proposal_id).expect("proposal not found");

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

        /// Returns index of member of members.
        fn get_member_index(&self, owner: &Address) -> u32 {
            self.members
                .iter()
                .position(|x| *x == *owner)
                .expect("Member not found in members list") as u32
        }

        /// Gov call only call from contract
        fn ensure_from_gov(&self) {
            assert_eq!(self.env().caller(), self.env().address());
        }

        /// Get period of call
        fn get_period(&self, call: &Call) -> u16 {
            let index = self
                .period_rules
                .get((call.contract.clone(), Some(call.selector.clone())))
                .unwrap_or(0u16);
            if index > 0 {
                return index;
            }

            return self
                .period_rules
                .get((call.contract.clone(), None::<Selector>))
                .unwrap_or(0u16);
        }

        /// Take proposal and remove it from list
        fn take_proposal(&mut self, pid: CalllId) -> Option<Call> {
            let proposal = self.proposals.get(pid);
            if proposal.is_some() {
                self.proposals.remove(pid);
                let pos = self
                    .proposals_helper
                    .list
                    .iter()
                    .position(|t| t == &pid)
                    .expect("Proposal not found in list");
                self.proposals_helper.list.swap_remove(pos);
            }
            proposal
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

    #[cfg(test)]
    mod tests {}
}
