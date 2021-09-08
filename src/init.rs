use std::{
    fs::{self, File},
    path::PathBuf,
};

pub fn init_project(path: PathBuf) {
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

pub fn create_project(project_name: String, current_dir: PathBuf) {
    let mut project_path = current_dir;
    project_path.push(project_name);
    println!(
        "Initializing new project in {}",
        project_path.to_str().unwrap()
    );
    fs::create_dir(project_path.clone()).unwrap();
    init_project(project_path);
}
