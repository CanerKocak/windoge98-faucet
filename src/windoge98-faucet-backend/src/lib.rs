#[macro_use]
extern crate ic_cdk_macros;
extern crate serde;

use std::cell::RefCell;
use std::collections::VecDeque;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api;

// State struct definition
#[derive(CandidType, Deserialize)]
struct State {
    admin: Principal,
    is_faucet_enabled: bool,
    faucet_code: String,
    faucet_amount: u64,
    claimed_principals: Vec<Principal>,
    recent_claims: VecDeque<(Principal, u64)>,
    total_claims: Vec<(Principal, u64)>,
}

// Globals: thread_local!
thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

#[init]
fn init(admin: Principal) {
    // This function is called when the canister is created.
    // It initializes the state with the provided admin principal.
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.admin = admin;
    });
}

// Smart contract functions

#[update]
fn toggle_faucet(is_enabled: bool) {
    // This function allows the admin to enable or disable the faucet.
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert_eq!(
            api::caller(),
            state.admin,
            "Only admin can toggle the faucet"
        );
        state.is_faucet_enabled = is_enabled;
    });
}

#[update]
fn set_faucet_code(code: String) {
    // This function allows the admin to set the faucet code.
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert_eq!(
            api::caller(),
            state.admin,
            "Only admin can set the faucet code"
        );
        state.faucet_code = code;
    });
}

#[update]
fn set_faucet_amount(amount: u64) {
    // This function allows the admin to set the amount of EXE tokens to be claimed.
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert_eq!(
            api::caller(),
            state.admin,
            "Only admin can set the faucet amount"
        );
        state.faucet_amount = amount;
    });
}

#[update]
fn reset_claimed_principals() {
    // This function allows the admin to reset the list of claimed principals.
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert_eq!(
            api::caller(),
            state.admin,
            "Only admin can reset claimed principals"
        );
        state.claimed_principals.clear();
    });
}

#[update]
fn claim_faucet(code: String) {
    // This function allows users to claim EXE tokens by providing the correct faucet code.
    STATE.with(|state| {
        let mut state = state.borrow_mut();

        assert!(state.is_faucet_enabled, "Faucet is currently disabled");
        assert_eq!(code, state.faucet_code, "Invalid faucet code");

        let caller = api::caller();
        assert!(
            !state.claimed_principals.contains(&caller),
            "Principal has already claimed from the faucet"
        );

        // Transfer EXE tokens to the caller (assuming you have a transfer function)
        // transfer(caller, state.faucet_amount);

        state.claimed_principals.push(caller);
        state
            .recent_claims
            .push_front((caller, state.faucet_amount));
        state.total_claims.push((caller, state.faucet_amount));

        if state.recent_claims.len() > 10 {
            state.recent_claims.pop_back();
        }
    });
}

#[query]
fn get_recent_claims() -> Vec<(Principal, u64)> {
    // This function returns the list of recent claims.
    STATE.with(|state| {
        let state = state.borrow();
        state.recent_claims.iter().cloned().collect()
    })
}

#[query]
fn get_total_claims() -> Vec<(Principal, u64)> {
    // This function returns the total history of claims.
    STATE.with(|state| {
        let state = state.borrow();
        state.total_claims.clone()
    })
}
