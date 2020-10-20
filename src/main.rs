extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;
use std::path::Path;

const MEM_SIZE: usize = 8 * 1024 * 1024 * 4; // 4 MB
const ROM_MEM_BASE: usize = 0x200;
const FONT_BASE_ADDR: usize = 0x00;

const FONT_DATA: [u8; 16 * 5] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn main() -> Result<(), String> {
    // let rom = include_bytes!("../ibm.ch8");
    let rom = include_bytes!("../BC_test.ch8");
    // let rom = include_bytes!("../tetris.ch8");

    let mut memory = vec![0u8; MEM_SIZE];

    for i in 0..rom.len() {
        memory[i + ROM_MEM_BASE] = rom[i];
    }

    for i in 0..FONT_DATA.len() {
        memory[i + FONT_BASE_ADDR] = FONT_DATA[i];
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 64 * 16, 32 * 16)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    canvas.set_logical_size(64, 32);

    canvas.present();

    const COL_BLACK: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 0);
    const COL_WHITE: sdl2::pixels::Color = sdl2::pixels::Color::RGB(255, 255, 255);

    let mut pc: usize = ROM_MEM_BASE;
    let mut i_register: u16 = 0;
    let mut data_registers = [0u8; 16];

    let mut backbuffer = [0u8; 64 * 32];

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => {}
            }
        }

        let instruction = ((memory[pc] as u16) << 8) | memory[pc + 1] as u16;
        pc = pc + 2;

        let op = ((instruction >> 12) & 0xF) as u8;
        let x = ((instruction >> 8) & 0xF) as u8;
        let y = ((instruction >> 4) & 0xF) as u8;
        let n = ((instruction >> 0) & 0xF) as u8;
        let nn = ((instruction) & 0xFF) as u8;
        let nnn = (instruction) & 0xFFF;

        match op {
            0x0 if y == 0xE => {
                canvas.set_draw_color(COL_BLACK);
                canvas.clear();
                for i in backbuffer.iter_mut() {
                    *i = 0;
                }
            }
            0x1 => {
                pc = nnn as usize;
            }
            0x3 => {
                if data_registers[x as usize] == nn {
                    pc = pc + 2;
                }
            }
            0x4 => {
                if data_registers[x as usize] != nn {
                    pc = pc + 2;
                }
            }
            0x5 => {
                if data_registers[x as usize] == data_registers[y as usize] {
                    pc = pc + 2;
                }
            }
            0x6 => {
                data_registers[x as usize] = nn;
            }
            0x7 => {
                data_registers[x as usize] += nn;
            }
            0x8 => {
                let vy = data_registers[y as usize];
                let vx = data_registers[x as usize];
                match n {
                    1 => {
                        data_registers[x as usize] = vx | vy;
                    }
                    2 => {
                        data_registers[x as usize] = vx & vy;
                    }
                    3 => {
                        data_registers[x as usize] = vx ^ vy;
                    }
                    4 => {
                        data_registers[x as usize] = vx.wrapping_add(vy);
                        data_registers[15] = (data_registers[15] < vx) as u8;
                    }
                    5 => {
                        data_registers[x as usize] = vx.wrapping_sub(vy);
                        data_registers[15] = (vx > vy) as u8;
                    }
                    6 => {
                        data_registers[x as usize] = vx >> 1;
                        data_registers[15] = vx & 1;
                    }
                    7 => {
                        data_registers[x as usize] = vy.wrapping_sub(vx);
                        data_registers[15] = (vy > vx) as u8;
                    }
                    _ => {
                        data_registers[x as usize] = vx << 1; // shift vx rather than vy for s-chip compat?
                        data_registers[15] = (vx >> 7) & 1;
                    }
                }
            }
            0xA => {
                i_register = nnn;
            }
            0xD => {
                let base_x = (data_registers[x as usize] % 64) as usize;
                let base_y = (data_registers[y as usize] % 32) as usize;

                data_registers[15] = 0;

                for ypos in 0..(n as usize) {
                    let row = memory[i_register as usize + ypos as usize];
                    for xpos in 0..8 {
                        if base_x + xpos >= 64 {
                            break;
                        }
                        let display_addr = (base_x + xpos) + (base_y + ypos) * 64;
                        let old_val = backbuffer[display_addr];

                        backbuffer[display_addr] ^= (row >> (7 - xpos)) & 0x01;

                        if old_val == 0x1 && backbuffer[display_addr] == 0x0 {
                            data_registers[15] = 1;
                        }

                        canvas.set_draw_color(if backbuffer[display_addr] == 0x1 {
                            COL_WHITE
                        } else {
                            COL_BLACK
                        });

                        canvas.draw_point(sdl2::rect::Point::new(
                            (base_x + xpos) as i32,
                            (base_y + ypos) as i32,
                        ));
                    }
                    canvas.present();
                    if base_y + ypos >= 32 {
                        break;
                    }
                }
            }
            0xF if y == 1 && n == 0xE => {
                i_register += data_registers[x as usize] as u16;
            }
            0xF if y == 2 && n == 9 => {
                i_register = FONT_BASE_ADDR as u16 + data_registers[x as usize] as u16 * 5;
            }
            0xF if y == 3 && n == 3 => {
                let mut vx = data_registers[x as usize];
                for i in 0..3 {
                    memory[i_register as usize + (3 - i) - 1] = vx % 10;
                    vx /= 10;
                }
            }
            // store registers
            0xF if y == 5 && n == 5 => {
                for i in 0..=x {
                    memory[(i_register + i as u16) as usize] = data_registers[i as usize];
                }
                // disable for s-compat
                // i_register = i_register + x as u16 + 1;
            }
            // load registers
            0xF if y == 6 && n == 5 => {
                for i in 0..=x {
                    data_registers[i as usize] = memory[(i_register + i as u16) as usize];
                }
                // disable for s-compat
                // i_register = i_register + x as u16 + 1;
            }
            _ => {
                dbg!("Encountered unimplemented instruction");
                dbg!(pc, op, x, y, n, nnn, i_register);
                std::io::stdin().read_line(&mut String::new());
            }
        }
    }

    Ok(())
}
