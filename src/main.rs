use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};

fn create_project(path: PathBuf) {
    // println!("{:?}", path);
    let path_string = String::from(path.to_str().unwrap());

    let folders: Vec<&str> = vec![
        "assets", "_layouts", "_modules", "_pages", "_posts", "styles",
    ];
    for folder in folders {
        fs::create_dir(format!("{}/{}", path_string, folder)).unwrap();
    }

    let files: Vec<&str> = vec!["_pages/index.html", "styles/base.css", "config.toml"];
    for file in files {
        File::create(format!("{}/{}", path_string, file)).unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);
    if args.len() > 1 {
        if args[1] == "init" {
            create_project(env::current_dir().unwrap());
        } else if args[1] == "new" {
            if args.len() > 2 {
                let mut project_path = env::current_dir().unwrap();
                project_path.push(args[2].clone());
                println!(
                    "Initializing new project in {}",
                    project_path.to_str().unwrap()
                );
                fs::create_dir(project_path.clone()).unwrap();
                create_project(project_path);
            } else {
                println!("[ERROR] Please provide a name for this project.")
            };
        } else {
            println!("[ERROR] Subcommand \"{}\" not recognized.", args[1]);
        }
    }
}
