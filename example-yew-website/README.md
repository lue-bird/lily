develop by installing
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
```
then running `trunk serve`.
If no lsp is running, run `lily build` whenever you want to rebuild.

Build with `trunk build --release`.

  - https://github.com/thedodd/trunk
