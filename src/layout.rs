use std::{fs, str::FromStr};
pub enum SourceFileType {
    Html,
    Md,
}

pub struct SourceFile {
    pub filetype: SourceFileType,
    pub path: String,
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
