# rust-cmacros
[![Build Status](https://travis-ci.org/robertknight/rust-cmacros.png?branch=master)](https://travis-ci.org/robertknight/rust-cmacros)

Rust library to assist with parsing and translating #define
macro definitions from C header files
to corresponding Rust code for use with bindings
to external libraries.

## Intro

To use libraries with a C interface from Rust,
[rust-bindgen](https://github.com/crabtw/rust-bindgen) can be used
to generate Rust bindings automatically. It does not translate
constants or functions defined as macros in C headers to Rust
code however.

rust-cmacros is a simple and fairly dumb library which helps to
fill this gap.

It has two basic functions:

* `extract_macros()` parses the source of a C header file and extracts C macro definitions
* `generate_rust_src()` takes as input a set of extracted macros and a translator function
  and generates Rust code.

## Examples

```rust
cargo run --example parse_all_headers /usr/include
```

Parse all headers in `/usr/include` and print re-constructed #define statements

```rust
cargo run --example translate_macros /usr/include/sqlite3.h
```

Parse `/usr/include/sqlite3.h` and output translations of the macros using the default translation function.
In real usage, you would supply a translation function to skip or modify the translations
for non-trivial or unnecessary macros.

