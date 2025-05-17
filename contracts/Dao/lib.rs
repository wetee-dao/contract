#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod dao {
    use ink::{
        env::{
            call::{build_call, ExecutionInput},
            CallFlags,
        },
        prelude::vec::Vec,
        scale::Output,
        storage::Mapping,
        U256,
    };

    type CalllId = u32;

    #[derive(Clone)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Call {
        /// The address of the contract that is call in this proposal.
        pub contract: Address,
        /// The selector bytes that identifies the function of the contract that should be call.
        pub selector: [u8; 4],
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

    #[derive(Clone, Default)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct ListHelper<T> {
        list: Vec<T>,
        next_id: T,
    }

    #[derive(Clone)]
    struct CallInput<'a>(&'a [u8]);
    impl ink::scale::Encode for CallInput<'_> {
        fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
            dest.write(self.0);
        }
    }

    /// The member added event.
    #[ink(event)]
    pub struct MemberAdd {
        #[ink(topic)]
        user: Address,
    }

    #[ink(event)]
    pub struct ProposalSubmission {
        #[ink(topic)]
        proposal_id: CalllId,
    }

    #[ink(event)]
    pub struct ProposalExecution {
        #[ink(topic)]
        proposal_id: CalllId,
        #[ink(topic)]
        result: Result<Option<Vec<u8>>, Error>,
    }

    #[ink(event)]
    pub struct SudoExecution {
        #[ink(topic)]
        sudo_id: CalllId,
        #[ink(topic)]
        result: Result<Option<Vec<u8>>, Error>,
    }

    #[ink(storage)]
    pub struct DAO {
        /// proposals
        proposals: Mapping<CalllId, Call>,
        proposals_helper: ListHelper<CalllId>,
        proposal_owners: Mapping<CalllId, Address>,

        /// sudo call history
        sudo_calls: Mapping<CalllId, Call>,
        /// next sudo call id
        next_sudo_id: CalllId,
        /// sudo account
        sudo_account: Option<Address>,

        /// members
        members: Vec<Address>,
        /// member balances
        member_balances: Mapping<Address, U256>,
        /// total issuance TOKEN
        total_issuance: U256,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Returned if the call failed.
        CallFailed,
    }

    impl DAO {
        #[ink(constructor)]
        pub fn new(args: Vec<(Address, U256)>, sudo_account: Option<Address>) -> Self {
            let proposals_helper: ListHelper<CalllId> = Default::default();
            let proposals = Mapping::default();
            let proposal_owners = Mapping::default();

            let sudo_calls = Mapping::default();

            let mut members = Vec::new();
            let mut member_balances = Mapping::default();
            let mut total_issuance = U256::from(0);

            // Init members balances
            for (user, balance) in args.iter() {
                member_balances.insert(*user, balance);
                members.push(*user);
                total_issuance = total_issuance
                    .checked_add(*balance)
                    .expect("issuance overflow");
            }

            Self {
                proposals,
                proposals_helper,
                proposal_owners,
                members,
                member_balances,
                total_issuance,
                sudo_calls,
                next_sudo_id: 0,
                sudo_account,
            }
        }

        /// Returns the list of members.
        #[ink(message)]
        pub fn members(&self) -> Vec<Address> {
            self.members.clone()
        }

        /// add member to DAO
        #[ink(message)]
        pub fn add_member(&mut self, new_user: Address, balance: U256) {
            self.ensure_from_gov();

            // check if the user is already an member
            assert!(!self.member_balances.contains(new_user));

            self.member_balances.insert(new_user, &balance);
            self.members.push(new_user);

            self.env().emit_event(MemberAdd { user: new_user });
        }

        /// delete a member from the list
        #[ink(message)]
        pub fn delete_member(&mut self, user: Address) {
            self.ensure_from_gov();

            // check if the user is an member
            assert!(self.member_balances.contains(user));

            let index = self.get_member_index(&user) as usize;
            self.members.swap_remove(index);
            self.member_balances.remove(user);
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
            let call_id = self.next_sudo_id;
            self.next_sudo_id = call_id.checked_add(1).expect("call id overflow");
            self.sudo_calls.insert(call_id, &call);

            let result = self.exec_call(call);
            self.env().emit_event(SudoExecution {
                sudo_id: call_id,
                result: result.clone().map(Some),
            });

            result
        }

        /// After ensuring the stable operation of DAO, delete sudo.
        #[ink(message)]
        pub fn remove_sudo(&mut self) {
            self.ensure_from_gov();

            self.sudo_account = None;
        }

        /// Submit a proposal to the DAO
        #[ink(message)]
        pub fn submit_proposal(&mut self, call: Call) -> CalllId {
            let caller = self.env().caller();

            // check if the user is an member
            assert!(self.member_balances.contains(caller));

            let call_id = self.proposals_helper.next_id;
            self.proposals_helper.next_id = call_id.checked_add(1).expect("proposal id overflow");

            self.proposals.insert(call_id, &call);
            self.proposals_helper.list.push(call_id);
            self.proposal_owners.insert(call_id, &caller);

            self.env().emit_event(ProposalSubmission {
                proposal_id: call_id,
            });

            call_id
        }

        /// Cancel a proposal
        #[ink(message)]
        pub fn cancel_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            Ok(())
        }

        /// Confirm a proposal with deposit TOKEN.
        #[ink(message)]
        pub fn deposit_proposal(&mut self, proposal_id: CalllId) -> Result<(), Error> {
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
        pub fn unlock(&mut self, proposal_id: CalllId) -> Result<(), Error> {
            Ok(())
        }

        /// Execute proposal after vote is passed
        #[ink(message, payable)]
        pub fn exec_proposal(&mut self, proposal_id: CalllId) -> Result<Vec<u8>, Error> {
            let call = self.take_proposal(proposal_id).expect("proposal not found");

            let result = self.exec_call(call);
            self.env().emit_event(ProposalExecution {
                proposal_id,
                result: result.clone().map(Some),
            });

            result
        }

        /// Returns the index of the member in the list of members.
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
                .call(call.contract)
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
