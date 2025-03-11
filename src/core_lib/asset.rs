use candid::{CandidType, Nat, Principal};
use ic_cdk;

use icrc_ledger_types::{
    icrc1::{
        account::{Account, Subaccount},
        transfer::{TransferArg, TransferError},
    },
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};

use ic_ledger_types::{
    transfer, AccountIdentifier, Memo, Subaccount as ICSubaccount, Tokens,
    TransferArgs as ICRCTransferArgs, DEFAULT_FEE, DEFAULT_SUBACCOUNT,
};
use serde::{Deserialize, Serialize};

type Amount = u128;

#[derive(CandidType, Deserialize, Serialize, Clone, Copy)]
pub enum AssetType {
    ICP,
    ICRC,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Copy)]
pub struct Asset {
    pub ledger_id: Principal,
    pub asset_type: AssetType,
}
impl Default for Asset {
    fn default() -> Self {
        return Asset {
            ledger_id: Principal::anonymous(),
            asset_type: AssetType::ICRC,
        };
    }
}

impl Asset {
    pub async fn move_asset(
        &self,
        amount: Amount,
        from_account: Account,
        to_account: Account,
        out: bool,
    ) -> bool {
        match self.asset_type {
            AssetType::ICP => {
                move_asset_icp(amount, self.ledger_id, from_account.subaccount, to_account).await
            }
            AssetType::ICRC => {
                if out {
                    send_asset_out_icrc(amount, self.ledger_id, from_account.subaccount, to_account)
                        .await
                } else {
                    send_asset_in_asset_icrc(amount, self.ledger_id, from_account, to_account).await
                }
            }
        }
    }
}

/// Transfers ICP tokens between accounts on the Internet Computer
///
/// # Arguments
/// * `amount` - Amount of ICP tokens to transfer (in e8s)
/// * `ledger_id` - Principal ID of the ICP ledger canister
/// * `from_sub` - Optional subaccount to transfer from
/// * `to_account` - Destination account details including owner and subaccount
///
/// # Returns
/// * `bool` - True if transfer succeeded, false otherwise
///
/// # Notes
/// - Returns early with true if amount is 0
/// - Uses default fee and memo(0) for all transfers
/// - Handles nested Result types from IC ledger response
async fn move_asset_icp(
    amount: Amount,
    ledger_id: Principal,
    from_sub: Option<Subaccount>,
    to_account: Account,
) -> bool {
    if amount == 0 {
        return true;
    }

    let args = ICRCTransferArgs {
        amount: Tokens::from_e8s(amount as u64),
        memo: Memo(0),
        fee: DEFAULT_FEE,
        from_subaccount: Some(_to_ic_subaccount(from_sub)),
        to: AccountIdentifier::new(&to_account.owner, &_to_ic_subaccount(to_account.subaccount)),
        created_at_time: None,
    };

    match transfer(ledger_id, args).await {
        Ok(res) => {
            if let Ok(_) = res {
                return true;
            } else {
                return false;
            }
        }
        Err(_) => return false,
    };
}

/// Transfers ICRC tokens from the canister to an external account
///
/// # Arguments
/// * `amount` - Amount of tokens to transfer
/// * `ledger_id` - Principal ID of the token's ledger canister
/// * `from_subaccount` - Optional subaccount to transfer from
/// * `to_account` - Destination account details
///
/// # Returns
/// * `bool` - True if transfer succeeded, false otherwise
///
/// # Notes
/// - Uses ICRC1 standard transfer call
/// - Does not specify fee, memo or timestamp (all None)
/// - Returns false on any error in the transfer
/// - Handles nested Result types from IC ledger response
async fn send_asset_out_icrc(
    amount: Amount,
    ledger_id: Principal,
    from_subaccount: Option<Subaccount>,
    to_account: Account,
) -> bool {
    // Error: Typo in struct name ICRCTransferrgs -> ICRCTransferArgs
    let args = TransferArg {
        amount: Nat::from(amount),
        from_subaccount,
        to: to_account,
        fee: None,
        created_at_time: None,
        memo: None,
    };

    let tx_result: Result<Nat, TransferError>;

    if let Ok((result,)) = ic_cdk::call(ledger_id, "icrc1_transfer", (args,)).await {
        tx_result = result;
        if let Ok(_) = tx_result {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

/// Transfers ICRC2 tokens from one account to another using the spender's allowance
///
/// # Arguments
/// * `amount` - Amount of tokens to transfer
/// * `ledger_id` - Principal ID of the token's ledger canister
/// * `from_account` - Source account to transfer from
/// * `to_account` - Destination account to transfer to
///
/// # Returns
/// * `bool` - True if transfer succeeded, false otherwise
///
/// # Notes
/// - Uses ICRC2 standard transferFrom call
/// - Requires prior approval/allowance from source account for the None subaccount of the canister
/// - Does not specify fee, memo, timestamp or spender subaccount (all None)
/// - Returns false on any error in the transfer
/// - Handles nested Result types from IC ledger response
pub async fn send_asset_in_asset_icrc(
    amount: Amount,
    ledger_id: Principal,
    from_account: Account,
    to_account: Account,
) -> bool {
    let args = TransferFromArgs {
        spender_subaccount: None,
        from: from_account,
        to: to_account,
        amount: Nat::from(amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let tx_result: Result<Nat, TransferFromError>;

    if let Ok((result,)) = ic_cdk::call(ledger_id, "icrc2_transfer_from", (args,)).await {
        tx_result = result;
        if let Ok(_) = tx_result {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

fn _to_ic_subaccount(sub: Option<Subaccount>) -> ICSubaccount {
    match sub {
        Some(res) => return ICSubaccount(res),
        None => return DEFAULT_SUBACCOUNT,
    }
}
