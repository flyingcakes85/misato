use std::{ffi::OsStr, fs, io, path::PathBuf, vec};

pub enum SourceFileType {
    Html,
    Md,
}

pub struct SourceFile {
    pub filetype: SourceFileType,
    pub path: String,
}

pub fn available_layouts() ->  io::Result<Vec<PathBuf>>{

    let mut layouts = vec![];

    for path in fs::read_dir("./_layouts/")? {
        let path = path?.path();
        if let Some("html") = path.extension().and_then(OsStr::to_str) {
            layouts.push(path.to_owned());
        }
    }
    Ok(layouts)
}

pub fn names_from_path(paths: Vec<PathBuf>, ext_len: usize) -> Vec<String>{
    let mut layout_list : Vec<String> = vec![];
    let mut path: String;

    for p in paths{
        path = p.display().to_string();
        layout_list.push(String::from(&path[11..(path.len()-ext_len)]));
    }

    layout_list
}

pub fn detect_layout(source_file: SourceFile) {
    let mut layout: &str = "";
    let contents = fs::read_to_string("te.html").expect("Something went wrong reading the file");

    for line in contents.lines() {
        match source_file.filetype {
            SourceFileType::Html => {
                if line.contains("<!-- layout:") {
                    layout = &line[13..(line.len() - 4)];
                }
            }
            SourceFileType::Md => {}
        }
    }

    println!("{}", layout);
}
