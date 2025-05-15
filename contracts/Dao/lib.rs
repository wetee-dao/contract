#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod dao {
    use ink::{
        U256,scale::Output,
        env::{
            call::{
                build_call,
                ExecutionInput,
            },
            CallFlags,
        },
        prelude::vec::Vec,
        storage::Mapping,
    };

    type ProposalId = u32;

    #[derive(Clone)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Proposal {
        /// The address of the contract that is Proposaled in this transaction.
        pub contract: Address,
        /// The selector bytes that identifies the function of the Proposalee that should be
        /// Proposaled.
        pub selector: [u8; 4],
        /// The SCALE encoded parameters that are passed to the Proposaled function.
        pub input: Vec<u8>,
        /// The amount of chain balance that is transferred to the Proposalee.
        pub amount: U256,
        /// Gas limit for the execution of the Proposal.
        pub ref_time_limit: u64,
        /// If set to true the transaction will be allowed to re-enter the multisig
        /// contract. Re-entrancy can lead to vulnerabilities. Use at your own
        /// risk.
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

    #[ink(storage)]
    pub struct DAO {
        /// proposals of the DAO
        proposals: Mapping<ProposalId, Proposal>,
        proposal_helper: ListHelper<ProposalId>,

        /// members of the DAO
        members: Vec<Address>,
        /// member balances
        member_balances: Mapping<Address, U256>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Returned if the call failed.
        CallFailed,
    }

    impl DAO {
        #[ink(constructor)]
        pub fn new(members: Vec<Address>) -> Self {
            let proposal_helper:ListHelper<ProposalId> = Default::default();
            let proposals = Mapping::default();
            let member_balances = Mapping::default();

            Self { proposals,proposal_helper,members,member_balances }
        }

        #[ink(message)]
        pub fn members(&self) -> Vec<Address> {
            self.members.clone()
        }

        #[ink(message)]
        pub fn add_member(&mut self, new_user: Address) {
            self.ensure_from_gov();

            // check if the user is already an member
            assert!(!self.member_balances.contains(new_user));

            self.member_balances.insert(new_user, &U256::from(0));
            self.members.push(new_user);
            // self.env().emit_event(OwnerAddition { owner: new_user });
        }

        #[ink(message)]
        pub fn delete_member(&mut self, user: Address) {
            self.ensure_from_gov();

            // check if the user is an member
            assert!(self.member_balances.contains(user));

            let index = self.get_member_index(&user) as usize;
            self.members.swap_remove(index);
            self.member_balances.remove(user);
        }

        // #[ink(message)]
        pub fn submit_proposal(
            &mut self,
            transaction: Proposal,
        ) -> ProposalId {
            let caller = self.env().caller();
            // check if the user is an member
            assert!(self.member_balances.contains(caller));

            let call_id = self.proposal_helper.next_id;
            self.proposal_helper.next_id =
                call_id.checked_add(1).expect("Call ids exhausted.");

            self.proposals.insert(call_id, &transaction);
            self.proposal_helper.list.push(call_id);
            // self.env().emit_event(Submission {
            //     transaction: trans_id,
            // });
            // (
            call_id
            //     self.confirm_by_caller(self.env().caller(), call_id),
            // )
        }

        #[ink(message, payable)]
        pub fn run_proposal(
            &mut self,
            proposal_id: ProposalId,
        ) -> Result<Vec<u8>, Error> {
            let p = self.take_proposal(proposal_id).expect("proposal not found");
            let call_flags = if p.allow_reentry {
                CallFlags::ALLOW_REENTRY
            } else {
                CallFlags::empty()
            };

            let result = build_call::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(p.contract)
                .ref_time_limit(p.ref_time_limit)
                .transferred_value(p.amount)
                .call_flags(call_flags)
                .exec_input(
                    ExecutionInput::new(p.selector.into()).push_arg(CallInput(&p.input)),
                )
                .returns::<Vec<u8>>()
                .try_invoke();

            let result = match result {
                Ok(Ok(v)) => Ok(v),
                _ => Err(Error::CallFailed),
            };

            // self.env().emit_event(Execution {
            //     transaction: trans_id,
            //     result: result.clone().map(Some),
            // });
            result
        }

        // cancel_proposal
        // deposit_proposal
        // vote_for_prop
        // cancel_vote
        // unlock

        fn get_member_index(&self, owner: &Address) -> u32 {
            self.members.iter().position(|x| *x == *owner).expect(
                "Member not found in members list",
            ) as u32
        }

        fn ensure_from_gov(&self) {
            assert_eq!(self.env().caller(), self.env().address());
        }

        fn take_proposal(&mut self, pid: ProposalId) -> Option<Proposal> {
            let proposal = self.proposals.get(pid);
            if proposal.is_some() {
                self.proposals.remove(pid);
                let pos = self
                    .proposal_helper
                    .list
                    .iter()
                    .position(|t| t == &pid)
                    .expect("Proposal not found in list");
                self.proposal_helper.list.swap_remove(pos);
            }
            proposal
        }
    }
}
