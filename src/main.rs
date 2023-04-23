use clap::{App, Arg};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

mod sha256;

fn main() {
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
            Arg::with_name("threads")
                .short('t')
                .long("threads")
                .takes_value(true)
                .help("How many threads can the program spawn at once"),
        )
        .arg(
            Arg::with_name("files")
                .short('f')
                .long("files")
                .takes_value(true)
                .required(true)
                .min_values(1)
                .help("Files or directories separated by spaces"),
        )
        .get_matches();

    let threads = match matches.value_of("threads") {
        None => 4,
        Some(val) => match val.parse::<u8>() {
            Ok(n) if n > 0 => n,
            _ => 4,
        },
    };

    let buf_size_kb = (match matches.value_of("memory") {
        None => 1000,
        Some(val) => match val.parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => 1000,
        },
    }) / threads as usize;

    let filepaths: Vec<PathBuf> = matches
        .values_of("files")
        .unwrap()
        .map(PathBuf::from)
        .collect();
    let filepaths = Arc::new(Mutex::new(filepaths));

    let mut handles = Vec::new();

    for _ in 0..threads {
        let local_filenames = filepaths.clone();
        let handle = thread::spawn(move || loop {
            let path = local_filenames.lock().unwrap().pop();
            match &path {
                Some(path) => {
                    if path.is_dir() {
                        match fs::read_dir(path) {
                            Ok(children) => {
                                for child in children {
                                    local_filenames.lock().unwrap().push(child.unwrap().path());
                                }
                            }
                            Err(why) => {
                                println!(
                                    "Couldn't read directory '{}', error message: {}",
                                    path.display(),
                                    why
                                );
                            }
                        }
                        continue;
                    }

                    let hash = match sha256::hash_file(path, buf_size_kb) {
                        Ok(hash) => hash,
                        Err(why) => {
                            println!(
                                "Couldn't process '{}', error message: {}",
                                path.display(),
                                why
                            );
                            continue;
                        }
                    };

                    println!("{} {}", path.display(), hash);
                }
                None => break,
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}
