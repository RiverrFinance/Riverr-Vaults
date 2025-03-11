use std::cell::RefCell;

use candid::{CandidType, Deserialize, Principal};

use icrc_ledger_types::icrc1::account::{Account, Subaccount};

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};

use core_lib::staking::{StakeDetails, StakeSpan, VaultStakingDetails};
use types::VaultDetails;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type Amount = u128;
type Time = u64;

const _VAULT_DETAILS_MEMORY_ID: MemoryId = MemoryId::new(1);
const _USERS_STAKES_DETAILS_MEMORY_ID: MemoryId = MemoryId::new(2);
const _USERS_MARGIN_BALANCE_MEMORY_ID: MemoryId = MemoryId::new(3);
const _APPROVED_MARKETS_MEMORY_ID: MemoryId = MemoryId::new(4);
const _VAULT_STAKING_DETAILS_MEMORY: MemoryId = MemoryId::new(5);
const _ADMIN_MEMORY_ID: MemoryId = MemoryId::new(6);

thread_local! {

    static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;


    static VAULT_DETAILS :RefCell<StableCell<VaultDetails,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(_VAULT_DETAILS_MEMORY_ID)
    }),VaultDetails::default()).unwrap());

    static VAULT_STAKING_DETAILS :RefCell<StableCell<VaultStakingDetails,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(_VAULT_STAKING_DETAILS_MEMORY)
    }),VaultStakingDetails::default()).unwrap());

    static USERS_STAKES :RefCell<StableBTreeMap<(Principal,Time),StakeDetails,Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(_USERS_STAKES_DETAILS_MEMORY_ID)
    })));

    static USERS_MARGIN_BALANCE :RefCell<StableBTreeMap<Principal,Amount,Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(_USERS_MARGIN_BALANCE_MEMORY_ID)
    })));

    static APPROVED_MARKETS :RefCell<StableBTreeMap<Principal,bool,Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(_APPROVED_MARKETS_MEMORY_ID)
    })));
    static ADMIN: RefCell<StableCell<Principal, Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|reference| {
        reference.get(_ADMIN_MEMORY_ID)
    }), Principal::anonymous()).unwrap());

}

#[ic_cdk::init]
fn init(vault_details: VaultDetails) {
    let caller = ic_cdk::caller();
    ADMIN.with_borrow_mut(|admin| {
        admin.set(caller).unwrap();
    });
    VAULT_DETAILS.with_borrow_mut(|reference| reference.set(vault_details).unwrap());
}

/// Gets the current margin balance for a user
///
/// # Arguments
/// * `user` - Principal ID of the user to get balance for
///
/// # Returns
/// * `Amount` - User's current margin balance in atomic units
///
/// # Notes
/// - Returns 0 if user has no margin balance
/// - Margin balance represents funds available for creating positions and providing leverage
#[ic_cdk::query(name = "getUserMarginBalance")]
fn get_user_margin_balance(user: Principal) -> Amount {
    return _get_user_margin_balance(user);
}

/// Gets all active stakes for a user
///
/// # Arguments
/// * `user` - Principal ID of the user to get stakes for
///
/// # Returns
/// * `Vec<(Time, StakeDetails)>` - Vector of tuples containing:
///   - Timestamp when stake was created
///   - Details of the stake including amount and other parameters
///
/// # Notes
/// - Returns empty vector if user has no active stakes
/// - Stakes are ordered by timestamp
#[ic_cdk::query(name = "getUserStakes")]
fn get_user_stakes(user: Principal) -> Vec<(Time, StakeDetails)> {
    return _get_user_stakes(user);
}

/// Gets the current staking details for the vault
///
/// # Returns
/// * `VaultStakingDetails` - Current vault staking state including:
///   - Total staked amount
///   - Free liquidity available for lending
///   - Outstanding debt
///   - Distribution of stakes across time spans
///   - Fee accumulation data
///
/// # Notes
/// - Used to check vault capacity and health
/// - Helps determine if new positions can be opened
/// - Provides data for fee distribution calculations
#[ic_cdk::query(name = "getVaultStakingDetails")]
fn get_vault_staking_details() -> VaultStakingDetails {
    _get_vault_staking_details()
}

