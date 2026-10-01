use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if (args.len() != 4)
        || (args[1] != "create" && args[1] != "impl")
        || (args[2].parse::<u64>().unwrap_or(0) == 0)
    {
        println!("Usage:");
        println!("  create <number of records> <file name>");
        println!("  impl <impl number> <file name>");
        println!("e.g.:");
        println!("  create 1000000 m_1m.data");
        println!("  impl 1 m_1m.data");

        process::exit(1);
    }

    let num = args[2].parse::<u64>().unwrap();
    let file_path = args[3].clone();

    match args[1].as_str() {
        "create" => one_brc::create_file::run(num, &file_path),
        "impl" => match num {
            1 => one_brc::impl_1::run(&file_path),
            2 => one_brc::impl_2::run(&file_path),
            3 => one_brc::impl_3::run(&file_path),
            4 => one_brc::impl_4::run(&file_path),
            _ => println!("'impl {num}' not available"),
        },
        _ => (),
    }
}
