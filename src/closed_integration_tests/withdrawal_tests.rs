use super::*;

#[test]
fn test_that_withdrawal_fails_small_amounts() {
    let pic = PocketIc::new();

    let (token_id, _, vault_id) = _setup_vault(&pic, 1000000);

    let caller = _get_principals()[1];
    _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

    let amount_to_withdraw = 1000;

    let tx_result = _withdraw_from_account(&pic, vault_id, amount_to_withdraw, caller);

    assert!(tx_result.is_err_and(|val| { val == "Amount is less than min amount".to_string() }))
}

#[test]
fn test_that_margn_balance_updates_correctly_if_successful() {
    let pic = PocketIc::new();

    let (token_id, _, vault_id) = _setup_vault(&pic, 0);

    let caller = _get_principals()[1];
    _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

    let margin_balance_before = _get_user_margin_balance(&pic, vault_id, caller);

    let amount_to_withdraw = 1000;

    let _ = _withdraw_from_account(&pic, vault_id, amount_to_withdraw, caller);

    let margin_balance_after = _get_user_margin_balance(&pic, vault_id, caller);

    assert_eq!(
        margin_balance_after,
        margin_balance_before - amount_to_withdraw
    );
}

pub fn _mint_approve_and_fund_account(
    init_pic: &PocketIc,
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
    //minting
    let _ = _icrc1_transfer(&init_pic, token_id, args, Principal::anonymous());

    let deposit_ampount = 10000000000u128;

    let args = ApproveArgs {
        from_subaccount: None,
        created_at_time: None,
        expected_allowance: None,
        fee: None,
        memo: None,
        spender: Account {
            owner: vault_id,
            subaccount: None,
        },
        amount: Nat::from(deposit_ampount),
        expires_at: None,
    };
    let _ = _icrc2_approve(&init_pic, token_id, args, account0);

    let tx_result = _fund_account(
        &init_pic,
        vault_id,
        deposit_ampount,
        None,
        account0,
        account0,
    );

    assert!(tx_result.is_ok());

    let balance = _get_user_margin_balance(&init_pic, vault_id, account0);

    assert_eq!(balance, deposit_ampount);
}
