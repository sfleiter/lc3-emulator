# LC-3 Emulator

This project provides a 🦀 Rust implementation of an emulator for
the [Little Computer 3](https://en.wikipedia.org/wiki/Little_Computer_3).

## Goals

The main goals are:

- Deepen my understanding of Rust by implementing something low-level
- Learn an easy form of assembly not (only) by writing assembly code but writing an implementation for the LC-3 CPU and
  its opcodes

## Status

Implementation of opcodes and trap routines is still incomplete, see `todo!()` markers in [
`opcodes.rs`](https://github.com/sfleiter/lc3-emulator/blob/main/src/emulator/opcodes.rs)
and [`emulator/mod.rs`](https://github.com/sfleiter/lc3-emulator/blob/main/src/emulator/mod.rs)

## Usage

Compile and run as usual.

Minimum Supported Rust Version (MSRV): is `1.87.0`,
see [Rust Reference](https://doc.rust-lang.org/cargo/reference/rust-version.html#rust-version).

## Contributing

As this is a learning project for myself I do **not** plan to accept pull requests. If you see issues or have ideas how
to improve this project I am happy for every filed issue on that, though.

## Helpful links for understanding the LC-3

- [LC3 Tutor](http://lc3tutor.org/) - Provides information on all opcodes as well as the ability to load programs and
  step through them, showing the instructions and registers and providing a possibility to step though the instructions
- [Instruction Set Architecture (ISA)](https://www.jmeiners.com/lc3-vm/supplies/lc3-isa.pdf) - Main documentation of the
  system to implement
- [Disassembler](https://github.com/vastopol/disco) - See Shell Wrapper
  in [dis/disco](https://github.com/vastopol/disco/blob/master/dis/disco)
- [Sign Extension](https://en.wikipedia.org/wiki/Sign_extension) - Method of increasing number of bits of a number
  represented in [Two's complement](https://en.wikipedia.org/wiki/Two%27s_complement)

## Continuous Integration (CI)

- [GitHub Workflows](https://github.com/sfleiter/lc3-emulator/tree/main/.github/workflows) implement the following
  - For each commit
    - Build, run and test the code and fail on any errors
    - Verify the code with clippy according to config
      in [Cargo.toml](https://github.com/sfleiter/lc3-emulator/blob/main/Cargo.toml)
    - Check formatting agrees with `rustfmt` (default) configuration
    - Generate README.md from template
  - Daily
    - Audit dependencies for security issues
  - Weekly
    - Create Pull Requests for possible updates of all Rust dependencies and GitHub actions
    - [CodeQL](https://github.com/github/codeql?tab=readme-ov-file#codeql) scanning for GitHub Actions and code
      - For Rust code there is the warning `Low Rust analysis quality` generated which seems to be caused by the
        issue [20643](https://github.com/github/codeql/issues/20643) tracked in the CodeQL project
