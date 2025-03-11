```bash
cargo build --release --target wasm32-unknown-unknown --package vault
candid-extractor target/wasm32-unknown-unknown/release/vault.wasm > src/vault.did
```
