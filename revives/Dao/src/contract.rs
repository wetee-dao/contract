//! DAO 合约 — PolkaVM/wrevive 迁移版。
//! 第一阶段先迁移构造、成员、ERC20、sudo 与通用 call 基础能力，
//! 后续再补齐 proposal / vote / treasury 的完整治理流转。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<65536> = pvm_bump_allocator::BumpAllocator::new();

mod curve;
mod datas;
mod errors;

use pallet_revive_uapi::CallFlags;
use wrevive_api::{env, Address, Env, Mapping, Storage, U256, Vec};
use wrevive_macro::{mapping, revive_contract, storage};

pub use curve::{arg_to_curve, Curve, CurveArg, Percent};
pub use datas::{Call, CallInput, CalllId, Opinion, PropStatus, Selector, Spend, TokenInfo, Track, VoteInfo};
pub use errors::Error;
pub use primitives::{ensure, ok_or_err};

#[revive_contract]
pub mod dao {
    use super::*;

    const EMPTY_TOPICS: &[[u8; 32]] = &[];

    const MEMBERS: Storage<Vec<Address>> = storage!(b"members");
    const PUBLIC_JOIN: Storage<bool> = storage!(b"public_join");
    const TOTAL_ISSUANCE: Storage<U256> = storage!(b"total_issuance");
    const SUDO_ACCOUNT: Storage<Option<Address>> = storage!(b"sudo_account");
    const TRANSFER_ENABLED: Storage<bool> = storage!(b"transfer_enabled");
    const NEXT_TRACK_ID: Storage<u16> = storage!(b"next_track_id");
    const DEFAULT_TRACK: Storage<Option<u16>> = storage!(b"default_track");

    const MEMBER_BALANCES: Mapping<Address, U256> = mapping!(b"member_balances");
    const MEMBER_LOCK_BALANCES: Mapping<Address, U256> = mapping!(b"member_lock_balances");
    const ALLOWANCES: Mapping<(Address, Address), U256> = mapping!(b"allowances");
    const TOKENS: Mapping<u32, TokenInfo> = mapping!(b"tokens");
    const MEMBER_TOKENS: Mapping<(Address, u32), U256> = mapping!(b"member_tokens");
    const TRACKS: Mapping<u16, Track> = mapping!(b"tracks");
    const TRACK_RULES: Mapping<(Option<Address>, Option<Selector>), u16> = mapping!(b"track_rules");
    const SUDO_CALLS: Mapping<CalllId, Call> = mapping!(b"sudo_calls");
    const NEXT_SUDO_CALL_ID: Storage<CalllId> = storage!(b"next_sudo_call_id");

    #[revive(constructor)]
    pub fn new(
        users: Vec<(Address, U256)>,
        public_join: bool,
        sudo_account: Option<Address>,
    ) -> Result<(), Error> {
        init_state(users, public_join, sudo_account, None)
    }

    #[revive(constructor)]
    pub fn new_with_track(
        users: Vec<(Address, U256)>,
        public_join: bool,
        sudo_account: Option<Address>,
        track: Track,
    ) -> Result<(), Error> {
        init_state(users, public_join, sudo_account, Some(track))
    }

    #[revive(constructor)]
    pub fn new_with_default_track(
        users: Vec<(Address, U256)>,
        public_join: bool,
        sudo_account: Option<Address>,
    ) -> Result<(), Error> {
        let track = Track {
            name: Vec::new(),
            prepare_period: 1,
            max_deciding: 1,
            confirm_period: 1,
            decision_period: 1,
            min_enactment_period: 1,
            decision_deposit: U256::from(1u64),
            max_balance: U256::from(1u64),
            min_approval: Curve::LinearDecreasing { begin: 10000, end: 5000, length: 30 },
            min_support: Curve::LinearDecreasing { begin: 10000, end: 50, length: 30 },
        };
        init_state(users, public_join, sudo_account, Some(track))
    }

    #[revive(message)]
    pub fn list() -> Vec<Address> {
        MEMBERS.get().unwrap_or_default()
    }

    #[revive(message)]
    pub fn get_public_join() -> bool {
        PUBLIC_JOIN.get().unwrap_or(false)
    }

    #[revive(message, write)]
    pub fn public_join() -> Result<(), Error> {
        ensure!(PUBLIC_JOIN.get().unwrap_or(false), Error::PublicJoinNotAllowed);
        let caller = env().caller();
        ensure!(MEMBER_BALANCES.get(&caller).is_none(), Error::MemberExisted);
        MEMBER_BALANCES.set(&caller, &U256::ZERO);
        MEMBER_LOCK_BALANCES.set(&caller, &U256::ZERO);
        let mut members = MEMBERS.get().unwrap_or_default();
        members.push(caller);
        MEMBERS.set(&members);
        env().deposit_event(EMPTY_TOPICS, &caller.0.as_slice());
        Ok(())
    }

