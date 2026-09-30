#[cfg(not(feature = "io_timing"))]
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

#[cfg(not(feature = "io_timing"))]
struct WeatherStation {
    temps_sum: f32,
    num_temps: i32,
    min_temp: f32,
    max_temp: f32,
}

pub fn run(file_path: &str) {
    println!("running impl 1 (basic implementation)");

    let start = Instant::now();

    let f = File::open(file_path).unwrap();
    let br = BufReader::new(f);
    let lines = br.lines();

    #[cfg(not(feature = "io_timing"))]
    let mut hash_map = HashMap::<String, WeatherStation>::new();

    #[cfg(feature = "io_timing")]
    let mut line_count = 0;

    for line in lines {
        #[cfg(feature = "io_timing")]
        {
            let _ = line.unwrap();
            line_count += 1;
        }

        #[cfg(not(feature = "io_timing"))]
        {
            let s = line.unwrap();
            let mut parts = s.split(';');
            let key = parts.next().unwrap();
            let temp = parts.next().unwrap().parse::<f32>().unwrap();
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
