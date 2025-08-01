# Utility script to build for WASM locally, to see changes to website in local
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/point_charge_simulation.wasm .