    #[revive(message, write)]
    pub fn set_public_join(public_join: bool) -> Result<(), Error> {
        ensure_from_gov()?;
        PUBLIC_JOIN.set(&public_join);
        Ok(())
    }

    #[revive(message, write)]
    pub fn join(new_user: Address, balance: U256) -> Result<(), Error> {
        ensure_from_gov()?;
        ensure!(MEMBER_BALANCES.get(&new_user).is_none(), Error::MemberExisted);
        MEMBER_BALANCES.set(&new_user, &balance);
        MEMBER_LOCK_BALANCES.set(&new_user, &U256::ZERO);
        TOTAL_ISSUANCE.set(&(TOTAL_ISSUANCE.get().unwrap_or(U256::ZERO) + balance));
        let mut members = MEMBERS.get().unwrap_or_default();
        members.push(new_user);
        MEMBERS.set(&members);
        Ok(())
    }

    #[revive(message, write)]
    pub fn levae() -> Result<(), Error> {
        let caller = env().caller();
        ensure!(MEMBER_BALANCES.get(&caller).is_some(), Error::MemberNotExisted);
        ensure!(MEMBER_BALANCES.get(&caller).unwrap_or(U256::ZERO) == U256::ZERO, Error::MemberBalanceNotZero);
        ensure!(MEMBER_LOCK_BALANCES.get(&caller).unwrap_or(U256::ZERO) == U256::ZERO, Error::MemberBalanceNotZero);
        MEMBER_BALANCES.clear(&caller);
        MEMBER_LOCK_BALANCES.clear(&caller);
        let mut members = MEMBERS.get().unwrap_or_default();
        members.retain(|x| *x != caller);
        MEMBERS.set(&members);
        Ok(())
    }

    #[revive(message, write)]
    pub fn levae_with_burn() -> Result<(), Error> {
        let caller = env().caller();
        ensure!(MEMBER_BALANCES.get(&caller).is_some(), Error::MemberNotExisted);
        let amount = MEMBER_BALANCES.get(&caller).unwrap_or(U256::ZERO)
            + MEMBER_LOCK_BALANCES.get(&caller).unwrap_or(U256::ZERO);
        let total = TOTAL_ISSUANCE.get().unwrap_or(U256::ZERO);
        ensure!(total >= amount, Error::LowBalance);
        TOTAL_ISSUANCE.set(&(total - amount));
        MEMBER_BALANCES.clear(&caller);
        MEMBER_LOCK_BALANCES.clear(&caller);
        let mut members = MEMBERS.get().unwrap_or_default();
        members.retain(|x| *x != caller);
        MEMBERS.set(&members);
        Ok(())
    }

    #[revive(message, write)]
    pub fn delete(user: Address) -> Result<(), Error> {
        ensure_from_gov()?;
        ensure!(MEMBER_BALANCES.get(&user).is_some(), Error::MemberNotExisted);
        let amount = MEMBER_BALANCES.get(&user).unwrap_or(U256::ZERO)
            + MEMBER_LOCK_BALANCES.get(&user).unwrap_or(U256::ZERO);
        let total = TOTAL_ISSUANCE.get().unwrap_or(U256::ZERO);
        ensure!(total >= amount, Error::LowBalance);
        TOTAL_ISSUANCE.set(&(total - amount));
        MEMBER_BALANCES.clear(&user);
        MEMBER_LOCK_BALANCES.clear(&user);
        let mut members = MEMBERS.get().unwrap_or_default();
        members.retain(|x| *x != user);
        MEMBERS.set(&members);
        Ok(())
    }

    #[revive(message)]
    pub fn total_supply() -> U256 {
        TOTAL_ISSUANCE.get().unwrap_or(U256::ZERO)
    }

    #[revive(message)]
    pub fn balance_of(owner: Address) -> U256 {
        MEMBER_BALANCES.get(&owner).unwrap_or(U256::ZERO)
    }

    #[revive(message)]
    pub fn lock_balance_of(owner: Address) -> U256 {
        MEMBER_LOCK_BALANCES.get(&owner).unwrap_or(U256::ZERO)
    }

