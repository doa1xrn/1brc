use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::time::Instant;

struct WeatherStation {
    temps_sum: i32,
    num_temps: i32,
    min_temp: i32,
    max_temp: i32,
}

const BUFF_SIZE: usize = 0x4000; // 4K sweet spot on my system
const NEW_LINE: u8 = 0x0A;
const SEMICOLON: u8 = 0x3B;
const ASCII_ZERO: u8 = 0x30;
const ASCII_PERIOD: u8 = 0x2E;
const ASCII_MINUS: u8 = 0x2D;

fn parse_u8_slice_as_i32(sl: &[u8]) -> i32 {
    let mut sum = 0;
    let mut mult = 1;
    for b in sl.iter().rev() {
        match *b {
            ASCII_PERIOD => (),
            ASCII_MINUS => sum *= -1,
            _ => {
                sum += (b - ASCII_ZERO) as i32 * mult;
                mult *= 10;
            }
        }
    }
    sum
}

pub fn run(file_path: &str) {
    println!(
        "running impl 3 (String -> Vec, &str -> &[u8], and custom integer temperature parsing)"
    );

    let start = Instant::now();

    let f = File::open(file_path).unwrap();
    let mut br = BufReader::new(f);

    // prefer array over e.g. slice into vec for potential
    // optimisation benefits (I'm seeing about 2% improvement)
    let mut buf = [0u8; BUFF_SIZE];

    let mut num_bytes = br.read(buf.as_mut_slice()).unwrap();

    let mut hash_map = HashMap::<Vec<u8>, WeatherStation>::new();

    let mut buf_pos = 0usize;
    let mut slice_start = 0usize;
    let mut slice_end = 0usize;
    let mut split_pos = 0usize;

    loop {
        let byte = buf[buf_pos];

        if byte == SEMICOLON {
            split_pos = buf_pos;
        } else if byte == NEW_LINE {
            let key = &buf[slice_start..split_pos];
            let temp_slice = &buf[(split_pos + 1)..slice_end];
            let temp = parse_u8_slice_as_i32(temp_slice);

            if let Some(ws) = hash_map.get_mut(key) {
                ws.temps_sum += temp;
                ws.num_temps += 1;
                ws.min_temp = ws.min_temp.min(temp);
                ws.max_temp = ws.max_temp.max(temp);
            } else {
                hash_map.insert(
                    key.to_owned(),
                    WeatherStation {
                        temps_sum: temp,
                        num_temps: 1,
                        min_temp: temp,
                        max_temp: temp,
                    },
                );
            }
            slice_start = slice_end + 1;
        }

        slice_end += 1;
        buf_pos += 1;

        if buf_pos == num_bytes {
            let num_partial = buf_pos - slice_start;
            if num_partial > 0 {
                br.seek(SeekFrom::Current(-(num_partial as i64))).unwrap();
            }
            num_bytes = br.read(buf.as_mut_slice()).unwrap();
            if num_bytes == 0 {
                break;
            }
            buf_pos = 0;
            slice_start = 0;
            slice_end = 0;
        }
    }

    let mut v: Vec<(&Vec<u8>, f32, f32, f32)> = hash_map
        .iter()
        .map(|(key, v)| {
            (
                key,
                (v.temps_sum as f32 / v.num_temps as f32) / 10.0,
                v.min_temp as f32 / 10.0,
                v.max_temp as f32 / 10.0,
            )
        })
        .collect();

    let elapsed_ms = start.elapsed().as_millis();

    v.sort_by_key(|t| t.0);
    for (key, avg, min, max) in v {
        println!(
            "{}, {:.1}, {}, {}",
            str::from_utf8(key.as_ref()).unwrap(),
            avg,
            min,
            max
        )
    }

    if elapsed_ms < 1000 {
        println!("Execution time {}ms", elapsed_ms);
    } else if elapsed_ms < 10000 {
        println!("Execution time {:.1}s", start.elapsed().as_secs_f32());
    } else {
        println!("Execution time {}s", start.elapsed().as_secs());
    }
}