#[ic_cdk::query(name = "getVaultDetails")]
fn get_vault_details() -> VaultDetails {
    _get_vault_details()
}

/// Validates and processes a position creation request
///
/// # Arguments
/// * `user` - Principal ID of the user creating position
/// * `collateral` - Amount of collateral to lock
/// * `debt` - Amount of leverage to borrow
///
/// # Returns
/// * `(bool, u32)` - (validity status, interest rate)
///   - First value indicates if user has sufficient margin balance and vault has enough liquidity
///   - Second value is the interest rate for the borrowed amount
///
/// If valid, updates user's margin balance and vault's free liquidity by reducing both
#[ic_cdk::update(name = "createPositionValidityCheck", guard = "approved_market_guard")]
async fn create_position_validity_check(
    user: Principal,
    collateral: Amount,
    debt: Amount,
) -> (bool, u32) {
    let account_balance = _get_user_margin_balance(user);

    let mut staking_details = _get_vault_staking_details();

    let valid = account_balance >= collateral && staking_details.free_liquidity >= debt;

    if valid {
        staking_details.free_liquidity -= debt;
        staking_details.debt += debt;
        _update_user_balance(user, collateral, false);
    }

    _update_vault_staking_details(staking_details);

    return (valid, 0);
}

/// Updates position state and distributes fees when a position is modified or closed
///
/// # Arguments
/// * `user` - Principal ID of position owner
/// * `margin_delta` - Amount to return to user's margin balance
/// * `manage_debt_params` - Parameters for debt repayment and fee calculation
///
/// # Effects
/// - Updates user margin balance
/// - Adjusts vault debt and liquidity
/// - Distributes earned fees across stake spans
#[ic_cdk::update(name = "managePositionUpdate", guard = "approved_market_guard")]
async fn manage_position_update(
    user: Principal,
    margin_delta: Amount,
    manage_debt_params: ManageDebtParams,
) {
    if margin_delta != 0 {
        _update_user_balance(user, margin_delta, true);
    }

    let mut staking_details = _get_vault_staking_details();

    let ManageDebtParams {
        initial_debt,
        net_debt,
        amount_repaid,
    } = &manage_debt_params;

    staking_details.debt = staking_details.debt + net_debt - (initial_debt + amount_repaid);
    staking_details.free_liquidity += amount_repaid;

    let fees_gotten = if amount_repaid > initial_debt {
        amount_repaid - initial_debt
    } else {
        return; // basically no fees
    };

    staking_details.lifetime_fees += fees_gotten;
    staking_details._update_fees_across_span(fees_gotten);
    _update_vault_staking_details(staking_details);
}

/// Funds a user's account with assets
///
/// # Arguments
/// * `amount` - Amount of tokens to deposit
/// * `from_subaccount` - Optional subaccount to transfer from
/// * `receiver` - Principal ID of account to fund
///
/// # Returns
/// * `bool` - True if funding succeeded, false otherwise
///
/// # Notes
/// - Transfers tokens from caller's account to canister
/// - Updates receiver's balance if transfer succeeds
/// - Amount must be >= vault's minimum amount
#[ic_cdk::update(name = "fundAccount")]
async fn fund_account(
    amount: Amount,
    from_subaccount: Option<Subaccount>,
    receiver: Principal,
) -> Result<Amount, String> {
    let vault_details = _get_vault_details();
    if amount < vault_details.min_amount {
        return Err("Amount is less than min amount".to_string());
    }

    let depositor = ic_cdk::caller();

    let asset = vault_details.asset;

    let tx_valid = asset
        .move_asset(
            amount,
            Account {
                owner: depositor,
                subaccount: from_subaccount,
            },
            Account {
                owner: ic_cdk::id(),
                subaccount: None,
            },
            false,
        )
        .await;
    if tx_valid {
        _update_user_balance(receiver, amount, true);
        return Ok(amount);
    }

    return Err("transaction failed".to_string());
}

