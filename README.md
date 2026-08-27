# CHIP-8 Virtual Machine (Rust)

A fully functional emulator for the historical CHIP-8 architecture, written entirely in safe Rust. 

This project was built to gain hands-on experience with low-level computer architecture, including the fetch-decode-execute cycle, bitwise arithmetic, memory management, and hardware emulation.

## Features

* **Complete Instruction Set:** Implements all 34 original CHIP-8 opcodes.
* **Accurate Timing:** CPU executes at ~600Hz while hardware timers (Delay/Sound) strictly decrement at 60Hz.
* **Video Emulation:** Maps the 64x32 monochrome display buffer to a scalable desktop window using the `minifb` crate, handling XOR sprite rendering and collision detection.
* **Input Mapping:** Maps the original 16-key hexadecimal keypad to a modern QWERTY layout.
* **Memory Safe:** Leverages Rust's strict type system to prevent out-of-bounds memory access and safely mimic 8-bit overflow/underflow behavior.

## Build and Run

### Prerequisites
* [Rust and Cargo](https://rustup.rs/) installed.
* A `.ch8` ROM file (e.g., `Tetris.ch8`, `Pong.ch8`, `SpaceInvaders.ch8`). Note: ROMs are not included in this repository.

### Running a game
1. Clone the repository:
   ```bash
   git clone [https://github.com/yourusername/CHIP-8vm.git](https://github.com/yourusername/CHIP-8vm.git)
   cd CHIP-8vm
   ```
2. Place your chosen ROM file in the root directory (next to `Cargo.toml`).
3. Update the `load_rom()` path in `src/main.rs`:
   ```rust
   cpu.load_rom("YourGame.ch8"); 
   ```
4. Build and run:
   ```bash
   cargo run --release
   ```
   *(Running in `--release` mode ensures the CPU cycles execute at the proper speed).*

## Controls

The original hex pad is mapped to the left side of the keyboard:

| CHIP-8 | Modern Key |
| :--- | :--- |
| `1` `2` `3` `C` | `1` `2` `3` `4` |
| `4` `5` `6` `D` | `Q` `W` `E` `R` |
| `7` `8` `9` `E` | `A` `S` `D` `F` |
| `A` `0` `B` `F` | `Z` `X` `C` `V` |

Press `Escape` to close the emulator.

## Technical Architecture

The core of the VM is structured to mirror the physical hardware specifications:
* **RAM:** 4KB (`[u8; 4096]`)
* **Registers:** 16x 8-bit data registers (`V0-VF`) and 1x 16-bit address register (`I`)
* **Stack:** 16-level deep 16-bit array to handle nested subroutines.
* **Display:** 64x32 boolean array representing pixel states.
* **Cycle:** Reads 16-bit opcodes from memory, decodes them using bit masks and shifts, and executes the corresponding logic via a structured `match` block.