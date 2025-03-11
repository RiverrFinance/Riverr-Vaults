use super::*;
use icrc_ledger_types::icrc1::account::Account;
mod providing_leverage {
    use super::*;

    #[test]
    fn test_margin_balance_changes_after_liquidity_provision() {
        let pic = PocketIc::new();

        let (token_id, vtoken_id, vault_id) = _setup_vault(&pic, 0);

        let caller = _get_principals()[1];

        _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

        let margin_balance_before = _get_user_margin_balance(&pic, vault_id, caller);

        let amount_utilised = 1000000u128;

        let _ = _provide_leverage(&pic, vault_id, amount_utilised, caller);

        let margin_balance_after = _get_user_margin_balance(&pic, vault_id, caller);

        assert_eq!(
            margin_balance_before - amount_utilised,
            margin_balance_after
        );

        let vtoken_balance = _icrc1_balance_of(
            &pic,
            vtoken_id,
            Account {
                owner: caller,
                subaccount: None,
            },
            caller,
        );

        assert_eq!(Nat::from(amount_utilised), vtoken_balance);
    }

    #[test]
    fn test_vault_liquidity_changes() {
        let pic = PocketIc::new();

        let (token_id, _, vault_id) = _setup_vault(&pic, 0);

        let vault_staking_details_before =
            _get_vault_staking_details(&pic, vault_id, Principal::anonymous());

        let initial_free_liquidity = vault_staking_details_before.free_liquidity;
        let caller = _get_principals()[1];

        _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

        let amount_utilised = 1000000u128;

        let _ = _provide_leverage(&pic, vault_id, amount_utilised, caller);

        let vault_staking_details_after =
            _get_vault_staking_details(&pic, vault_id, Principal::anonymous());

        let free_liquidity_after = vault_staking_details_after.free_liquidity;

        assert_eq!(
            initial_free_liquidity + amount_utilised,
            free_liquidity_after
        );
    }

    #[test]
    pub fn test_user_stakes_update_after_liquidity_provision() {
        let pic = PocketIc::new();

        let (token_id, _, vault_id) = _setup_vault(&pic, 0);

        let caller = _get_principals()[1];

        _mint_approve_and_fund_account(&pic, vault_id, caller, token_id);

        let amount_utilised = 1000000u128;

        let _ = _provide_leverage(&pic, vault_id, amount_utilised, caller);

        let user_stakes = _get_user_stakes(&pic, vault_id, caller);

        assert!(user_stakes.len() == 1);

        let user_stake = user_stakes[0].1;

        assert_eq!(user_stake.stake_span, StakeSpan::Instant);
        assert_eq!(user_stake.amount, amount_utilised)
    }
}