#[ic_cdk::update(name = "withdrawFromAccount")]

async fn withdraw_from_account(amount: Amount, to_account: Account) -> Result<Amount, String> {
    let vault_details = _get_vault_details();
    if amount < vault_details.min_amount {
        return Err("Amount is less than min amount".to_string());
    }
    let user = ic_cdk::caller();

    _update_user_balance(user, amount, false);

    let asset = vault_details.asset;
    let tx_valid = asset
        .move_asset(
            amount,
            Account {
                owner: ic_cdk::id(),
                subaccount: None,
            },
            to_account,
            true,
        )
        .await;
    if !tx_valid {
        _update_user_balance(user, amount, true);
        return Err("transaction failed".to_string());
    }

    return Ok(amount);
}

///////////////////////////
///  Stakers Functions
//////////////////////////

/// Provide Leverage Function

/// Provides leverage by converting user's funding balance to virtual tokens
///
/// # Arguments
/// * `amount` - Amount of tokens to convert to virtual tokens
///
/// # Returns
/// * `Result<bool, String>` - Ok(true) if successful, Err with message if failed
///
/// # Notes
/// - Deducts amount from user's funding balance
/// - Mints equivalent virtual tokens to user's account
/// - Creates a stake of the Instant span type
/// - Updates vault's free liquidity
/// - Amount must be >= vault's minimum amount
/// - Reverts funding balance change if virtual token transfer fails
#[ic_cdk::update(name = "provideLeverage")]
async fn provide_leverage(amount: Amount) -> Result<bool, String> {
    let user = ic_cdk::caller();

    let vault_details = _get_vault_details();

    if amount < vault_details.min_amount {
        return Err("Amount is less than min amount".to_string());
    }
    _update_user_balance(user, amount, false);

    let virtual_asset = vault_details.virtual_asset;

    let mint_tx_valid = virtual_asset
        .move_asset(
            amount,
            Account {
                owner: ic_cdk::id(),
                subaccount: None,
            },
            Account {
                owner: user,
                subaccount: None,
            },
            true,
        )
        .await;
    if !mint_tx_valid {
        _update_user_balance(user, amount, true);
        return Err("Error occured during minting transaction".to_string());
    }

    let mut staking_details = _get_vault_staking_details();
    staking_details.free_liquidity += amount;

    let stake: StakeDetails = staking_details._create_stake(amount, StakeSpan::Instant);
    _insert_user_stake(user, stake);
    _update_vault_staking_details(staking_details);

    return Ok(true);
}

/// Removes leverage by burning virtual tokens and returning the equivalent amount to user's balance
///
/// # Arguments
/// * `amount` - Amount of virtual tokens to burn
/// * `from_subaccount` - Optional subaccount to transfer tokens from
#[ic_cdk::update(name = "removeLeverage")]
async fn remove_leverage(
    amount: Amount,
    from_sub_account: Option<Subaccount>,
) -> Result<bool, String> {
    let user = ic_cdk::caller();

    let vault_details = _get_vault_details();

    let mut vault_staking_details = _get_vault_staking_details();

    if amount < vault_details.min_amount || vault_staking_details.free_liquidity < amount {
        return Err(
            "Amount is less than min amount or vault has insufficient free liquidity".to_string(),
        );
    }

    // reduce vault staking details first before inter cansiter call to avoid in-consistent state
    vault_staking_details.free_liquidity -= amount;

    let VaultDetails { virtual_asset, .. } = vault_details;

    let burn_tx_valid = virtual_asset
        .move_asset(
            amount,
            Account {
                owner: user,
                subaccount: from_sub_account,
            },
            Account {
                owner: ic_cdk::id(),
                subaccount: None,
            },
            false,
        )
        .await;
    if !burn_tx_valid {
        vault_staking_details.free_liquidity += amount;
        return Err("Error occured during burning transaction".to_string());
    }

    _update_user_balance(user, amount, true);
    _update_vault_staking_details(vault_staking_details);
    return Ok(true);
}

