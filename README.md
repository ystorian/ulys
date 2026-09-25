# <img src="https://raw.githubusercontent.com/ystorian/ulys/main/ulys.svg" width="48" align="absmiddle" alt="ULYS logo"> ULYS

![Build Status](https://github.com/ystorian/ulys/actions/workflows/ci-rust.yaml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/ulys.svg)](https://crates.io/crates/ulys)
[![docs.rs](https://docs.rs/ulys/badge.svg)](https://docs.rs/ulys)

This lib is inspired by the Rust implementation of the [`ulid`](https://github.com/ulid/spec)
project which provides _Universally Unique Lexicographically Sortable Identifiers_.

## Bit layout

A ULYS is a 128-bit [UUIDv8](https://www.rfc-editor.org/rfc/rfc9562#section-5.8). The diagram
shows the bits in big-endian order, 32 bits for each row.

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        timestamp (32)                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|        timestamp (16)         |  ver  |      random (12)      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|var|                        random (30)                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         checksum (32)                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- **`timestamp`** (bits 0 to 47): Unix time in milliseconds.
- **`ver`** (bits 48 to 51): the UUID version, `0b1000`.
- **`random`** (bits 52 to 63 and 66 to 95): 42 bits of random data.
- **`var`** (bits 64 and 65): the UUID variant, `0b10`.
- **`checksum`** (bits 96 to 127): the upper 32 bits of the `xxh3_64` hash of the 128-bit value.
  Before the hash, the checksum bits are set to zero.

The string form encodes these 128 bits from the first bit to the last bit. Each character holds
5 bits. The last character holds 3 bits and 2 zero padding bits.

The checksum includes the version and variant bits.

- `Ulys::is_valid()` checks only the checksum. A ULYS from an earlier version stays valid.
- `Ulys::is_uuidv8()` checks the version and variant bits.
- `Ulys::is_valid_time()` checks that the time is from 2020 to 2100, both included.


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

## Command-line interface

ULYS [releases](https://github.com/ystorian/ulys/releases) are signed and can be installed with
[cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall --only-signed ulys
```

To build from source, enable the `cli` feature:

```sh
cargo install ulys --features cli
```

Generate ULYS:

```sh
ulys                  # One ULYS.
ulys -n 5             # Five ULYS.
ulys --uuid           # One ULYS, shown as UUID.
ulys generate -n 5 -u # Same options with the explicit command.
```

Validate a ULYS or a UUID:

```console
$ ulys validate 06gdj3y3my1w93zfs456n8x6y4
ULYS:   06gdj3y3my1w93zfs456n8x6y4
UUID:   01a0d90f-c3a7-83c4-8fef-c90a6aa3a6f1
Valid:  yes
UUIDv8: yes
Time:   2026-09-25 14:54:44 UTC
```

- **`Valid`**: the checksum is correct, and the time is from 2020 to 2100 (both included). If not,
  the line shows the reasons.
- **`UUIDv8`**: the UUIDv8 version and variant bits are set.
- **`Time`**: the time that the ULYS contains, in UTC.

The exit code is `0` when all values are valid. It is `1` when a checksum or a time is not valid,
and `2` when a value is not a ULYS or a UUID.

## Crate features

All features are off by default.

- **`cli`**: builds the `ulys` binary (refer to [Command-line interface](#command-line-interface)).
  Also enables `uuid`.
- **`postgres`**: implements `ToSql` and `FromSql` from
  [`postgres-types`](https://docs.rs/postgres-types). A ULYS is stored in a PostgreSQL `uuid`
  column.
- **`serde`**: implements `Serialize` and `Deserialize`. A ULYS is serialized as its 26-character
  string. To use a different form on a field, use one of these modules with [`#[serde(with =
  "...")]`](https://serde.rs/field-attrs.html#with):
  - `ulys::serde::ulys_as_u128`: the `u128` value.
  - `ulys::serde::ulys_as_uuid`: the UUID string.
- **`uuid`**: implements `From<Uuid> for Ulys` and `From<Ulys> for Uuid` with the
  [`uuid`](https://docs.rs/uuid) crate. These conversions cannot fail.

## License

Licensed under either of

- [Apache License](LICENSE-APACHE), Version 2.0
- [MIT License](LICENSE-MIT)

at your option.

This crate is a fork of [`ulid-rs`](https://github.com/dylanhart/ulid-rs) by Dylan Hart, released
under the MIT License. The MIT copyright notice in [LICENSE-MIT](LICENSE-MIT) applies to the
original code.
