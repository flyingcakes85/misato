use std::{
    fs::{self, create_dir_all, File},
    path::PathBuf,
};

/// function to initialize project directory
pub fn init_project(path: PathBuf) {
    // folders to create
    let folders: Vec<String> = vec![
        "assets".to_string(),
        "_layouts".to_string(),
        "_modules".to_string(),
        "_pages".to_string(),
        "_posts".to_string(),
        "styles".to_string(),
    ];

    let path_base: PathBuf = path;

    // create those folders
    for folder in folders {
        let mut path_new = path_base.clone();
        path_new.push(folder);

        create_dir_all(path_new).unwrap();
    }

    // files to create
    let files: Vec<Vec<String>> = vec![
        vec!["_pages".to_string(), "index.html".to_string()],
        vec!["styles".to_string(), "base.css".to_string()],
        vec!["config.toml".to_string()],
    ];

    // create those files
    for file in files {
        let mut path_new = path_base.clone();
        for f in file {
            path_new.push(f);
        }

        File::create(path_new).unwrap();
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