    #[revive(message)]
    pub fn allowance(owner: Address, spender: Address) -> U256 {
        ALLOWANCES.get(&(owner, spender)).unwrap_or(U256::ZERO)
    }

    #[revive(message, write)]
    pub fn approve(spender: Address, value: U256) -> Result<(), Error> {
        let caller = env().caller();
        ensure!(MEMBER_BALANCES.get(&caller).is_some(), Error::MemberNotExisted);
        ALLOWANCES.set(&(caller, spender), &value);
        Ok(())
    }

    #[revive(message, write)]
    pub fn transfer(to: Address, value: U256) -> Result<(), Error> {
        ensure!(TRANSFER_ENABLED.get().unwrap_or(true), Error::TransferDisable);
        transfer_from_to(env().caller(), to, value)
    }

    #[revive(message, write)]
    pub fn transfer_from(from: Address, to: Address, value: U256) -> Result<(), Error> {
        ensure!(TRANSFER_ENABLED.get().unwrap_or(true), Error::TransferDisable);
        let caller = env().caller();
        let allowance = ALLOWANCES.get(&(from, caller)).unwrap_or(U256::ZERO);
        ensure!(allowance >= value, Error::InsufficientAllowance);
        ALLOWANCES.set(&(from, caller), &(allowance - value));
        transfer_from_to(from, to, value)
    }

    #[revive(message, write)]
    pub fn burn(value: U256) -> Result<(), Error> {
        let caller = env().caller();
        let free = free_balance(caller);
        ensure!(free >= value, Error::LowBalance);
        let balance = MEMBER_BALANCES.get(&caller).unwrap_or(U256::ZERO);
        MEMBER_BALANCES.set(&caller, &(balance - value));
        let total = TOTAL_ISSUANCE.get().unwrap_or(U256::ZERO);
        ensure!(total >= value, Error::LowBalance);
        TOTAL_ISSUANCE.set(&(total - value));
        Ok(())
    }

    #[revive(message)]
    pub fn sudo_account() -> Option<Address> {
        SUDO_ACCOUNT.get().unwrap_or(None)
    }

    #[revive(message, write)]
    pub fn sudo(call: Call) -> Result<Vec<u8>, Error> {
        let caller = env().caller();
        ensure!(SUDO_ACCOUNT.get().unwrap_or(None) == Some(caller), Error::MustCallByGov);
        let call_id = NEXT_SUDO_CALL_ID.get().unwrap_or(0);
        NEXT_SUDO_CALL_ID.set(&(call_id + 1));
        SUDO_CALLS.set(&call_id, &call);
        exec_call_internal(call)
    }

    #[revive(message, write)]
    pub fn remove_sudo() -> Result<(), Error> {
        let caller = env().caller();
        ensure!(SUDO_ACCOUNT.get().unwrap_or(None) == Some(caller), Error::MustCallByGov);
        SUDO_ACCOUNT.set(&None);
        Ok(())
    }

    #[revive(message)]
    pub fn defalut_track() -> Option<u16> {
        DEFAULT_TRACK.get().unwrap_or(None)
    }

    #[revive(message)]
    pub fn track(id: u16) -> Option<Track> {
        TRACKS.get(&id)
    }

    #[revive(message)]
    pub fn track_list() -> Vec<(u16, Track)> {
        let total = NEXT_TRACK_ID.get().unwrap_or(0);
        let mut out = Vec::new();
        let mut i = 0u16;
        while i < total {
            if let Some(track) = TRACKS.get(&i) {
                out.push((i, track));
            }
            i += 1;
        }
        out
    }

    #[revive(message, write)]
    pub fn add_track(track: Track) -> Result<u16, Error> {
        ensure_from_gov()?;
        let track_id = NEXT_TRACK_ID.get().unwrap_or(0);
        TRACKS.set(&track_id, &track);
        NEXT_TRACK_ID.set(&(track_id + 1));
        Ok(track_id)
    }

    #[revive(message, write)]
    pub fn set_defalut_track(track_id: u16) -> Result<(), Error> {
        ensure_from_gov()?;
        ensure!(TRACKS.get(&track_id).is_some(), Error::NoTrack);
        DEFAULT_TRACK.set(&Some(track_id));
        Ok(())
    }

    #[revive(message, write)]
    pub fn edit_track(track_id: u16, track: Track) -> Result<(), Error> {
        ensure_from_gov()?;
        ensure!(TRACKS.get(&track_id).is_some(), Error::NoTrack);
        TRACKS.set(&track_id, &track);
        Ok(())
    }

