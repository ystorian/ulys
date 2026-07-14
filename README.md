# ulys

![Build Status](https://github.com/ystorian/ulys/actions/workflows/ci-rust.yaml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/ulys.svg)](https://crates.io/crates/ulys)
[![docs.rs](https://docs.rs/ulys/badge.svg)](https://docs.rs/ulys)

This lib is inspired from the Rust implementation of the [ulid](https://github.com/ulid/spec)project which provides Universally Unique Lexicographically Sortable Identifiers.


## Quickstart

```rust
use ulys::Ulys;

// Generate a ulys
let ulys = Ulys::new();

// Generate a string for a ulys
let s = ulys.to_string();

// Create from a String
let res = Ulys::from_string(&s);

assert_eq!(ulys, res.unwrap());
```

## Crate Features

- **`serde`**: Enables serialization and deserialization of `Ulys` types via `serde`. ULYSs are serialized using their canonical 26-character representation as defined in the ULID standard. An optional `ulys_as_u128` module is provided, which enables serialization through an `Ulys`'s inner `u128` primitive type. See the [documentation](https://docs.rs/ulys/latest/ulys/serde/index.html) and [serde docs](https://serde.rs/field-attrs.html#with) for more information.
- **`uuid`**: Implements infallible conversions between ULYSs and UUIDs from the [`uuid`](https://github.com/uuid-rs/uuid) crate via the [`std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html) trait.
