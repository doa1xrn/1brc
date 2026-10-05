use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::thread;
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

fn thread_fn(
    file_path: String,
    file_offset: u64,
    chunk_size: u64,
) -> HashMap<Vec<u8>, WeatherStation> {
    let f = File::open(file_path).unwrap();

    let mut br = BufReader::new(f);

    // prefer array over e.g. slice into vec for potential
    // optimisation benefits (I'm seeing about 2% improvement)
    let mut buf = [0u8; BUFF_SIZE];

    br.seek(SeekFrom::Start(file_offset)).unwrap();

    let mut num_bytes = br.read(buf.as_mut_slice()).unwrap();

    let mut hash_map = HashMap::<Vec<u8>, WeatherStation>::new();

    let mut buf_pos = 0;
    // if not processing from start of file then move past partial line
    if file_offset != 0 {
        buf_pos = buf.iter().position(|b| *b == NEW_LINE).unwrap() + 1;
    }
    let mut slice_start = buf_pos;
    let mut slice_end = buf_pos;
    let mut split_pos = buf_pos;
    let mut proc_count = buf_pos;

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

            if proc_count >= chunk_size as usize {
                break;
            }

            slice_start = slice_end + 1;
        }

        slice_end += 1;
        buf_pos += 1;
        proc_count += 1;

        // end of buf reached?
        if buf_pos == num_bytes {
            // rewind to start of new line
            let num_partial = buf_pos - slice_start;
            if num_partial > 0 {
                br.seek(SeekFrom::Current(-(num_partial as i64))).unwrap();
                proc_count -= num_partial;
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

    hash_map
}

pub fn run(file_path: &str) {
    println!("running impl 5 (fully multi-threaded)");

    let num_threads = env::var("NUM_THREADS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| {
            println!("NUM_THREADS environment variable not specified, defaulting to 1");
            1
        });

    let start = Instant::now();

    let file_len = fs::metadata(file_path).unwrap().len();

    // launch threads
    let mut join_handles_vec = Vec::new();
    let mut chunk_size = file_len / num_threads;
    let mut file_offset = 0;
    for i in 0..num_threads {
        let file_path = file_path.to_owned();
        if i == (num_threads - 1) {
            chunk_size = file_len - (chunk_size * (num_threads - 1));
        }
        join_handles_vec.push(thread::spawn(move || {
            thread_fn(file_path, file_offset, chunk_size)
        }));
        file_offset += chunk_size;
    }

    // gather thread results
    let mut hash_maps_vec = Vec::new();
    join_handles_vec
        .into_iter()
        .for_each(|jh| hash_maps_vec.push(jh.join().unwrap()));

    // merge hash maps
    let mut first_hash_map = hash_maps_vec.remove(0);
    hash_maps_vec.into_iter().for_each(|hm| {
        hm.into_iter().for_each(|(key, add)| {
            first_hash_map
                .entry(key)
                .and_modify(|ws| {
                    ws.temps_sum += add.temps_sum;
                    ws.num_temps += add.num_temps;
                    ws.min_temp = ws.min_temp.min(add.min_temp);
                    ws.max_temp = ws.max_temp.max(add.max_temp);
                })
                .or_insert(add);
        });
    });

    // integer float conversion and average calc
    let mut hash_map_as_vec = first_hash_map
        .into_iter()
        .map(|(key, v)| {
            (
                key,
                (v.temps_sum as f32 / v.num_temps as f32) / 10.0,
                v.min_temp as f32 / 10.0,
                v.max_temp as f32 / 10.0,
            )
        })
        .collect::<Vec<(Vec<u8>, f32, f32, f32)>>();

    let elapsed_ms = start.elapsed().as_millis();

    // print results
    hash_map_as_vec.sort_by(|a, b| a.0.cmp(&b.0));
    for (key, avg, min, max) in hash_map_as_vec {
        println!(
            "{}, {:.1}, {}, {}",
            str::from_utf8(key.as_ref()).unwrap(),
            avg,
            min,
            max
        )
    }

    // print timing
    if elapsed_ms < 1000 {
        println!("Execution time {}ms", elapsed_ms);
    } else if elapsed_ms < 10000 {
        println!("Execution time {:.1}s", start.elapsed().as_secs_f32());
    } else {
        println!("Execution time {}s", start.elapsed().as_secs());
    }
}