    #[revive(message, write)]
    pub fn set_track_rule(contract: Option<Address>, selector: Option<Selector>, track_id: u16) -> Result<(), Error> {
        ensure_from_gov()?;
        ensure!(TRACKS.get(&track_id).is_some(), Error::NoTrack);
        TRACK_RULES.set(&(contract, selector), &track_id);
        Ok(())
    }

    #[revive(message)]
    pub fn token(id: u32) -> Option<TokenInfo> {
        TOKENS.get(&id)
    }

    #[revive(message)]
    pub fn member_token(owner: Address, token_id: u32) -> U256 {
        MEMBER_TOKENS.get(&(owner, token_id)).unwrap_or(U256::ZERO)
    }

    #[revive(message)]
    pub fn set_code(_code_hash: wrevive_api::H256) -> Result<(), Error> {
        ensure_from_gov()?;
        Err(Error::SetCodeFailed)
    }

    fn init_state(
        users: Vec<(Address, U256)>,
        public_join: bool,
        sudo_account: Option<Address>,
        track: Option<Track>,
    ) -> Result<(), Error> {
        let mut members = Vec::new();
        let mut total = U256::ZERO;
        for (user, balance) in users.iter() {
            ensure!(MEMBER_BALANCES.get(user).is_none(), Error::MemberExisted);
            MEMBER_BALANCES.set(user, balance);
            MEMBER_LOCK_BALANCES.set(user, &U256::ZERO);
            members.push(*user);
            total = total + *balance;
        }
        MEMBERS.set(&members);
        PUBLIC_JOIN.set(&public_join);
        SUDO_ACCOUNT.set(&sudo_account);
        TOTAL_ISSUANCE.set(&total);
        TRANSFER_ENABLED.set(&true);
        NEXT_SUDO_CALL_ID.set(&0);

        let token = TokenInfo {
            name: b"WeTEE DAO".to_vec(),
            symbol: b"DAO".to_vec(),
            decimals: 18,
        };
        TOKENS.set(&0, &token);

        if let Some(track) = track {
            TRACKS.set(&0, &track);
            NEXT_TRACK_ID.set(&1);
            DEFAULT_TRACK.set(&Some(0));
        } else {
            NEXT_TRACK_ID.set(&0);
            DEFAULT_TRACK.set(&None);
        }
        Ok(())
    }

    fn ensure_from_gov() -> Result<(), Error> {
        ensure!(env().caller() == env().address(), Error::MustCallByGov);
        Ok(())
    }

    fn free_balance(owner: Address) -> U256 {
        MEMBER_BALANCES.get(&owner).unwrap_or(U256::ZERO)
            - MEMBER_LOCK_BALANCES.get(&owner).unwrap_or(U256::ZERO)
    }

    fn transfer_from_to(from: Address, to: Address, value: U256) -> Result<(), Error> {
        ensure!(MEMBER_BALANCES.get(&from).is_some(), Error::MemberNotExisted);
        let free = free_balance(from);
        ensure!(free >= value, Error::LowBalance);
        let from_balance = MEMBER_BALANCES.get(&from).unwrap_or(U256::ZERO);
        MEMBER_BALANCES.set(&from, &(from_balance - value));
        let to_balance = MEMBER_BALANCES.get(&to).unwrap_or(U256::ZERO);
        MEMBER_BALANCES.set(&to, &(to_balance + value));
        let mut members = MEMBERS.get().unwrap_or_default();
        if !members.iter().any(|x| *x == to) {
            members.push(to);
            MEMBERS.set(&members);
        }
        Ok(())
    }

    fn exec_call_internal(call: Call) -> Result<Vec<u8>, Error> {
        let call_flags = if call.allow_reentry {
            CallFlags::ALLOW_REENTRY
        } else {
            CallFlags::empty()
        };
        let callee = call.contract.unwrap_or(env().address());
        env()
            .call(
                call_flags,
                &callee,
                call.ref_time_limit,
                u64::MAX,
                &U256::ZERO,
                &call.amount,
                &encode_raw_call(&call.selector, &call.input),
                None,
            )
            .map_err(|_| Error::CallFailed)?;
        let size = env().return_data_size() as usize;
        let mut buf = alloc::vec![0u8; size];
        let mut slice = buf.as_mut_slice();
        env().return_data_copy(&mut slice, 0);
        Ok(buf)
    }

    fn encode_raw_call(selector: &Selector, input: &[u8]) -> Vec<u8> {
        let mut data = Vec::with_capacity(4 + input.len());
        data.extend_from_slice(selector);
        data.extend_from_slice(input);
        data
    }
}

#[cfg(test)]
mod tests;
