use candid::{decode_one, encode_one, CandidType, Nat, Principal};

use icrc_ledger_types::{
    icrc1::transfer::{TransferArg, TransferError},
    icrc2::approve::{ApproveArgs, ApproveError},
};
use pocket_ic::{PocketIc, WasmResult};
use std::fs;

use super::*;

use crate::types::VaultDetails;

use crate::core_lib::asset::{Asset, AssetType};

const TOKEN_WASM: &str = "target/wasm32-unknown-unknown/release/token.wasm";
const VAULT_WASM: &str = "target/wasm32-unknown-unknown/release/vault.wasm";

pub mod deposit_test;
pub mod staking;
pub mod test_providing_leverage;
pub mod withdrawal_tests;

pub fn _setup_vault(init_pic: &PocketIc, min_amount: u128) -> (Principal, Principal, Principal) {
    // Create new PocketIC instance
    let pic = init_pic;

    // Install token canister
    let token_id = pic.create_canister();

    pic.add_cycles(token_id, 2_000_000_000_000); // 2T Cycles

    let vault_wasm = fs::read(VAULT_WASM).expect("Wasm file not found, run 'dfx build'.");

    let token_wasm = fs::read(TOKEN_WASM).expect("Wasm file not found, run 'dfx build'.");

    let args: LedgerArg = LedgerArg::Init(create_args(Principal::anonymous()));

    pic.install_canister(
        token_id,
        token_wasm.clone(),
        encode_one(args).unwrap(),
        Some(Principal::anonymous()),
    );

    let vault_id = pic.create_canister();

    pic.add_cycles(vault_id, 2_000_000_000_000); // 2T Cycles

    let vtoken_id = pic.create_canister();

    pic.add_cycles(vtoken_id, 2_000_000_000_000); // 2T Cycles

    let vtoken_args: LedgerArg = LedgerArg::Init(create_args(vault_id));

    pic.install_canister(
        vtoken_id,
        token_wasm,
        encode_one(vtoken_args).unwrap(),
        Some(Principal::anonymous()),
    );

    let vauilt_args = VaultDetails {
        asset: Asset {
            asset_type: AssetType::ICRC,
            ledger_id: token_id,
        },
        virtual_asset: Asset {
            asset_type: AssetType::ICRC,
            ledger_id: vtoken_id,
        },

        min_amount,
    };
    // let vault_wasm = fs::read(VAULT_WASM).expect("Wasm file not found, run 'dfx build'.");
    pic.install_canister(
        vault_id,
        vault_wasm,
        encode_one(vauilt_args).unwrap(),
        Some(Principal::anonymous()),
    );

    (token_id, vtoken_id, vault_id)
}

//////////////////////////////////////////////////////////////////
///

pub fn _get_user_margin_balance(pic: &PocketIc, vault_id: Principal, user: Principal) -> Amount {
    let Ok(WasmResult::Reply(val)) = pic.query_call(
        vault_id,
        Principal::anonymous(),
        "getUserMarginBalance",
        encode_one(user).unwrap(),
    ) else {
        panic!("Could not get user margin balance")
    };

    let reply: Amount = decode_one(&val).unwrap();
    reply
}

pub fn _provide_leverage(
    pic: &PocketIc,
    vault_id: Principal,
    amount: Amount,
    caller: Principal,
) -> Result<bool, String> {
    let Ok(WasmResult::Reply(val)) = pic.update_call(
        vault_id,
        caller,
        "provideLeverage",
        encode_one(amount).unwrap(),
    ) else {
        panic!("Could not provide leverage")
    };

    let reply: Result<bool, String> = decode_one(&val).unwrap();
    reply
}

pub fn _icrc1_transfer(
    pic: &PocketIc,
    token_id: Principal,
    args: TransferArg,
    caller: Principal,
) -> Result<Nat, TransferError> {
    let Ok(WasmResult::Reply(val)) = pic.update_call(
        token_id,
        caller,
        "icrc1_transfer",
        encode_one(args).unwrap(),
    ) else {
        panic!("Transfer failed")
    };

    let reply = decode_one(&val).unwrap();
    reply
}

pub fn _icrc1_balance_of(
    pic: &PocketIc,
    token_id: Principal,
    account: Account,
    caller: Principal,
) -> Nat {
    match pic.query_call(
        token_id,
        caller,
        "icrc1_balance_of",
        encode_one(account).unwrap(),
    ) {
        Ok(reply) => {
            if let WasmResult::Reply(val) = reply {
                let response = decode_one(&val).unwrap();
                return response;
            } else {
                panic!("error occurred in canister");
            }
        }
        Err(error) => {
            println!("{:?}", error);
            panic!("error getting balance occurred at pocket ic");
        }
    }
}

