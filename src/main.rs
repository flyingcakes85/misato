use std::env;

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
                println!("[ERROR] Please provide a name for this project.")
            };
        } else if args[1] == "build" {
            init::build_project(String::from(env::current_dir().unwrap().to_str().unwrap()));
        } else {
            println!("[ERROR] Subcommand \"{}\" not recognized.", args[1]);
        }
    }
}
