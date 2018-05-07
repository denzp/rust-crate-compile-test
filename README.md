# Rust Crate Compilation Test helper
> Swiss army knife for `proc-macro` crates testing.

[![Build Status](https://travis-ci.org/denzp/rust-crate-compile-test.svg?branch=master)](https://travis-ci.org/denzp/rust-crate-compile-test)
[![Current Version](https://img.shields.io/crates/v/crate-compile-test.svg)](https://crates.io/crates/crate-compile-test)

## Purpose
The library was highly inspired by [laumann/compiletest-rs](https://github.com/laumann/compiletest-rs), and it's origin [Rust's compiletest](https://github.com/rust-lang/rust/tree/master/src/tools/compiletest).
Difference between them and this library is that latter lets to test whole crates instead of single compilation units.

This can be useful if your `proc-macro` uses **cargo** (or **xargo**) or you want to test more complex scenarios.

**There is a lot of work** needs to be done, to get feature parity with the other Rust compilation testing libs, currently planned only:

* Successful Compilation tests
* Failed Compilation tests
* Macro Expansion tests

## Installation
No third party tools are needed. Just add the library to `dev-dependencies`:

```
[dev-dependencies]
crate-compile-test = "0.1"
```

## Usage
The example usage can be found at `example` directory.

### Failed Compilation messages
Expected messages specification is similar to original [compiletest's specification](https://github.com/rust-lang/rust/blob/master/src/test/COMPILER_TESTS.md#summary-of-error-info-commands), with small addition - you can specify either **error code** or **error message**:

``` rust
use mod2::func3; //~ ERROR unresolved import `mod2::func3`

fn func2() -> NonExistingType {
    0
}
//~^^^ ERROR E0433
```

### Macro Expansion
TBD