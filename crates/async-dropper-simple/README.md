<h1 align="center">🗑  <code>async-dropper-simple</code></h1>

`async-dropper` is probably the least-worst ad-hoc `AsyncDrop` implementation you've seen, and it works in two ways:

- `async_dropper::simple` is stolen nearly verbatim from [this StackOverflow answer](https://stackoverflow.com/a/75584109) (thanks to [`paholg`](https://stackoverflow.com/users/2977291/paholg)!)
- `async_dropper::derive` provides a trait called `AsyncDrop` and corresponding [derive macro][rust-derive-macro], which try to use `Default` and `PartialEq` to determine when to async drop.

The code in this crate powers `async_dropper::simple`. See the `async_dropper` crate for more details.

## Feature flags

| Flag               | Description                                                                           |
|--------------------|---------------------------------------------------------------------------------------|
| `tokio`            | Use the [`tokio`][tokio] async runtime                                                |
| `async-std`        | use the [`async-std`][async-std] async runtime                                        |
| `no-default-bound` | Avoid the `Default` bound on your `T` by wrapping the interior data in an `Option<T>` |
