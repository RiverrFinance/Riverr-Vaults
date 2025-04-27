use super::*;

use candid::Nat;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::approve::ApproveArgs;

pub mod testing_funding {
    use super::*;

    #[test]
    fn test_principal() {
        print!("Principal: {:?}", Principal::anonymous().to_string());
    }

    /// Tests that deposits fail when not approved
    ///
    /// This test verifies that:
    /// - A deposit attempt without prior approval fails
    /// - The transaction returns false
    /// - The user's margin balance remains 0
    #[test]
    fn test_that_deposit_fails_if_not_approved() {
        let pic = PocketIc::new();
        let (token_id, _, vault_id) = _setup_vault(&pic, 0);

        let account0 = _get_principals()[0];
        let args = TransferArg {
            from_subaccount: None,
            created_at_time: None,
            to: Account {
                owner: account0,
                subaccount: None,
            },
            amount: Nat::from(1000000000000000000u128),
            fee: None,
            memo: None,
        };
        //minting
        let _ = _icrc1_transfer(&pic, token_id, args, Principal::anonymous());

        let account1 = _get_principals()[1];

        let balance_before = _get_user_margin_balance(&pic, vault_id, account1);

        let deposit_amount = 10000000000u128;

        let tx_result = _fund_account(&pic, vault_id, deposit_amount, None, account1, account0);

        assert!(tx_result.is_err_and(|val| { val == "transaction failed".to_string() })); // funding should fail and return false

        let balance_after = _get_user_margin_balance(&pic, vault_id, account1);
        //balance does not change
        assert_eq!(balance_after, balance_before);
    }

    #[test]
    fn test_that_funding_account_fails_when_amount_is_less_than_min_amount() {
        let pic = PocketIc::new();
        let min_amount = 10000000000u128;
        let (token_id, _, vault_id) = _setup_vault(&pic, min_amount);

        let account0 = _get_principals()[0];
        let args = TransferArg {
            from_subaccount: None,
            created_at_time: None,
            to: Account {
                owner: account0,
                subaccount: None,
            },
            amount: Nat::from(1000000000000000000u128),
            fee: None,
            memo: None,
        };
        //minting
        let _ = _icrc1_transfer(&pic, token_id, args, Principal::anonymous());

        let account1 = _get_principals()[1];

        let balance_before = _get_user_margin_balance(&pic, vault_id, account1);

        let deposit_amount = min_amount - 10000; // less than min_amount

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
            amount: Nat::from(deposit_amount),
            expires_at: None,
        };
        let _ = _icrc2_approve(&pic, token_id, args, account0);

        let tx_result = _fund_account(&pic, vault_id, deposit_amount, None, account1, account0);

        assert!(tx_result.is_err_and(|val| { val == "Amount is less than min amount".to_string() })); // funding should fail and return false

        let balance_after = _get_user_margin_balance(&pic, vault_id, account1);

        assert_eq!(balance_after, balance_before);
    }

    /// Tests that deposits succeed when properly approved
    ///
    /// This test verifies that:
    /// - A deposit can be made after approval is granted
    /// - The transaction returns true
    /// - The user's margin balance is updated correctly
    #[test]
    pub fn test_that_deposit_succeeds_when_approve_is_successful() {
        let pic = PocketIc::new();
        let (token_id, _, vault_id) = _setup_vault(&pic, 0);

        let account0 = _get_principals()[0];

        let args = TransferArg {
            from_subaccount: None,
            created_at_time: None,
            to: Account {
                owner: account0,
                subaccount: None,
            },
            amount: Nat::from(1000000000000000000u128),
            fee: None,
            memo: None,
        };
        //minting
        let _ = _icrc1_transfer(&pic, token_id, args, Principal::anonymous());

        let account1 = _get_principals()[1];

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
        let _ = _icrc2_approve(&pic, token_id, args, account0);

        let tx_result = _fund_account(&pic, vault_id, deposit_ampount, None, account1, account0);

        assert!(tx_result.is_ok_and(|val| { deposit_ampount == val }));

        let balance = _get_user_margin_balance(&pic, vault_id, account1);

        assert_eq!(balance, deposit_ampount);

        return;
    }
}