/// Stake Virtual Tokens Function
///
/// Stakes virtual tokens in a staking position for a specified duration
///
/// # Arguments
/// * `amount` - Amount of virtual tokens to stake
/// * `stake_span` - Duration to lock tokens (2 months, 6 months, or 1 year)
/// * `from_subaccount` - Optional subaccount to transfer tokens from
///
/// # Returns
/// * `bool` - True if staking succeeded, false if failed
///
#[ic_cdk::update(name = "stakeVirtualTokens")]
async fn stake_virtual_tokens(
    amount: Amount,
    stake_span: StakeSpan,
    from_subaccount: Option<Subaccount>,
) -> Result<Amount, &'static str> {
    if let StakeSpan::Instant = stake_span {
        return Err("Can not stake with instant stakespan");
    };
    let user = ic_cdk::caller();
    let vault_details = _get_vault_details();

    if amount < vault_details.min_amount {
        return Err("Amount less than min amount");
    }

    let virtual_asset = vault_details.virtual_asset;

    let tx_valid = virtual_asset
        .move_asset(
            amount,
            Account {
                owner: user,
                subaccount: from_subaccount,
            },
            Account {
                owner: ic_cdk::id(),
                subaccount: None,
            },
            false,
        )
        .await;
    if !tx_valid {
        return Err("Deposit transaction failed");
    }
    let mut staking_details = _get_vault_staking_details();

    let stake = staking_details._create_stake(amount, stake_span);

    _insert_user_stake(user, stake);
    _update_vault_staking_details(staking_details);

    return Ok(amount);
}
/// Unstakes virtual tokens and returns them to the user with earned rewards
///
/// # Arguments
/// * `stake_timestamp` - Timestamp of the stake to unstake
///
/// # Returns
/// * `Ok(Amount)` - Amount of tokens returned including rewards
/// * `Err(String)` - Error message if unstaking fails

#[ic_cdk::update(name = "unStakeVirtualTokens")]
async fn unstake_virtual_tokens(stake_timestamp: Time) -> Result<Amount, String> {
    let user = ic_cdk::caller();
    let ref_stake = _get_user_stake(user, stake_timestamp);

    if ic_cdk::api::time() < ref_stake.expiry_time {
        return Err("Expiry time in the future".to_string());
    };

    let vault_details = _get_vault_details();

    let mut vault_staking_details = _get_vault_staking_details();

    let stake_earnings = vault_staking_details._calc_stake_earnings(ref_stake);

    let amount_to_send = match ref_stake.stake_span {
        StakeSpan::Instant => stake_earnings,
        _ => ref_stake.amount + stake_earnings,
    };

    let tx_valid = vault_details
        .virtual_asset
        .move_asset(
            amount_to_send,
            Account {
                owner: ic_cdk::id(),
                subaccount: None,
            },
            Account {
                owner: user,
                subaccount: None,
            },
            true,
        )
        .await;
    if !tx_valid {
        return Err("transaction failed".to_string());
    }

    vault_staking_details._close_stake(ref_stake);
    _remove_user_stake(user, stake_timestamp);
    _update_vault_staking_details(vault_staking_details);

    return Ok(0);
}

/// Update user balance

fn _update_user_balance(user: Principal, delta: Amount, deposit: bool) {
    USERS_MARGIN_BALANCE.with_borrow_mut(|reference| {
        let initial_balance = { reference.get(&user).or(Some(0)).unwrap() };
        let new_balance = if deposit {
            initial_balance + delta
        } else {
            initial_balance - delta
        };
        if new_balance == 0 {
            reference.remove(&user)
        } else {
            reference.insert(user, new_balance)
        }
    });
}

