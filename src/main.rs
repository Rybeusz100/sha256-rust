use clap::{App, Arg};
use std::path::Path;

mod sha256;

fn main() {
    // process command line arguments
    let matches = App::new("SHA-256 calculator")
        .about("Calculates SHA-256 value for given files")
        .arg(
            Arg::with_name("memory")
                .short('m')
                .long("memory")
                .takes_value(true)
                .help("How much memory should the program use as a buffer (in kilobytes)"),
        )
        .arg(
            Arg::with_name("files")
                .short('f')
                .long("files")
                .takes_value(true)
                .required(true)
                .min_values(1)
                .help("Filenames separated by spaces"),
        )
        .get_matches();

    let buf_size_kb = match matches.value_of("memory") {
        None => 100,
        Some(val) => match val.parse::<u32>() {
            Ok(n) => {
                if n > 0 {
                    n
                } else {
                    100
                }
            }
            Err(_) => 100,
        },
    };

    let filenames: Vec<_> = matches.values_of("files").unwrap().collect();

    // process each file
    for filename in filenames {
        let path = Path::new(filename);
        let hash = match sha256::hash_file(path, buf_size_kb as usize) {
            Ok(hash) => hash,
            Err(why) => {
                println!("Couldn't process '{}', error message: {}", filename, why);
                continue;
            }
        };

        println!("{}: {}", filename, hash);
    }
}
