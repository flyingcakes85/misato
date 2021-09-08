use handlebars::Handlebars;
use std::fs::{create_dir_all, metadata};
use std::path::PathBuf;
use std::{collections::BTreeMap, fs};
use walkdir::WalkDir;

use crate::build::layout_utils::detect_layout;

mod layout_utils;

pub fn build_project() {
    fs::create_dir::<PathBuf>([r".", "target"].iter().collect()).unwrap();
    // let folders: Vec<&str> = vec!["./assets", "./_pages", "./styles"];

    let html_pages_folder = "./_pages";

    for source_path in WalkDir::new(html_pages_folder) {
        // entry is of form ./_pages/index.html
        let source_path = source_path.unwrap();
        let mut dest_path: PathBuf = [r".", "target"].iter().collect();
        if metadata(source_path.path().to_str().unwrap())
            .unwrap()
            .is_file()
        {
            dest_path.push(&source_path.path().to_str().unwrap()[9..]);

            println!(
                "source_path: {}\n dest path: {}",
                source_path.path().to_str().unwrap(),
                dest_path.to_str().unwrap()
            );

            let dest_path_str = dest_path.to_str().unwrap();
            if dest_path_str[dest_path_str.len() - 4..] == *"html" {
                generate_from_html(source_path.path(), dest_path);
            } else {
                // only during testing
                // to be fixed
                println!(
                    "Not touching unknown file: {}",
                    source_path.path().to_str().unwrap()
                );
            }
        } else {
            dest_path.push(&source_path.path().to_str().unwrap()[8..]);
            create_dir_all(dest_path).unwrap();
        }
    }
}

fn generate_from_html(source_path: &std::path::Path, dest_path: PathBuf) {
    let layout: String;

    let source_file = layout_utils::SourceFile {
        filetype: layout_utils::SourceFileType::Html,
        path: source_path.to_str().unwrap().to_string(),
    };
    let layout_detected = detect_layout(source_file);

    layout = match layout_detected {
        Some(s) => s,
        None => panic!("Incorrect layout in {}", source_path.to_str().unwrap()),
    };

    let content = fs::read_to_string(source_path).expect("Could not read file");

    let mut layout_template_path: PathBuf = [r".", "_layouts"].iter().collect();
    layout_template_path.push(layout);
    layout_template_path.set_extension("html");
    println!(
        "layout template path : {}",
        layout_template_path.to_str().unwrap()
    );
    let layout_template = fs::read_to_string(layout_template_path).expect("error reading layout");
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(source_path.to_str().unwrap(), &layout_template)
        .unwrap();
    let mut data = BTreeMap::new();
    data.insert("content".to_string(), content);

    fs::write(
        dest_path,
        handlebars
            .render(source_path.to_str().unwrap(), &data)
            .unwrap(),
    )
    .unwrap();
}