fn _get_vault_staking_details() -> VaultStakingDetails {
    VAULT_STAKING_DETAILS.with(|reference| reference.borrow().get().clone())
}

fn _update_vault_staking_details(new_details: VaultStakingDetails) {
    VAULT_STAKING_DETAILS.with_borrow_mut(|reference| [reference.set(new_details).unwrap()]);
}

fn _get_vault_details() -> VaultDetails {
    VAULT_DETAILS.with(|reference| reference.borrow().get().clone())
}

fn _update_vault_details(new_details: VaultDetails) {
    VAULT_DETAILS.with_borrow_mut(|reference| [reference.set(new_details).unwrap()]);
}

fn _get_user_margin_balance(user: Principal) -> Amount {
    USERS_MARGIN_BALANCE.with_borrow_mut(|reference| {
        return reference.get(&user).or(Some(0)).unwrap();
    })
}

fn _get_user_stake(user: Principal, timestamp: Time) -> StakeDetails {
    USERS_STAKES.with_borrow(|reference| reference.get(&(user, timestamp)).unwrap())
}

fn _get_user_stakes(user: Principal) -> Vec<(Time, StakeDetails)> {
    USERS_STAKES.with_borrow(|reference| {
        let iter_map = reference.iter().filter_map(|entries| {
            if entries.0 .0 == user {
                return Some((entries.0 .1, entries.1));
            }
            None
        });

        let mut array = Vec::new();
        for stake in iter_map {
            array.push(stake)
        }
        return array;
    })
}

fn _insert_user_stake(user: Principal, stake: StakeDetails) {
    let timestamp = ic_cdk::api::time();
    USERS_STAKES.with_borrow_mut(|reference| reference.insert((user, timestamp), stake));
}

fn _remove_user_stake(user: Principal, timestamp: Time) {
    USERS_STAKES.with_borrow_mut(|reference| reference.remove(&(user, timestamp)));
}

/// Approved Markets Guard
///
/// Ensures that only approved markets can call the specified functions
fn approved_market_guard() -> Result<(), String> {
    let caller = ic_cdk::caller();
    APPROVED_MARKETS.with_borrow(|reference| {
        if reference.contains_key(&caller) {
            return Ok(());
        } else {
            return Err("Caller not an approved market".to_string());
        }
    })
}

// --------------------------------------------------------------------------------------
// Admin Functions
// --------------------------------------------------------------------------------------

/// Approves a market canister to interact with the vault
///
/// This function allows the admin to approve new market canisters that can interact with
/// the vault's functionality. Only approved markets can call guarded functions.
///
/// # Arguments
/// * `market` - The Principal ID of the market canister to approve
///
/// # Returns
/// * `Ok(())` if the market was successfully approved
/// * `Err(String)` if the caller is not the admin
///
/// # Access Control
/// Only the admin (set during initialization) can call this function
#[ic_cdk::update(name = "approveMarket")]
fn approve_market(market: Principal) -> Result<(), String> {
    // Only allow canister owner/admin to approve markets
    let caller = ic_cdk::caller();
    ADMIN.with_borrow(|admin| {
        if &caller != admin.get() {
            return Err("Only admin can approve markets".to_string());
        };
        APPROVED_MARKETS.with_borrow_mut(|reference| {
            reference.insert(market, true);
        });

        return Ok(());
    })
}

#[derive(Copy, Clone, Default, Deserialize, CandidType)]
struct ManageDebtParams {
    initial_debt: Amount,
    net_debt: Amount,
    amount_repaid: Amount,
}

ic_cdk::export_candid!();
#[cfg(test)]
pub mod closed_integration_tests;
pub mod core_lib;
pub mod types;
