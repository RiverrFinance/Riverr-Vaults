## Building Locally

 <p> to build project locally install all dependencies then run 

<p> to build the canister into wasm and extract the candid interface for deployemnt 

## Testing

 <p> This project is tested with PocketIC (current version 6.0.0) to setup Pocket IC check out this resource [here] (https://github.com/dfinity/pocketic). <p>
 <b>NOTE<b> :the token wasm needs to be downloaded and saved in the "target/wasm32-unknown-unknown/release/vault.wasm"

## Local Deployment

```sh
cargo build --release --target wasm32-unknown-unknown --package liquidity_manager

candid-extractor target/wasm32-unknown-unknown/release/liquidity_manager.wasm > src/liquidity_manager.did
```

```sh
dfx start 
```

### **Deploy test token**

```sh
export CONTROLLER=$(dfx identity get-principal)
##export SUBNET=csyj4-zmann-ys6ge-3kzi6-onexi-obayx-2fvak-zersm-euci4-6pslt-lae
dfx deploy token --argument "(variant {Init = record {burn_fee = 0 ;decimals = opt 8;token_symbol = \"ICP\";transfer_fee = 0;metadata = vec {};minting_account = record { owner = principal \"lmfrn-3iaaa-aaaaf-qaova-cai\" ; subaccount = null};
initial_balances = vec {};archive_options = record {num_blocks_to_archive = 1000;trigger_threshold = 2000;controller_id = principal \"${CONTROLLER}\";
cycles_for_archive_creation = opt 10000000000000;};token_name = \"ICP\";feature_flags = opt record{icrc2 =true};transfer_fee_rate = 0;burn_fee_rate = 0;fee_collector_account = null}
})"  # --network ic --subnet ${SUBNET}
```

### **Create Empty canisters for vtoken Liqudity Manager**

```sh
export INIT_CYCLES=2500000000000
dfx canister create vtoken   --with-cycles ${INIT_CYCLES} # --network ic --subnet ${SUBNET}
```

```sh
export INIT_CYCLES=3000000000000
dfx canister create vault   --with-cycles ${INIT_CYCLES}  #--network ic --subnet ${SUBNET}
```

### **Deploy Vtokens and Liquidity Manager canister**

```sh
export CONTROLLER=$(dfx identity get-principal)
export VAULT=$(dfx canister id vault)  ##--network ic)
# export SUBNET=csyj4-zmann-ys6ge-3kzi6-onexi-obayx-2fvak-zersm-euci4-6pslt-lae
dfx deploy vtoken --argument "(variant {Init = record {burn_fee = 0 ;decimals = opt 8;token_symbol = \"QICP\";transfer_fee = 0;metadata = vec {};minting_account = record { owner = principal \"${VAULT}\" ; subaccount = null};
initial_balances = vec {};archive_options = record {num_blocks_to_archive = 1000;trigger_threshold = 2000;controller_id = principal \"${CONTROLLER}\";
cycles_for_archive_creation = opt 10000000000000;};token_name = \"Quotex ICP\";feature_flags = opt record{icrc2 =true};transfer_fee_rate = 0;burn_fee_rate = 0;fee_collector_account = null}
})"  #--network ic --subnet ${SUBNET}
```

```sh

```

```sh
export TOKEN=$(dfx canister id token  --network ic)
export VTOKEN=$(dfx canister id vtoken --network ic)
export MINAMOUNT=10000000

dfx deploy liquidity_manager --argument "(record {asset = record { asset_type = variant  {ICRC} ; ledger_id = principal \"${TOKEN}\"};min_amount = ${MINAMOUNT};
virtual_asset = record { asset_type = variant {ICRC} ; ledger_id = principal \"${VTOKEN}\";}})" #--network ic 
```

```sh
dfx canister call liquidity_manager approveMarket "(principal \"i4w3l-hiaaa-aaaaf-qao5a-cai\")" --network ic 
```

```sh
dfx canister  uninstall-code liquidity_manager --network ic
```