pub fn _icrc2_approve(
    pic: &PocketIc,
    token_id: Principal,
    args: ApproveArgs,
    caller: Principal,
) -> Result<Nat, ApproveError> {
    match pic.update_call(token_id, caller, "icrc2_approve", encode_one(args).unwrap()) {
        Ok(reply) => {
            if let WasmResult::Reply(val) = reply {
                let reply = decode_one(&val).unwrap();
                reply
            } else {
                panic!("error occurred in canister");
            }
        }
        Err(error) => {
            println!("{:?}", error);
            panic!("error approving occurred at pocket ic");
        }
    }
}

pub fn _get_vault_staking_details(
    pic: &PocketIc,
    vault_id: Principal,
    caller: Principal,
) -> VaultLockDetails {
    match pic.query_call(
        vault_id,
        caller,
        "getVaultStakingDetails",
        candid::encode_args(()).unwrap(),
    ) {
        Ok(reply) => {
            if let WasmResult::Reply(val) = reply {
                let response = candid::decode_one(&val).unwrap();
                return response;
            } else {
                panic!("error occurred in canister");
            }
        }
        Err(error) => {
            println!("{:?}", error);
            panic!("error getting vault staking details occurred at pocket ic");
        }
    }
}

pub fn _get_user_stakes(
    pic: &PocketIc,
    vault_id: Principal,
    caller: Principal,
) -> Vec<(Time, LockDetails)> {
    match pic.query_call(
        vault_id,
        caller,
        "getUserStakes",
        candid::encode_one(caller).unwrap(),
    ) {
        Ok(reply) => {
            if let WasmResult::Reply(val) = reply {
                let response = candid::decode_one(&val).unwrap();
                return response;
            } else {
                panic!("error occurred in canister");
            }
        }
        Err(error) => {
            println!("{:?}", error);
            panic!("error getting user stake occurred at pocket ic");
        }
    }
}

pub fn _stake(
    pic: &PocketIc,
    caller: Principal,
    vault_id: Principal,
    amount: Amount,
    span: LockSpan,
    from_subaccount: Option<Subaccount>,
) -> Result<Amount, String> {
    let Ok(WasmResult::Reply(val)) = pic.update_call(
        vault_id,
        caller,
        "stakeVirtualTokens",
        candid::encode_args((amount, span, from_subaccount)).unwrap(),
    ) else {
        panic!("create stake failed")
    };

    let reply = candid::decode_one(&val);

    match reply {
        Ok(val) => val,
        Err(error) => {
            println!("{:?}", error);
            panic!("error during staking occurred at pocket ic");
        }
    }
}
pub fn _fund_account(
    pic: &PocketIc,
    vault_id: Principal,
    amount: Amount,
    from_subaccount: Option<Subaccount>,
    receiver: Principal,
    sender: Principal,
) -> Result<Amount, String> {
    let Ok(WasmResult::Reply(val)) = pic.update_call(
        vault_id,
        sender,
        "fundAccount",
        candid::encode_args((amount, from_subaccount, receiver)).unwrap(),
    ) else {
        panic!("Fund account failed")
    };

    let reply = candid::decode_one(&val);

    match reply {
        Ok(val) => val,
        Err(error) => {
            println!("{:?}", error);
            panic!("error funding account occurred at pocket ic");
        }
    }
}

