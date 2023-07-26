<h1 align="center">🗑  <code>async-dropper</code></h1>

`async-dropper` is probably the least-worst ad-hoc `AsyncDrop` implementation you've seen, as a trait called `AsyncDrop` and corresponding [derive macro][rust-derive-macro].

The code in this crate was most directly inspired by [this StackOverflow thread on Async Drop](https://stackoverflow.com/questions/71541765/rust-async-drop) and many other conversations:

- [Async Drop? - Reddit](https://www.reddit.com/r/rust/comments/vckd9h/async_drop/)
- [Asynchronous Destructors - rust-lang.org](https://internals.rust-lang.org/t/asynchronous-destructors/11127)
- [Async Drop roadmap](https://rust-lang.github.io/async-fundamentals-initiative/roadmap/async_drop.html) (once this is done, this crate will be deprecated!)

[rust-derive-macro]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros

## Install

```console
cargo add async-dropper                      # by default, tokio is enabled
cargo add async-dropper --features async-std # use async-std
```

If you're editing `Cargo.toml` by hand:

```toml
[dependencies]
async-dropper = "0.1"
#async-dropper = { version = "0.1", features = [ "async-std" ] }
```

> **Warning**
> `async-dropper` does not allow using both `async-std` and `tokio` features at the same time (see [the FAQ below](#FAQ)).

## Quickstart

### `async_dropper::simple`

To use the "simple" version which uses a wrapper struct (`AsyncDropper<T>`), see [`examples/async_drop_simple.rs`](./examples/async_drop_simple.rs):

```rust
```

You can run the example and see the output:

```console
cargo run --example async-drop-simple --features=tokio
```

### `async_dropper::derive`

To use the complex (and possibly *worse*) version which is a derive macro that tries to do everything for you, see [`examples/async_drop.rs`](./examples/async_drop.rs):

```rust
```

You can run the example and see the output:

```console
cargo run --example async-drop --features=tokio
```

## Supported environments

`async-dropper` works with the following async environments:

| Name                              | Supported? |
|-----------------------------------|------------|
| Async w/ [`tokio`][tokio]         | ✅         |
| Async w/ [`async-std`][async-std] | ✅         |

[tokio]: https://crates.io/crates/tokio
[async-std]: https://crates.io/crates/async-std

## FAQ

### Why does `async-dropper` assume that I'm using *either* `async-std` or `tokio`

Because you probably are. If this is a problem for you, it *can* be changed, please file an issue.

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

Contributions are welcome! If you find a bug or an impovement that should be included in `async-dropper`, [create an issue](https://github.com/t3hmrman/async-dropper/issues) or open a pull request.
