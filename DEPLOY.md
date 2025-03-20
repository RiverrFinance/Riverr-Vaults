## Deployment

```sh
dfx start 
```

```sh
export CONTROLLER=$(dfx identity get-principal)
export SUBNET=csyj4-zmann-ys6ge-3kzi6-onexi-obayx-2fvak-zersm-euci4-6pslt-lae
dfx deploy token --argument "(variant {Init = record {burn_fee = 0 ;decimals = opt 8;token_symbol = \"ICP\";transfer_fee = 0;metadata = vec {};minting_account = record { owner = principal \"2vxsx-fae\" ; subaccount = null};
initial_balances = vec {};archive_options = record {num_blocks_to_archive = 1000;trigger_threshold = 2000;controller_id = principal \"${CONTROLLER}\";
cycles_for_archive_creation = opt 10000000000000;};token_name = \"ICP\";feature_flags = opt record{icrc2 =true};transfer_fee_rate = 0;burn_fee_rate = 0;fee_collector_account = null}
})"  --network ic --subnet ${SUBNET}
```

Create Empty canisters for vault and vtoken

```sh
export INIT_CYCLES=2500000000000
dfx canister create vtoken --network ic --subnet ${SUBNET} --with-cycles ${INIT_CYCLES}
```

```sh
export INIT_CYCLES=3000000000000
dfx canister create vault --network ic --subnet ${SUBNET} --with-cycles ${INIT_CYCLES}
```

```sh
export CONTROLLER=$(dfx identity get-principal)
export VAULT=$(dfx canister id vault  --network ic)
export SUBNET=csyj4-zmann-ys6ge-3kzi6-onexi-obayx-2fvak-zersm-euci4-6pslt-lae
dfx deploy vtoken --argument "(variant {Init = record {burn_fee = 0 ;decimals = opt 8;token_symbol = \"QICP\";transfer_fee = 0;metadata = vec {};minting_account = record { owner = principal \"${VAULT}\" ; subaccount = null};
initial_balances = vec {};archive_options = record {num_blocks_to_archive = 1000;trigger_threshold = 2000;controller_id = principal \"${CONTROLLER}\";
cycles_for_archive_creation = opt 10000000000000;};token_name = \"Quotex ICP\";feature_flags = opt record{icrc2 =true};transfer_fee_rate = 0;burn_fee_rate = 0;fee_collector_account = null}
})"  --network ic --subnet ${SUBNET}
```

```sh

```

```sh
export TOKEN=$(dfx canister id token --network ic)
export VTOKEN=$(dfx canister id vtoken --network ic)
export MINAMOUNT=10000000

dfx deploy vault --argument "(record {asset = record { asset_type = variant  {ICRC} ; ledger_id = principal \"${TOKEN}\"};min_amount = ${MINAMOUNT};
virtual_asset = record { asset_type = variant {ICRC} ; ledger_id = principal \"${VTOKEN}\";}})" --network ic --subnet ${SUBNET}
```