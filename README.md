# ulys

![Build Status](https://github.com/ystorian/ulys/actions/workflows/ci-build.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/ulys.svg)](https://crates.io/crates/ulys)
[![docs.rs](https://docs.rs/ulys/badge.svg)](https://docs.rs/ulys)

This lib is inspired from the Rust implementation of the [ulid][ulid] project which provides Universally Unique Lexicographically Sortable Identifiers.

[ulid]: https://github.com/ulid/spec

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

* **`serde`**: Enables serialization and deserialization of `Ulys` types via `serde`. ULYSs are serialized using their canonical 26-character representation as defined in the ULID standard. An optional `ulys_as_u128` module is provided, which enables serialization through an `Ulys`'s inner `u128` primitive type. See the [documentation][serde_mod] and [serde docs][serde_docs] for more information.
* **`uuid`**: Implements infallible conversions between ULYSs and UUIDs from the [`uuid`][uuid] crate via the [`std::convert::From`][trait_from] trait.

[serde_mod]: https://docs.rs/ulys/latest/ulys/serde/index.html
[serde_docs]: https://serde.rs/field-attrs.html#with
[uuid]: https://github.com/uuid-rs/uuid
[trait_from]: https://doc.rust-lang.org/std/convert/trait.From.html
