## Building Locally
 <p> to build project locally install all dependencies then run 

```sh
cargo build --release --target wasm32-unknown-unknown --package vault
candid-extractor target/wasm32-unknown-unknown/release/vault.wasm > src/vault.did
```

<p> to build the canister into wasm and extract the candid interface for deployemnt 

## Testing 
 <p> This project is tested with PocketIC (current version 6.0.0) to setup Pocket IC check out this resource [here] (https://github.com/dfinity/pocketic). <p>
 <b>NOTE<b> :the token wasm needs to be downloaded and saved in the "target/wasm32-unknown-unknown/release/vault.wasm "