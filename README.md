# Simple Heartbeats Rust Wrappers

The `heartbeats-simple-rust` crate provides some abstractions over the
`heartbeats-simple-sys` crate, available at
[https://github.com/connorimes/heartbeats-simple-sys](https://github.com/connorimes/heartbeats-simple-sys).

## Dependencies

The `heartbeats-simple-rust` crate depends on the `heartbeats-simple-sys`
crate.

Additionally, you must have the `heartbeats-simple` libraries installed to the
system.

The latest `heartbeats-simple` C libraries can be found at
[https://github.com/connorimes/heartbeats-simple](https://github.com/connorimes/heartbeats-simple).

## Usage
Add `heartbeats-simple-rust` as a dependency in `Cargo.toml`:

```toml
[dependencies.heartbeats-simple-rust]
git = "https://github.com/connorimes/heartbeats-simple-rust.git"
```
