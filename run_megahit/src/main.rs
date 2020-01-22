extern crate run_megahit;
use std::process;

fn main() {
    let config = match run_megahit::get_args() {
        Ok(c) => c,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = run_megahit::run(config) {
        println!("Error: {}", e);
        process::exit(1);
    }
}
