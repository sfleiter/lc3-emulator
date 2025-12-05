# LC-3 Emulator

This project provides a 🦀 Rust implementation of an emulator for
the [Little Computer 3](https://en.wikipedia.org/wiki/Little_Computer_3).

## Goals

The main goals are:

- Deepen my understanding of Rust by implementing something low-level
- Learn an easy form of assembly not (only) by writing assembly code but writing an implementation for the LC-3 CPU and
  its opcodes

## Status

[![Minimum Supported Rust Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fsfleiter%2Flc3-emulator%2Fmain%2FCargo.toml&query=%24%5B%22package%22%5D.rust-version&label=MSRV)](https://doc.rust-lang.org/cargo/reference/rust-version.html#rust-version)
[![Build Status](https://github.com/sfleiter/lc3-emulator/actions/workflows/rust.yml/badge.svg)](https://github.com/sfleiter/lc3-emulator/actions/workflows/rust.yml)
[![Audit Dependencies](https://github.com/sfleiter/lc3-emulator/actions/workflows/rust-audit-dependencies.yml/badge.svg)](https://github.com/sfleiter/lc3-emulator/actions/workflows/rust-audit-dependencies.yml)
[![codecov](https://codecov.io/github/sfleiter/lc3-emulator/graph/badge.svg?token=6Z6DRK6Q3I)](https://codecov.io/github/sfleiter/lc3-emulator)
[![GitHub License](https://img.shields.io/github/license/sfleiter/lc3-emulator)](https://github.com/sfleiter/lc3-emulator?tab=MIT-1-ov-file#readme)

See the [rustdoc documentation](https://sfleiter.github.io/lc3-emulator/),
f.e. on [trap routines](https://sfleiter.github.io/lc3-emulator/lc3_emulator/emulator/trap_routines/index.html)
or [opcodes](https://sfleiter.github.io/lc3-emulator/lc3_emulator/emulator/opcodes/index.html).

### Open Implementation tasks
- ☐ missing opcodes, see `todo!()` markers in  [`emulator/opcodes.rs`](https://github.com/sfleiter/lc3-emulator/blob/main/src/emulator/opcodes.rs)
- ☐ memory mapped IO

## Contributing

As this is a learning project for myself I do **not** plan to accept pull requests. If you see issues or have ideas how
to improve this project I am happy for every filed issue on that, though.

## Helpful links for working with the LC-3

<details open>
<summary>Educational information</summary>

- [CS-131 Textbook](https://cs131.info/Assembly/GettingStarted/) describing the LC3 assembly language
- [Introduction to Computing Systems: From Bits and Gates to C and Beyond, 2/e](https://highered.mheducation.com/sites/0072467509/) ([Book](https://highered.mheducation.com/sites/0072467509/information_center_view0/))
  - Educational slides
  - Appendix PDFs
    - A - [Instruction Set Architecture (ISA)](https://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf),
      also see [alternative version](https://www.jmeiners.com/lc3-vm/supplies/lc3-isa.pdf)
    - B - [From LC-3 to x86](https://highered.mheducation.com/sites/dl/free/0072467509/104652/pat67509_appb.pdf)
    - C - [Microarchitecture of the LC-3](https://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appc.pdf)
    - and [more](https://highered.mheducation.com/sites/0072467509/student_view0/appendices_a__b__c__d____e.html)
- [Hardware view on LC3](https://people.cs.georgetown.edu/~squier/Teaching/HardwareFundamentals/120-2013-CourseDocuments/Lec-7c-programmingIO.pdf)
- [Everything you need to know on Endianess](https://www.thecodedmessage.com/posts/endian_polymorphism/)
- [Sign Extension](https://en.wikipedia.org/wiki/Sign_extension) - Method of increasing number of bits of a number
  represented in [Two's complement](https://en.wikipedia.org/wiki/Two%27s_complement)

</details>

### Tools

<details>
<summary>Command Line (CLI)</summary>

- [assembler, converter and simulator](https://highered.mheducation.com/sites/0072467509/student_view0/lc-3_simulator.html) (C)
  with [instructions](https://highered.mheducation.com/sites/dl/free/0072467509/104652/LC3_unix.pdf)
  - [lc3_ensemble](https://github.com/endorpersand/lc3-ensemble) (Rust, lib for same purpose, should be easy to add a main())
- [lcc](https://highered.mheducation.com/sites/0072467509/student_view0/c_to_lc-3_compiler.html) - C to LC3 compiler
- [Disassembler](https://github.com/vastopol/disco) - See Shell Wrapper in [dis/disco](https://github.com/vastopol/disco/blob/master/dis/disco) (Python)

</details>

<details>
<summary>GUI</summary>

- Web [LC3 Tutor](http://lc3tutor.org/) - Provides examples and information on all opcodes as well as the ability to load
  programs and step through them, showing the instructions and registers as well as <ins>assembling obj files</ins>
- ⚡ Web [WebLC3](https://lc3.cs.umanitoba.ca/)
- ⚡ Desktop [LC3Tools](https://github.com/gt-cs2110/lc3tools) - Electron, Vue for frontend and Rust for assembly

⚡ Beware: These generate incompatible obj-files ⚡ not usable with this simulator ⚡

</details>

### Interesting LC3 ASM examples
- [2048 game](https://github.com/rpendleton/lc3-2048)
- [Roguelike tunnel generator](https://github.com/justinmeiners/lc3-rogue)

## Continuous Integration (CI)

<details>
<summary>GitHub workflows and other automation</summary>

- [GitHub Workflows](https://github.com/sfleiter/lc3-emulator/tree/main/.github/workflows) implement the following
  - For each commit
    - Build, run and test the code and fail on any errors
    - Verify the code with clippy according to config
      in [Cargo.toml](https://github.com/sfleiter/lc3-emulator/blob/main/Cargo.toml)
    - Check formatting agrees with `rustfmt` (default) configuration
  - Daily
    - Audit dependencies for security issues
  - Weekly
    - Create Pull Requests for possible updates of all Rust dependencies and GitHub actions
    - [CodeQL](https://github.com/github/codeql?tab=readme-ov-file#codeql) scanning for GitHub Actions and code
      - For Rust code there is the warning `Low Rust analysis quality` generated which seems to be caused by the
        issue [20643](https://github.com/github/codeql/issues/20643) tracked in the CodeQL project
  - Manual
    - Cross compilation to ppc64 and running tests for making sure this works for Big Endian systems  
</details>
