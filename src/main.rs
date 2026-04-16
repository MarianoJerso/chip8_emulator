mod cpu;
use cpu::Cpu;
use minifb::{Key, Window, WindowOptions, Scale};
use std::env;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("ERROR: No ROM file specified.");
        eprintln!("Usage: cargo run -- <path/to/game.ch8>");
        return;
    }

    let rom_path = &args[1];
    println!("CHIP-8 Emulator starting...");
    println!("Loading ROM: {}", rom_path);

    let mut cpu = Cpu::new();

    let rom = std::fs::read(rom_path).unwrap_or_else(|e| {
        panic!("Failed to read ROM file: {}", e);
    });

    // Simple validation for common "wrong download" error (HTML instead of binary)
    if rom.starts_with(b"<!DOC") || rom.starts_with(b"<html") {
        eprintln!("ERROR: The file '{}' appears to be an HTML page, not a valid CHIP-8 ROM.", rom_path);
        eprintln!("Suggestion: Download the 'Raw' version of the file from GitHub.");
        return;
    }

    cpu.load_rom(&rom);
    println!("ROM loaded successfully ({} bytes).", rom.len());

    let mut window = Window::new(
        "CHIP-8 Emulator — Rust",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("Failed to create window: {}", e);
    });

    window.set_target_fps(60);

    let mut display_buffer: Vec<u32> = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {

        // --- Input: map QWERTY keyboard to the CHIP-8 hexadecimal keypad ---
        cpu.keypad[0x1] = window.is_key_down(Key::Key1);
        cpu.keypad[0x2] = window.is_key_down(Key::Key2);
        cpu.keypad[0x3] = window.is_key_down(Key::Key3);
        cpu.keypad[0xC] = window.is_key_down(Key::Key4);

        cpu.keypad[0x4] = window.is_key_down(Key::Q);
        cpu.keypad[0x5] = window.is_key_down(Key::W);
        cpu.keypad[0x6] = window.is_key_down(Key::E);
        cpu.keypad[0xD] = window.is_key_down(Key::R);

        cpu.keypad[0x7] = window.is_key_down(Key::A);
        cpu.keypad[0x8] = window.is_key_down(Key::S);
        cpu.keypad[0x9] = window.is_key_down(Key::D);
        cpu.keypad[0xE] = window.is_key_down(Key::F);

        cpu.keypad[0xA] = window.is_key_down(Key::Z);
        cpu.keypad[0x0] = window.is_key_down(Key::X);
        cpu.keypad[0xB] = window.is_key_down(Key::C);
        cpu.keypad[0xF] = window.is_key_down(Key::V);

        // --- CPU: execute ~600 Hz (10 ticks per 60 Hz display frame) ---
        for _ in 0..10 {
            cpu.tick();
        }

        // --- Timers: decrement hardware counters once per frame at 60 Hz ---
        cpu.tick_timers();

        // --- Render: translate VRAM pixel matrix into flat RGB u32 buffer ---
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                display_buffer[row * DISPLAY_WIDTH + col] = if cpu.display[row][col] == 1 {
                    0x00FFFFFF // White
                } else {
                    0x00000000 // Black
                };
            }
        }

        window.update_with_buffer(&display_buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT).unwrap();
    }
}
