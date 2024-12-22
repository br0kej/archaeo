# `archaeo` - Code and Binary Archaeology

`archaeo` is a CLI tool for extracting information from C/C++ source code and compiled 
binaries which is useful to conduct complexity and statistical analysis.

> `archaeo` is currently in development and only supports source code
> at the moment!

## Install/Setup

```bash
cargo install --git https://github.com:br0kej/archaeo.git
```

## Usage 

### Extract source code metrics from `dummy.cpp` and save to CSV
```bash
archaeo source --path test-data/dummy.cpp -o .
```

### Extract source code metrics from `test.c` and save to JSON 
```bash
archaeo source --path test-data/test.c --fmt json -o .
```

### Extract source code metrics for `test-data` directory, save to CSV and output into `my-test-dir`
```bash
archaeo source --path test-data/ -o my-test-dir
```

## Planned Features

- [x] Multi-file/Project level extraction of source code features
- [ ] Support for extracting source code line information from DWARF and PDB
- [ ] Support for merging source code metrics with those extracted from compiled binaries
- [ ] Support merging of source code metrics and corresponding decompiled code metrics

## Acknowledgements

Thanks for the folks who developed [`rust-code-analysis`](https://github.com/mozilla/rust-code-analysis)
for doing a hella' lot of heavy lifting.



