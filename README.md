<h1 align="center">🗑  <code>async-drop-derive</code></h1>

`async-drop-derive` is probably the least-worst ad-hoc `AsyncDrop` implementation you've seen, as a [derive macro][rust-derive-macro].

[rust-derive-macro]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros

## Install

```console
cargo add async-drop-derive                      # by default, tokio is enabled
cargo add async-drop-derive --features async-std # use async-std
```

If you're editing `Cargo.toml` by hand:

```toml
[dependencies]
async-drop-derive = "0.1"
#async-drop-derive = { version = "0.1", features = [ "async-std" ] }
```

> **Warning**
> `async-drop-derive` does not allow using both `async-std` and `tokio` features at the same time (see [the FAQ below](#FAQ)).

## Quickstart

### [`tokio`][tokio]

```rust
TODO
```

To run the example in [`examples/tokio.rs`](./examples/tokio.rs):

```
cargo run --example tokio --features=tokio
```

### [`async-std`][async-std]

```rust
TODO
```

To run the example in [`examples/async-std/main.rs`](./examples/async-std/main.rs):

```console
cargo run --example async-std --features=async-std
```

## Supported environments

`async-drop-derive` works with the following environments:

| Name                              | Supported? |
|-----------------------------------|------------|
| Async w/ [`tokio`][tokio]         | ✅         |
| Async w/ [`async-std`][async-std] | ✅         |

[tokio]: https://crates.io/crates/tokio
[async-std]: https://crates.io/crates/async-std

## FAQ

### Why does `async-drop-derive` assume that I'm using *either* `async-std` or `tokio`

Because you probably are. If this is a problem for you, it *can* be changed, file an issue and let's chat about it.

## Development

To get started working on developing `situwatiion`, run the following [`just`][just] targets:

```console
just setup build
```

To check that your changes are fine, you'll probably want to run:

```console
just test
```

If you want to see the full list of targets available that you can run `just` without any arguments.

```console
just
```

There are a few useful targets like `just build-watch` which will continuously build the project thanks to [`cargo watch`][cargo-watch].

[just]: https://github.com/casey/just
[cargo-watch]: https://crates.io/crates/cargo-watch

## Contributing

Contributions are welcome! If you find a bug or an impovement that should be included in `async-drop-derive`, [create an issue](https://github.com/t3hmrman/async-drop-derive/issues) or open a pull request.
