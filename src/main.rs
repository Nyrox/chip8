extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;
use std::path::Path;

const MEM_SIZE: usize = 8 * 1024 * 1024 * 4; // 4 MB
const ROM_MEM_BASE: usize = 0x200;

fn main() -> Result<(), String> {
    let rom = include_bytes!("../ibm_logo.ch8");

    let mut memory = vec![0u8; MEM_SIZE];

    for i in 0..rom.len() {
        memory[i + ROM_MEM_BASE] = rom[i];
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
        dbg!(pc, op, nnn, i_register);

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
            0xA => {
                i_register = nnn;
            }
            0x6 => {
                data_registers[x as usize] = nn;
            }
            0x7 => {
                data_registers[x as usize] += nn;
            }
            0xD => {
                let base_x = (data_registers[x as usize] % 64) as usize;
                let base_y = (data_registers[y as usize] % 32) as usize;

				data_registers[15] = 0;

                for y in 0..(n as usize) {
					let row = memory[i_register as usize + y as usize];
                    for x in 0..8 {
                        if base_x + x >= 64 {
                            break;
                        }
                        let display_addr = (base_x + x) + (base_y + y) * 32;
						let old_val = backbuffer[display_addr];

                        backbuffer[display_addr] ^= (row >> (7 - x)) & 0x01;

						if old_val == 0x1 && backbuffer[display_addr] == 0x0 {
							data_registers[15] = 1;
						}

                        canvas.set_draw_color(if backbuffer[display_addr] == 0x1 {
                            COL_WHITE
                        } else {
                            COL_BLACK
                        });

                        canvas.draw_point(sdl2::rect::Point::new(
                            (base_x + x) as i32,
                            (base_y + y) as i32,
                        ));
                    }
                    canvas.present();
					if base_y + y >= 32 {
						break;
					}
                }
            }
            _ => {
                std::io::stdin().read_line(&mut String::new());
            }
        }
    }

    Ok(())
}
