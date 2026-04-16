# CHIP-8 Emulator — Rust

A cycle-accurate, fully custom CHIP-8 emulator implemented from scratch in Rust.
Built without external CPU orchestration libraries, simulating the complete hardware architecture of the original 1977 COSMAC VIP interpreter.

---

## Features

- Complete Fetch-Decode-Execute CPU cycle with a 4096-byte memory address space.
- 16 general-purpose 8-bit registers (V0–VF), 16-bit Index Register (I), and Program Counter (PC).
- 16-level Call Stack with Stack Pointer for subroutine management.
- Full ALU implementation: arithmetic, carry flags, bitwise logic, and shift operations.
- 64x32 VRAM matrix with XOR sprite rendering and hardware collision detection (VF register).
- Hexadecimal keypad mapped to QWERTY input via I/O port polling at 60 Hz.
- 60 Hz hardware-accurate Delay and Sound Timers decoupled from the CPU fetch cycle.
- System font ROM (80 bytes) preloaded in memory at boot (0x050), supporting BCD display.
- Mass memory dump and load (FX55/FX65) for register persistence.
- Hardware-accelerated 60 FPS window via `minifb` with 16x pixel scaling.
- Cryptographically seeded random number generator via the `rand` crate (CXNN).

---

## Requirements

- [Rust & Cargo](https://rustup.rs/)

---

## Build & Run

```bash
git clone https://github.com/MarianoJerso/chip8_emulator.git
cd chip8_emulator
cargo build --release
cargo run --release -- path/to/game.ch8
```

### Example
If you use the ROMs included in the submodule (recommended):
```bash
cargo run --release -- roms/games/Pong.ch8
```

---

## Games & ROMs

### Important: Avoid HTML files
If you download ROMs from GitHub, make sure to download the **Raw** file. A real `.ch8` binary usually weighs less than 1KB. If it weighs more than 100KB, it's likely an HTML page downloaded by mistake.

### How to get games (ROMs)

To avoiding downloading HTML files by error, use these `curl` commands to download the `.ch8` binaries directly:

- **Pong**:
  ```bash
  curl -L -o pong.ch8 https://github.com/kripod/chip8-roms/raw/master/games/Pong%20%28alt%29.ch8
  ```
- **Tetris**:
  ```bash
  curl -L -o tetris.ch8 https://github.com/kripod/chip8-roms/raw/master/games/Tetris%20%5BFran%20Dachille%2C%201991%5D.ch8
  ```
- **Space Invaders**:
  ```bash
  curl -L -o invaders.ch8 https://github.com/kripod/chip8-roms/raw/master/games/Space%20Invaders%20%5BDavid%20Winter%5D.ch8
  ```
- **Breakout**:
  ```bash
  curl -L -o breakout.ch8 https://github.com/kripod/chip8-roms/raw/master/games/Breakout%20%5BCarmelo%20Cortez%2C%201979%5D.ch8
  ```

### Recommended Repositories:
- [Kripod / Chip8-ROMs](https://github.com/kripod/chip8-roms)
- [Mirrors / Chip-8-ROMs](https://github.com/JohnEarnest/Chip-8/tree/master/roms)

---

---

## Control Mapping

The original CHIP-8 used a 16-key hexadecimal keypad (0–F), mapped as follows:

| CHIP-8 Keypad | QWERTY Keyboard |
|---|---|
| 1 2 3 C | 1 2 3 4 |
| 4 5 6 D | Q W E R |
| 7 8 9 E | A S D F |
| A 0 B F | Z X C V |

---

## Architecture

The emulator implements the complete hardware stack of the CHIP-8 platform:

| Component | Implementation |
|---|---|
| CPU | `Cpu` struct with full ISA (34 opcodes) |
| Memory | `[u8; 4096]` address space |
| Registers | `[u8; 16]` general purpose + `u16` Index Register |
| Stack | `[u16; 16]` + Stack Pointer for CALL/RET |
| VRAM | `[[u8; 64]; 32]` pixel matrix with XOR rendering |
| Input | `[bool; 16]` I/O port boolean array updated at 60 Hz |
| Timers | Hardware 60 Hz Delay and Sound counters |
| Display | `minifb` hardware-accelerated window at 16x scale |

---

## Technical Notes

- **Carry Flags**: overflow and underflow arithmetic uses Rust's native `.overflowing_add()` and `.overflowing_sub()` to populate the VF collision register without undefined behavior.
- **Sprite Flicker**: The XOR rendering model faithfully replicates the visual flickering of the original hardware, caused by the erase-then-redraw cycle required by the CHIP-8 architecture.
- **Timer Decoupling**: The CPU runs at approximately 600 Hz (10 ticks per frame), while hardware timers decrement once per frame at 60 Hz, replicating the independent quartz crystal oscillator of the original board.

---

## License

MIT License. See `LICENSE` for details.
