Standalone Rust crate, edition 2021, rust-version 1.75, Apache-2.0. The
default native feature set has no runtime dependencies; the optional `wasm`
feature uses pinned `wasm-bindgen` bindings and Wasm tests. All dependencies
must preserve the offline/deterministic/no-model/no-network/no-RNG contract.
Build artifacts are under target/ and should remain ignored.
