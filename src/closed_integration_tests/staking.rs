use super::*;

#[test]
fn test_creating_stake_with_stakespan_instant_fails() {
    let caller = _get_principals()[1];

    let pic = PocketIc::new();

    let (token_id, vtoken_id, vault_id) = _setup_vault(&pic, 0);

    _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

    let amount_utilised = 1000000u128;

    let _ = _provide_leverage(&pic, vault_id, amount_utilised, caller);

    _approve_spending(&pic, vtoken_id, amount_utilised, caller, vault_id);

    let tx_result = _stake(
        &pic,
        caller,
        vault_id,
        amount_utilised,
        LockSpan::Instant,
        None,
    );

    assert!(
        tx_result.is_err_and(|err| { err == String::from("Can not stake with instant stakespan") })
    );
}

#[test]
fn test_creating_stake_with_amount_less_than_min_amount_fails() {
    let caller = _get_principals()[1];

    let pic = PocketIc::new();

    let amount_to_utilise = 1000000u128;

    let (token_id, vtoken_id, vault_id) = _setup_vault(&pic, amount_to_utilise + 100000);

    _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

    let amount_utilised = 1000000u128;

    let _ = _provide_leverage(&pic, vault_id, amount_utilised, caller);

    _approve_spending(&pic, vtoken_id, amount_utilised, caller, vault_id);

    let tx_result = _stake(
        &pic,
        caller,
        vault_id,
        amount_utilised,
        LockSpan::Month2,
        None,
    );

    assert!(tx_result.is_err_and(|err| { err == String::from("Amount less than min amount") }));
}

#[test]
fn test_that_transaction_fails_without_spending_approval_for_canister() {
    let caller = _get_principals()[1];

    let pic = PocketIc::new();

    let (token_id, _vtoken_id, vault_id) = _setup_vault(&pic, 0);

    _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

    let amount_utilised = 1000000u128;

    let _ = _provide_leverage(&pic, vault_id, amount_utilised, caller);

    //_approve_spending(&pic, vtoken_id, amount_utilised, caller, vault_id);

    let tx_result = _stake(
        &pic,
        caller,
        vault_id,
        amount_utilised,
        LockSpan::Month2,
        None,
    );

    assert!(tx_result.is_err_and(|err| { err == String::from("Deposit transaction failed") }));
}

#[test]
fn test_that_despoit_succeeds_when_all_conditons_are_satisfied() {
    let caller = _get_principals()[1];

    let pic = PocketIc::new();

    let (token_id, vtoken_id, vault_id) = _setup_vault(&pic, 0);

    _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

    let amount_utilised = 1000000u128;

    let _ = _provide_leverage(&pic, vault_id, amount_utilised, caller);

    _approve_spending(&pic, vtoken_id, amount_utilised, caller, vault_id);

    let tx_result = _stake(
        &pic,
        caller,
        vault_id,
        amount_utilised,
        LockSpan::Month2,
        None,
    );

    assert!(tx_result.is_ok_and(|mes| { mes == amount_utilised }));

    let user_stakes = _get_user_stakes(&pic, vault_id, caller);

    // the default instant and the
    assert!(user_stakes.len() == 2);

    let default_user_stake = user_stakes[0].1;

    assert_eq!(default_user_stake.stake_span, LockSpan::Instant);
    assert_eq!(default_user_stake.amount, amount_utilised);

    let user_stake = user_stakes[1].1;

    assert_eq!(user_stake.stake_span, LockSpan::Month2);
    assert_eq!(user_stake.amount, amount_utilised)
}