pub fn _withdraw_from_account(
    pic: &PocketIc,
    vault_id: Principal,
    amount: Amount,
    receiver: Principal,
) -> Result<Amount, String> {
    let Ok(WasmResult::Reply(val)) = pic.update_call(
        vault_id,
        receiver,
        "withdrawFromAccount",
        candid::encode_args((
            amount,
            Account {
                owner: receiver,
                subaccount: None,
            },
        ))
        .unwrap(),
    ) else {
        panic!("Withdrawl from Account Failed")
    };

    let reply = candid::decode_one(&val);

    match reply {
        Ok(val) => val,
        Err(error) => {
            println!("{:?}", error);
            panic!("error withdrawing from  account occurred at pocket ic");
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
pub fn _approve_spending(
    pic: &PocketIc,
    token_id: Principal,
    amount: Amount,
    caller: Principal,
    spender_id: Principal,
) {
    let args = ApproveArgs {
        from_subaccount: None,
        created_at_time: None,
        expected_allowance: None,
        fee: None,
        memo: None,
        spender: Account {
            owner: spender_id,
            subaccount: None,
        },
        amount: Nat::from(amount),
        expires_at: None,
    };
    let _ = _icrc2_approve(pic, token_id, args, caller);
}
///
///
///
pub fn _mint_approve_and_fund_account(
    pic: &PocketIc,
    vault_id: Principal,
    to_principal: Principal,
    token_id: Principal,
) {
    let account0 = to_principal;

    let args = TransferArg {
        from_subaccount: None,
        created_at_time: None,
        to: Account {
            owner: to_principal,
            subaccount: None,
        },
        amount: Nat::from(1000000000000000000u128),
        fee: None,
        memo: None,
    };

    let _ = _icrc1_transfer(&pic, token_id, args, Principal::anonymous());

    let deposit_ampount = 10000000000u128;

    _approve_spending(pic, token_id, deposit_ampount, account0, vault_id);

    let tx_result = _fund_account(&pic, vault_id, deposit_ampount, None, account0, account0);

    assert!(tx_result.is_ok());

    let balance = _get_user_margin_balance(&pic, vault_id, account0);

    assert_eq!(balance, deposit_ampount);
}

pub fn _get_principals() -> Vec<Principal> {
    return vec![
        Principal::from_text("hpp6o-wqx72-gol5b-3bmzw-lyryb-62yoi-pjoll-mtsh7-swdzi-jkf2v-rqe")
            .unwrap(),
        Principal::from_text("cvwul-djb3r-e6krd-nbnfl-tuhox-n4omu-kejey-3lku7-ae3bx-icbu7-yae")
            .unwrap(),
        Principal::from_text("az6yt-a3f5b-k342j-5jncd-csa66-xfgvb-6f52c-jw5nh-o7g4k-bbf4q-vqe")
            .unwrap(),
        Principal::from_text("kmgqw-63fcp-ugexu-qgkyy-p3vjk-e4jh5-kyrdx-skajz-o3qra-wcyrb-qae")
            .unwrap(),
    ];
}

#[derive(CandidType)]
pub enum LedgerArg {
    Init(InitArgs),
    Upgrade(UpgradeArgs),
}

pub fn create_args(minting_id: Principal) -> InitArgs {
    InitArgs {
        decimals: Some(8),
        token_symbol: "ICP".to_string(),
        transfer_fee: 0,
        metadata: vec![],
        minting_account: Account {
            owner: minting_id,
            subaccount: None,
        },
        initial_balances: vec![],
        maximum_number_of_accounts: None,
        accounts_overflow_trim_quantity: None,
        fee_collector_account: None,
        archive_options: ArchiveOptions {
            num_blocks_to_archive: 1000,
            max_transactions_per_response: None,
            trigger_threshold: 1000,
            max_message_size_bytes: None,
            cycles_for_archive_creation: None,
            node_max_memory_size_bytes: None,
            controller_id: Principal::anonymous(),
        },
        max_memo_length: None,
        token_name: "Internet Computer".to_string(),
        feature_flags: Some(FeatureFlags { icrc2: true }),
    }
}

#[derive(CandidType, Clone)]
pub struct UpgradeArgs {
    pub token_symbol: Option<String>,
    pub transfer_fee: Option<u128>,
    pub metadata: Option<Vec<(String, MetadataValue)>>,
    pub maximum_number_of_accounts: Option<u64>,
    pub accounts_overflow_trim_quantity: Option<u64>,
    pub change_fee_collector: Option<ChangeFeeCollector>,
    pub max_memo_length: Option<u16>,
    pub token_name: Option<String>,
    pub feature_flags: Option<FeatureFlags>,
}

#[derive(Debug, Clone, CandidType)]
pub enum ChangeFeeCollector {
    Unset,
    Set(Account),
}

#[derive(CandidType, Clone)]
pub struct InitArgs {
    pub decimals: Option<u8>,
    pub token_symbol: String,
    pub transfer_fee: u128,
    pub metadata: Vec<(String, MetadataValue)>,
    pub minting_account: Account,
    pub initial_balances: Vec<(Account, u128)>,
    pub maximum_number_of_accounts: Option<u64>,
    pub accounts_overflow_trim_quantity: Option<u64>,
    pub fee_collector_account: Option<Account>,
    pub archive_options: ArchiveOptions,
    pub max_memo_length: Option<u16>,
    pub token_name: String,
    pub feature_flags: Option<FeatureFlags>,
}

#[derive(Debug, Clone, CandidType)]
pub enum MetadataValue {
    Int(i128),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(Debug, Clone, CandidType)]
pub struct ArchiveOptions {
    pub num_blocks_to_archive: u64,
    pub max_transactions_per_response: Option<u64>,
    pub trigger_threshold: u64,
    pub max_message_size_bytes: Option<u64>,
    pub cycles_for_archive_creation: Option<u64>,
    pub node_max_memory_size_bytes: Option<u64>,
    pub controller_id: Principal,
}

#[derive(Debug, Clone, CandidType)]
pub struct FeatureFlags {
    pub icrc2: bool,
}
