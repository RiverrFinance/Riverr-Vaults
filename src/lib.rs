use std::cell::RefCell;

use candid::{CandidType, Deserialize, Principal};

use icrc_ledger_types::icrc1::account::{Account, Subaccount};

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};

use core_lib::lock::{LockDetails, LockSpan, Vault};
use types::LiquidityManagerDetails;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type Amount = u128;
type Time = u64;

const LIQUIDITY_MANAGER_DETAILS_MEMORY_ID: MemoryId = MemoryId::new(1);
const _USERS_LOCKS_MEMORY_ID: MemoryId = MemoryId::new(2);
const _USERS_MARGIN_BALANCE_MEMORY_ID: MemoryId = MemoryId::new(3);
const _APPROVED_MARKETS_MEMORY_ID: MemoryId = MemoryId::new(4);
const _VAULT_MEMORY_ID: MemoryId = MemoryId::new(5);
const _ADMIN_MEMORY_ID: MemoryId = MemoryId::new(6);

thread_local! {

    static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;

    static LIQUIDTY_MANAGER_DETAILS :RefCell<StableCell<LiquidityManagerDetails,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(LIQUIDITY_MANAGER_DETAILS_MEMORY_ID)
    }),LiquidityManagerDetails::default()).unwrap());

    static VAULT :RefCell<StableCell<Vault,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(_VAULT_MEMORY_ID)
    }),Vault::default()).unwrap());

    static USERS_LOCKS :RefCell<StableBTreeMap<(Principal,Time),LockDetails,Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with_borrow(|reference|{
        reference.get(_USERS_LOCKS_MEMORY_ID)
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
fn init(details: LiquidityManagerDetails) {
    let caller = ic_cdk::caller();
    ADMIN.with_borrow_mut(|admin| {
        admin.set(caller).unwrap();
    });
    LIQUIDTY_MANAGER_DETAILS.with_borrow_mut(|reference| reference.set(details).unwrap());
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

/// Gets all active locks owned by  a user
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
#[ic_cdk::query(name = "getUserLocks")]
fn get_user_locks(user: Principal) -> Vec<(Time, LockDetails, Amount)> {
    return _get_user_locks(user);
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
#[ic_cdk::query(name = "getVault")]
fn get_vault() -> Vault {
    _get_vault()
}

#[ic_cdk::query(name = "getLiquidityManagerDetails")]
fn get_liquidity_manager_details() -> LiquidityManagerDetails {
    _get_liquidity_manager_details()
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
    let vault_details = _get_liquidity_manager_details();
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
    let vault_details = _get_liquidity_manager_details();
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
///  Vault Functions
//////////////////////////

/// Lend to Vault Function

/// lend liquidity to vault
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
#[ic_cdk::update(name = "lendToVault")]
async fn lend_to_vault(amount: Amount) -> Result<bool, String> {
    let user = ic_cdk::caller();

    let vault_details = _get_liquidity_manager_details();

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

    let mut vault = _get_vault();
    vault.free_liquidity += amount;

    let stake: LockDetails = vault._create_lock(amount, LockSpan::Instant);
    _insert_user_lock(user, stake);
    _update_vault(vault);

    return Ok(true);
}

/// collects debt back from vault by burning virtual tokens and returning the equivalent amount to user's balance
///
/// # Arguments
/// * `amount` - Amount of virtual tokens to burn
/// * `from_subaccount` - Optional subaccount to transfer tokens from
#[ic_cdk::update(name = "collectFromVault")]
async fn collect_from_vault(
    amount: Amount,
    from_sub_account: Option<Subaccount>,
) -> Result<bool, String> {
    let user = ic_cdk::caller();

    let liquidity_manager_details = _get_liquidity_manager_details();

    let mut vault = _get_vault();

    if amount < liquidity_manager_details.min_amount || vault.free_liquidity < amount {
        return Err(
            "Amount is less than min amount or vault has insufficient free liquidity".to_string(),
        );
    }

    // reduce vault staking details first before inter cansiter call to avoid in-consistent state
    vault.free_liquidity -= amount;

    let LiquidityManagerDetails { virtual_asset, .. } = liquidity_manager_details;

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
        vault.free_liquidity += amount;
        return Err("Error occured during burning transaction".to_string());
    }

    _update_user_balance(user, amount, true);
    _update_vault(vault);
    return Ok(true);
}

/// Lock Virtual Tokens Function
///
/// lock virtual tokens in a vault lock for a specified duration
///
/// # Arguments
/// * `amount` - Amount of virtual tokens to stake
/// * `stake_span` - Duration to lock tokens (2 months, 6 months, or 1 year)
/// * `from_subaccount` - Optional subaccount to transfer tokens from
///
/// # Returns
/// * `bool` - True if staking succeeded, false if failed
///
#[ic_cdk::update(name = "lockQTokens")]
async fn lock_qtokens(
    amount: Amount,
    stake_span: LockSpan,
    from_subaccount: Option<Subaccount>,
) -> Result<Amount, &'static str> {
    if let LockSpan::Instant = stake_span {
        return Err("Can not stake with instant stakespan");
    };
    let user = ic_cdk::caller();
    let vault_details = _get_liquidity_manager_details();

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
    let mut vault = _get_vault();

    let lock = vault._create_lock(amount, stake_span);

    _insert_user_lock(user, lock);
    _update_vault(vault);

    return Ok(amount);
}
/// Unlocks virtual tokens and returns them to the user with earned rewards
///
/// Note :This function should omly be called with the ref lock duration has been fully exhausted
///
/// # Arguments
/// * `stake_timestamp` - Timestamp of the stake to unstake
///
/// # Returns
/// * `Ok(Amount)` - Amount of tokens returned including rewards
/// * `Err(String)` - Error message if unstaking fails

#[ic_cdk::update(name = "unlockQTokens")]
async fn unlock_qtokens(lock_timestamp: Time) -> Result<Amount, String> {
    let user = ic_cdk::caller();
    let ref_lock = _get_user_lock(user, lock_timestamp);

    if ic_cdk::api::time() < ref_lock.expiry_time {
        return Err("Expiry time in the future".to_string());
    };

    let liquidity_manager_details = _get_liquidity_manager_details();

    let mut vault = _get_vault();

    let lock_earnings = vault._calc_lock_earnings(ref_lock);

    let amount_to_send = match ref_lock.stake_span {
        LockSpan::Instant => lock_earnings,
        _ => ref_lock.amount + lock_earnings,
    };

    let tx_valid = liquidity_manager_details
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

    vault._open_lock(ref_lock);
    _remove_user_lock(user, lock_timestamp);
    _update_vault(vault);

    return Ok(amount_to_send);
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
#[ic_cdk::update(name = "liquidityChangeValidityCheck", guard = "approved_market_guard")]
async fn liquidity_change_validity_check(
    user: Principal,
    collateral: Amount,
    debt: Amount,
) -> (bool, u32) {
    let account_balance = _get_user_margin_balance(user);

    let mut vault = _get_vault();

    let valid = account_balance >= collateral && vault.free_liquidity >= debt;

    if valid {
        vault.free_liquidity -= debt;
        vault.debt += debt;
        _update_user_balance(user, collateral, false);
    }

    _update_vault(vault);

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

    let mut vault = _get_vault();

    let ManageDebtParams {
        initial_debt,
        net_debt,
        amount_repaid,
    } = &manage_debt_params;

    vault.debt = vault.debt + net_debt - (initial_debt + amount_repaid);
    vault.free_liquidity += amount_repaid;

    let fees_gotten = if amount_repaid > initial_debt {
        amount_repaid - initial_debt
    } else {
        return; // basically no fees
    };

    vault.lifetime_fees += fees_gotten;
    vault._update_fees_across_span(fees_gotten);
    _update_vault(vault);
}

/// Update user balance

fn _update_user_balance(user: Principal, delta: Amount, deposit: bool) {
    USERS_MARGIN_BALANCE.with_borrow_mut(|reference| {
        let initial_balance = { reference.get(&user).unwrap_or_default() };
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

fn _get_vault() -> Vault {
    VAULT.with(|reference| reference.borrow().get().clone())
}

fn _update_vault(new_details: Vault) {
    VAULT.with_borrow_mut(|reference| [reference.set(new_details).unwrap()]);
}

fn _get_liquidity_manager_details() -> LiquidityManagerDetails {
    LIQUIDTY_MANAGER_DETAILS.with(|reference| reference.borrow().get().clone())
}

fn _update_liquidity_manager_details(new_details: LiquidityManagerDetails) {
    LIQUIDTY_MANAGER_DETAILS.with_borrow_mut(|reference| [reference.set(new_details).unwrap()]);
}

fn _get_user_margin_balance(user: Principal) -> Amount {
    USERS_MARGIN_BALANCE.with_borrow_mut(|reference| {
        return reference.get(&user).or(Some(0)).unwrap();
    })
}

fn _get_user_lock(user: Principal, timestamp: Time) -> LockDetails {
    USERS_LOCKS.with_borrow(|reference| reference.get(&(user, timestamp)).unwrap())
}

fn _get_user_locks(user: Principal) -> Vec<(Time, LockDetails, Amount)> {
    USERS_LOCKS.with_borrow(|reference| {
        let iter_map = reference.iter().filter_map(|entries| {
            if entries.0 .0 == user {
                let ref_lock = entries.1;
                let vault = _get_vault();
                let fees_earned = vault._calc_lock_earnings(ref_lock);
                return Some((entries.0 .1, ref_lock, fees_earned));
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

fn _insert_user_lock(user: Principal, stake: LockDetails) {
    let timestamp = ic_cdk::api::time();
    USERS_LOCKS.with_borrow_mut(|reference| reference.insert((user, timestamp), stake));
}

fn _remove_user_lock(user: Principal, timestamp: Time) {
    USERS_LOCKS.with_borrow_mut(|reference| reference.remove(&(user, timestamp)));
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
