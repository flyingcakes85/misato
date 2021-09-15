#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use std::{
    fs::{self, create_dir_all, File},
    path::PathBuf,
};

/// function to initialize project directory
pub fn init_project(path: PathBuf) {
    // folders to create
    let folders: Vec<String> = vec![
        "assets".to_string(),
        "layouts".to_string(),
        "pages".to_string(),
        "posts".to_string(),
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
        vec!["pages".to_string(), "index.html".to_string()],
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

    let first_post = "+++
[info]
title = \"First Post\"
subtitle = \"Example first post\"

[data]
layout = \"post\"

[vars]
ssg = \"Misato\"
+++

This is my first post.

Made using {{{vars.ssg}}}
"
    .to_string();

    let layout_post = "<!DOCTYPE html>
<html lang=\"en\">

<head>
    <title>{{{info.title}}}</title>
</head>

<body>
    <small>
        <i>{{{info.subtitle}}}</i>
    </small>
    <br>
    {{{content}}}
</body>
</html>
"
    .to_string();

    let pages_index = "<!DOCTYPE html>
<html lang=\"en\">

<head>
    <title>Homepage</title>
</head>

<body>
    <h2>This is a homepage</h2>
</body>

</html>
"
    .to_string();

    let config_toml = "[website]
name = \"New Website\"
author = \"Your Name\"
description = \"A new Misato project\"

[config]
scss = [\"main.scss\"]

[globals]
"
    .to_string();

    let mut layout_post_path = path_base.clone();
    layout_post_path.push("layouts");
    layout_post_path.push("post.html");

    let mut config_path = path_base.clone();
    config_path.push("config.toml");

    let mut first_post_path = path_base.clone();
    first_post_path.push("posts");
    first_post_path.push("first_post.md");

    let mut pages_index_path = path_base.clone();
    pages_index_path.push("pages");
    pages_index_path.push("index.html");

    fs::write(layout_post_path, layout_post).unwrap();
    fs::write(first_post_path, first_post).unwrap();
    fs::write(pages_index_path, pages_index).unwrap();
    fs::write(config_path, config_toml).unwrap();
}

/// Creates a folder then calles init() in there
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
