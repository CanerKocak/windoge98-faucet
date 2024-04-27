extern crate ic_cdk_macros;
extern crate serde;

use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api;
use ic_cdk::*;

// State struct definition (Canister Storage)
#[derive(CandidType, Deserialize, Default)]
struct State {
    custodians: HashSet<Principal>,
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

// Canister initialization
#[init]
fn init() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.custodians.insert(api::caller());
    });
}

// Pre-upgrade hook
#[pre_upgrade]
fn pre_upgrade() {
    STATE.with(|state| {
        let state = state.borrow();
        let owned_state = State {
            custodians: state.custodians.clone(),
            is_faucet_enabled: state.is_faucet_enabled,
            faucet_code: state.faucet_code.clone(),
            faucet_amount: state.faucet_amount,
            claimed_principals: state.claimed_principals.clone(),
            recent_claims: state.recent_claims.clone(),
            total_claims: state.total_claims.clone(),
        };
        ic_cdk::storage::stable_save((owned_state,)).unwrap();
    });
}

// Post-upgrade hook
#[post_upgrade]
fn post_upgrade() {
    let (state,): (State,) = ic_cdk::storage::stable_restore().unwrap();
    STATE.with(|state0| {
        *state0.borrow_mut() = state;
    });
}

// ----------------------------------------------
// Smart contract functions
// ----------------------------------------------

// Add a new custodian
#[update]
fn add_custodian(custodian: Principal) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(
            state.custodians.contains(&api::caller()),
            "Only custodians can add new custodians"
        );
        state.custodians.insert(custodian);
    });
}

// Remove a custodian
#[update]
fn remove_custodian(custodian: Principal) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(
            state.custodians.contains(&api::caller()),
            "Only custodians can remove custodians"
        );
        state.custodians.remove(&custodian);
    });
}

// Toggle faucet on/off
#[update]
fn toggle_faucet(is_enabled: bool) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(
            state.custodians.contains(&api::caller()),
            "Only custodians can toggle the faucet"
        );
        state.is_faucet_enabled = is_enabled;
    });
}

// Set faucet code
#[update]
fn set_faucet_code(code: String) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(
            state.custodians.contains(&api::caller()),
            "Only custodians can set the faucet code"
        );
        state.faucet_code = code;
    });
}

// Set faucet amount
#[update]
fn set_faucet_amount(amount: u64) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(
            state.custodians.contains(&api::caller()),
            "Only custodians can set the faucet amount"
        );
        state.faucet_amount = amount;
    });
}

// Reset claimed principals
#[update]
fn reset_claimed_principals() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(
            state.custodians.contains(&api::caller()),
            "Only custodians can reset claimed principals"
        );
        state.claimed_principals.clear();
    });
}

// Claim faucet
#[update]
fn claim_faucet(code: String) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(state.is_faucet_enabled, "Faucet is currently disabled");
        assert_eq!(code, state.faucet_code, "Invalid faucet code");

        let caller = api::caller();
        assert!(
            !state.claimed_principals.contains(&caller),
            "Principal has already claimed from the faucet"
        );

        let faucet_amount = state.faucet_amount;

        // TODO: Implement token transfer logic
        // transfer(caller, faucet_amount);

        state.claimed_principals.push(caller);
        state.recent_claims.push_back((caller, faucet_amount));
        state.total_claims.push((caller, faucet_amount));
    });
}

// Get recent claims
#[query]
fn get_recent_claims() -> Vec<(Principal, u64)> {
    STATE.with(|state| {
        let state = state.borrow();
        state.recent_claims.iter().cloned().collect()
    })
}

// Get total claims
#[query]
fn get_total_claims() -> Vec<(Principal, u64)> {
    STATE.with(|state| {
        let state = state.borrow();
        state.total_claims.clone()
    })
}
