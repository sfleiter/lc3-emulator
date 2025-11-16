# LC-3 Emulator
This project provides a 🦀 Rust implementation of an emulator for the [Little Computer 3](https://en.wikipedia.org/wiki/Little_Computer_3).

## Goals
The main goals of it are:
- Deepen my understanding of Rust by implementing something low-level
- Learn an easy form of assembly not (only) by writing assembly code but writing an implementation for the LC-3 CPU and its opcodes

## Helpful links for understanding the LC-3
- [LC3 Tutor](http://lc3tutor.org/) - Provides information on all opcodes as well as the ability to load programs and step through them, showing the instructions and registers and providing a possibility to step though the instructions
- [Instruction Set Architecture (ISA)](https://www.jmeiners.com/lc3-vm/supplies/lc3-isa.pdf)
- [Disassembler](https://github.com/vastopol/disco) - See Shell Wrapper in [`dis/disco`](https://github.com/vastopol/disco/blob/master/dis/disco)
- [Sign Extension](https://en.wikipedia.org/wiki/Sign_extension) - Method of increasing number of bits of a number represented in [Two's complement](https://en.wikipedia.org/wiki/Two%27s_complement)
