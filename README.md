<h1 align="center">🗑  <code>async-dropper</code></h1>

`async-dropper` is probably the least-worst ad-hoc `AsyncDrop` implementation you've seen, and it works in two ways:

- `async_dropper::simple` is stolen nearly verbatim from [this StackOverflow answer](https://stackoverflow.com/a/75584109) (thanks to [`paholg`](https://stackoverflow.com/users/2977291/paholg)!)
- `async_dropper::derive` provides a trait called `AsyncDrop` and corresponding [derive macro][rust-derive-macro], which try to use `Default` and `PartialEq` to determine when to async drop.

The code in this crate was most directly inspired by [this StackOverflow thread on Async Drop](https://stackoverflow.com/questions/71541765/rust-async-drop) and many other conversations:

- [Async Drop? - Reddit](https://www.reddit.com/r/rust/comments/vckd9h/async_drop/)
- [Asynchronous Destructors - rust-lang.org](https://internals.rust-lang.org/t/asynchronous-destructors/11127)
- [Async Drop roadmap](https://rust-lang.github.io/async-fundamentals-initiative/roadmap/async_drop.html) (once this is done, this crate will be deprecated!)

[rust-derive-macro]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros

## Install

You **must** set features on this crate, as it works with async runtimes:

```console
cargo add async-dropper --features tokio     # use tokio
cargo add async-dropper --features async-std # use async-std
```

If you're editing `Cargo.toml` by hand, **choose one** of the following lines:

```toml
[dependencies]
#async-dropper = { version = "0.1", features = [ "tokio" ] }
#async-dropper = { version = "0.1", features = [ "async-std" ] }
```

> **Warning**
> `async-dropper` does not allow using both `async-std` and `tokio` features at the same time (see [the FAQ below](#FAQ)).

## Quickstart

### `async_dropper::simple`

To use the "simple" version which uses a wrapper struct (`AsyncDropper<T>`), see [`examples/async_drop_simple.rs`](./examples/async_drop_simple.rs):

```rust
use std::{
    result::Result,
    time::Duration,
};

use async_dropper_simple::{AsyncDrop, AsyncDropper};
use async_trait::async_trait;

// NOTE: this example is rooted in crates/async-dropper

/// This object will be async-dropped (which must be wrapped in AsyncDropper)
#[derive(Default)]
struct AsyncThing(String);

#[async_trait]
impl AsyncDrop for AsyncThing {
    async fn async_drop(&mut self) {
        eprintln!("async dropping [{}]!", self.0);
        tokio::time::sleep(Duration::from_secs(2)).await;
        eprintln!("dropped [{}]!", self.0);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let _example_obj = AsyncDropper::new(AsyncThing(String::from("test")));
        eprintln!("here comes the (async) drop");
        // drop will be triggered here, and it will take *however long it takes*
        // you could also call `drop(_example_obj)`
    }

    Ok(())
}

```

You can run the example and see the output:

```console
cargo run --example async-drop-simple --features=tokio
```

### `async_dropper::derive`

The derive macro is a novel (and possibly foolhardy) attempt to implement `AsyncDrop` without actually wrapping the existing struct.

`async_dropper::derive` uses `Default` and `PartialEq` to *check if the struct in question is equivalent to it's default*.

For this approach to work well your `T` should have cheap-to-create `Default`s, and comparing a default value to an existing value should meaningfully differ (and identify an object that is no longer in use). **Please think thoroughly about whether this model works for your use case**.

For an example, see [`examples/async_drop.rs`](./examples/async_drop.rs):

```rust
use std::{
    result::Result,
    time::Duration,
};

use async_dropper::derive::AsyncDrop;
use async_trait::async_trait;

/// This object will be async-dropped
///
/// Objects that are dropped *must* implement [Default] and [PartialEq]
/// (so make members optional, hide them behind Rc/Arc as necessary)
#[derive(Debug, Default, PartialEq, Eq, AsyncDrop)]
struct AsyncThing(String);

/// Implementation of [AsyncDrop] that specifies the actual behavior
#[async_trait]
impl AsyncDrop for AsyncThing {
    // simulated work during async_drop
    async fn async_drop(&mut self) -> Result<(), AsyncDropError> {
        eprintln!("async dropping [{}]!", self.0);
        tokio::time::sleep(Duration::from_secs(2)).await;
        eprintln!("dropped [{}]!", self.0);
        Ok(())
    }

    // This function serves to indicate when async_drop behavior should *not* be performed
    // (i.e., if Self::default == Self, a drop must have occurred, or does not need to)
    fn reset(&mut self) {
        self.0 = String::default();
    }

    // How long we can allow async drop behavior to block
    fn drop_timeout(&self) -> Duration {
        Duration::from_secs(5) // extended from default 3 seconds
    }

    // NOTE: below was not implemented since we want the default of DropFailAction::Continue
    // fn drop_fail_action(&self) -> DropFailAction;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let _example_obj = AsyncThing(String::from("test"));
        eprintln!("here comes the (async) drop");
        // drop will be triggered here
        // you could also call `drop(_example_obj)`
    }

    Ok(())
}
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

### What does `async_dropper::derive` cost?

There is waste introduced by `async_dropper::derive`, namely:

- One `Mutex`-protected `T::default()` instance of your type, that exists as long as the program runs
- One extra `T::default()` that is made of an individual `T` being dropped.

As a result, every `drop` you perform on a T will perform two drops -- one on a `T::default()` and another on *your* `T`, which has been *converted* to a `T::default` (via `reset(&mut self)`).

## Development

To get started working on developing `async-dropper`, run the following [`just`][just] targets:

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

## Releasing

From the top level of this repository:

```console
PUBLISH_CRATE=yes PKG=<crate name> just release <version>
```

For example, to create the next semver `patch` release for `async-dropper-simple`:

```console
PUBLISH_CRATE=yes PKG=async-dropper-simple just release patch
```

## Contributing

Contributions are welcome! If you find a bug or an impovement that should be included in `async-dropper`, [create an issue](https://github.com/t3hmrman/async-dropper/issues) or open a pull request.
