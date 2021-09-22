#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use std::env;

mod build;
mod init;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if args[1] == "init" {
            init::init_project(env::current_dir().unwrap());
        } else if args[1] == "new" {
            if args.len() > 2 {
                init::create_project(args[2].clone(), env::current_dir().unwrap());
            } else {
                eprintln!("[ERR] Please provide a name for this project.")
            };
        } else if args[1] == "build" {
            build::build();
        } else {
            eprintln!("[ERR] Subcommand \"{}\" not recognized.", args[1]);
        }
    }
}
