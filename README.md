# Install Cargo Odra
First install Rust, Make and Git.

```bash
$ git clone https://github.com/odradev/cargo-odra
$ cd cargo-odra
$ git fetch -a
$ git checkout release/0.0.1
$ make install
$ cargo odra -h
```

# Test on MockVM
```bash
$ cargo test
```

# Test on CasperVM
```bash
$ cargo odra test -b casper
```

# Build WASM
| Test already should have built it, but if you want to rebuild use below line.

```bash
$ cargo odra build -b casper
```
