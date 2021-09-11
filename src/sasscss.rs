use rsass::{compile_scss_path, output};
use std::str;
fn main() {
    let path = "./style.scss".as_ref();
    let format = output::Format {
        style: output::Style::Compressed,
        ..Default::default()
    };
    let css = compile_scss_path(path, format).unwrap();

    std::fs::write("./style.css", &css);

} 
