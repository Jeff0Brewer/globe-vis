cargo clean
cargo build --target wasm32-unknown-unknown
mkdir -p ./generated
wasm-bindgen ./target/wasm32-unknown-unknown/debug/globe-vis.wasm --out-dir generated --target web
cp ./index.html ./generated
http-server ./generated -c-1
