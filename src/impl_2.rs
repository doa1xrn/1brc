#[cfg(not(feature = "io_timing"))]
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::time::Instant;

#[cfg(not(feature = "io_timing"))]
struct WeatherStation {
    temps_sum: f32,
    num_temps: i32,
    min_temp: f32,
    max_temp: f32,
}

const BUFF_SIZE: usize = 0x4000; // 4K sweet spot on my system
const NEW_LINE: u8 = 0x0A;
const SEMICOLON: u8 = 0x3B;

pub fn run(file_path: &str) {
    println!("running impl 2 (replace lines() processing with custom buf processing)");

    let start = Instant::now();

    let f = File::open(file_path).unwrap();
    let mut br = BufReader::new(f);

    // prefer array over e.g. slice into vec for potential
    // optimisation benefits (I'm seeing about 2% improvement)
    let mut buf = [0u8; BUFF_SIZE];

    let mut num_bytes = br.read(buf.as_mut_slice()).unwrap();

    #[cfg(not(feature = "io_timing"))]
    let mut hash_map = HashMap::<String, WeatherStation>::new();

    #[cfg(feature = "io_timing")]
    let mut line_count = 0;

    let mut buf_pos = 0usize;
    let mut slice_start = 0usize;
    let mut slice_end = 0usize;
    let mut split_pos = 0usize;

    loop {
        let byte = buf[buf_pos];

        #[cfg(feature = "io_timing")]
        {
            if byte == NEW_LINE {
                line_count += 1;
            }
        }

        #[cfg(not(feature = "io_timing"))]
        {
            if byte == SEMICOLON {
                split_pos = buf_pos;
            } else if byte == NEW_LINE {
                let key = str::from_utf8(&buf[slice_start..split_pos]).unwrap();
                let temp_str = str::from_utf8(&buf[(split_pos + 1)..slice_end]).unwrap();
                let temp = temp_str.parse::<f32>().unwrap();

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
        }

        slice_end += 1;
        buf_pos += 1;

        if buf_pos == num_bytes {
            #[cfg(not(feature = "io_timing"))]
            {
                let num_partial = buf_pos - slice_start;
                if num_partial > 0 {
                    br.seek(SeekFrom::Current(-(num_partial as i64))).unwrap();
                }
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

    #[cfg(not(feature = "io_timing"))]
    for (_, ws) in hash_map.iter_mut() {
        ws.temps_sum = ws.temps_sum / ws.num_temps as f32;
    }

    let elapsed_ms = start.elapsed().as_millis();

    #[cfg(not(feature = "io_timing"))]
    {
        let mut v: Vec<(&String, &WeatherStation)> = hash_map.iter().collect();
        v.sort_by_key(|t| t.0);
        for (key, ws) in v {
            println!(
                "{}, {:.1}, {}, {}",
                key, ws.temps_sum, ws.min_temp, ws.max_temp
            )
        }
    }

    #[cfg(feature = "io_timing")]
    println!(
        "number of lines processed: {}",
        super::num_format::thousands(line_count)
    );

    if elapsed_ms < 1000 {
        println!("Execution time {}ms", elapsed_ms);
    } else if elapsed_ms < 10000 {
        println!("Execution time {:.1}s", start.elapsed().as_secs_f32());
    } else {
        println!("Execution time {}s", start.elapsed().as_secs());
    }
}
