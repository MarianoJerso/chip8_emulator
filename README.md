# 🕹️ CHIP-8 Rust Emulator

A cycle-accurate, custom-built CHIP-8 Emulator programmed entirely from scratch in Rust. This repository demonstrates low-level systems programming, hardware reverse-engineering, and computer architecture principles.

## ✨ Features
- **Accurate CPU Architecture**: Complete simulation of Memory Address Space (4096 bytes), 16 8-bit Data Registers (V0-VF), 16-level memory Stack pointer, and the 16-bit Index Register (I).
- **ALU (Arithmetic Logic Unit)**: Exact implementation of the CHIP-8 logic engine (Opcode 8 family), natively handling overflow/underflow carry flags (VF).
- **Native Graphical Rendering**: 64x32 monochrome hardware-accelerated display matrix bound to a strict 60 FPS hardware loop powered by `minifb`.
- **Keyboard Engine**: Mathematical mapping of the original 1970s 16-key Hexadecimal pad to standard QWERTY boolean input arrays.
- **Mechanical Timers**: Precise 60Hz Delay and Sound timers uncoupled from the CPU fetch cycle.
- **Memory Routing**: Extensible mass memory-dumping and binary loading (FX55/FX65) implemented in the internal state machine.

## 🚀 Getting Started

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (The compiler and package manager)

### Compilation
Clone this repository to your local machine and compile the robust executable:
```bash
git clone https://github.com/YourUsername/chip8-emulator.git
cd chip8-emulator
cargo build --release
```

### Running a Cartridge
To play, you will need a `.ch8` or `.rom` cartridge file (such as Pong, Tetris, or Space Invaders) which can be legally legally downloaded from public Domain sources.

Run the emulator passing the path to the game as a terminal argument:
```bash
cargo run --release -- roms/pong.ch8
```

## 🎮 Control Mapping
The CHIP-8 originally used a 16-key hexadecimal keypad (0-F). It is fully mapped to the left-side of your QWERTY keyboard as follows:

| Original CHIP-8 | QWERTY Mapping |
|---------|---------|
| 1 2 3 C | 1 2 3 4 |
| 4 5 6 D | Q W E R |
| 7 8 9 E | A S D F |
| A 0 B F | Z X C V |

## 🛠️ Technical Architecture Details
This emulator does not rely on third-party orchestration logic. The entire `Fetch-Decode-Execute` cycle was constructed from ground zero:
1. **Fetch**: Reads 2 consecutive physical bytes from the `u8` memory vector layout depending on the `Program Counter`.
2. **Decode**: Merges both bits into a contiguous 16-bit Opcode and slices hexadecimal variables using rigorous Bitwise `AND/OR` masking logic (`& 0x0F00 >> 8`).
3. **Execute**: Branches into the concrete Hardware emulation subroutines by leveraging Rust's impenetrable `match` exhaustive pattern system over all 34 historical Opcodes.
