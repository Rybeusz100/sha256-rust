use std::env;
use std::path::Path;

mod sha256;

fn main() {
    // check command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("No filenames supplied");
        return;
    }

    let filenames = &args[1..];
    
    // process each file
    for filename in filenames {
        let path = Path::new(filename);
        let hash = match sha256::hash_file(path, 1000) {
            Ok(hash) => hash,
            Err(why) => {
                println!("Couldn't process '{}', error message: {}", filename, why);
                continue;
            }
        };

        println!("{}: {}", filename, hash);
    }
}