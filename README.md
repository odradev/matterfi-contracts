# MatterFi Casper Smart Contracts

Smart contracts using [Odra Framework](https://github.com/odradev/odra).
Note that the Odra is still under heavy development.

## Prepare
First install Rust, Make, WebAssembly Binary Toolkit (wabt), Perl and Git.

Install Cargo Odra.
```bash
$ git clone https://github.com/odradev/cargo-odra
$ cd cargo-odra
$ git fetch -a
$ git checkout release/0.0.1
$ make install
$ cargo odra -h
```

Add Rust target.
```bash
$ rustup target add wasm32-unknown-unknown
```

Add `wasm-strip`.
```bash
$ sudo apt install wabt
# or for Fedora
$ sudo dnf install wabt
```

Install Perl
```bash
$ sudo apt install perl
# or for Fedora
$ sudo dnf install perl-core
```

## Test on MockVM
```bash
$ cargo test
```

## Build WASM

```bash
$ cargo odra build -b casper
```

Check the `wasm` directory that was created for `*.wasm` files.

## Test on CasperVM
To build and test against CasperVM run: 

```bash
$ cargo odra test -b casper
```
