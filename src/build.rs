use crate::build::layout_utils::detect_layout;
use comrak::{markdown_to_html, ComrakOptions};
use extract_frontmatter::Extractor;
use handlebars::Handlebars;
use rsass::{compile_scss_path, output};
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    fs::create_dir_all,
    path::{Path, PathBuf},
    str,
};
use toml::Value;
use walkdir::WalkDir;

mod layout_utils;

#[cfg(test)]
mod tests {
    use crate::build::_scss_to_css;
    use crate::build::generate_from_html;
    use std::{
        fs::{self, create_dir_all},
        path::PathBuf,
        str,
    };
    #[test]
    fn check_generation_from_html() {
        let base_path: PathBuf = [r".", "test_cache", "generation-from-html-test"]
            .iter()
            .collect();
        create_dir_all(&base_path).unwrap();

        let mut source_path = base_path.clone();
        source_path.push("source");
        source_path.set_extension("html");

        let mut dest_path = base_path.clone();
        dest_path.push("dest");
        dest_path.set_extension("html");

        let mut layout_path = base_path.clone();
        layout_path.push("_layout");
        layout_path.push("test");
        layout_path.set_extension("html");

        let mut layout_folder = base_path;
        layout_folder.push("_layout");

        create_dir_all(&layout_folder).unwrap();
        // @TODO : Separate test files from code
        let layout_html_code = "
<!DOCTYPE html>
<html>
<body>
Static text
<br>
{{{content}}}
</body>
</html>
";

        let source_code = "
<!-- layout: test -->

Example text
<a href=\"https://www.rust-lang.org/\">Rust Website</a>
";

        let dest_code = "
<!DOCTYPE html>
<html>
<body>
Static text
<br>

<!-- layout: test -->

Example text
<a href=\"https://www.rust-lang.org/\">Rust Website</a>

</body>
</html>
";
        fs::write(layout_path, layout_html_code).unwrap();
        fs::write(&source_path, source_code).unwrap();

        generate_from_html(&source_path, &dest_path, &layout_folder);

        let generated_dest_code = fs::read_to_string(dest_path).unwrap();

        assert_eq!(generated_dest_code, dest_code);
    }
    #[test]
    fn check_generation_from_sass() {
        let base_path: PathBuf = [r".", "test_cache", "generation-from-scss-test"]
            .iter()
            .collect();
        create_dir_all(&base_path).unwrap();

        let mut source_path = base_path.clone();
        source_path.push("source");
        source_path.set_extension("scss");

        let source_scss = "
$font-stack: Helvetica, sans-serif;
$primary-color: #333;

body {
    font: 100% $font-stack;
    color: $primary-color;
}
    "
        .to_string();

        let expected_css = "body{font:100% Helvetica,sans-serif;color:#333}\n".to_string();

        fs::write(&source_path, source_scss).unwrap();

        let mut dest_path = base_path;
        dest_path.push("dest");
        dest_path.set_extension("css");

        _scss_to_css(&source_path, &dest_path);

        let generated_css = fs::read(dest_path).unwrap();
        assert_eq!(expected_css, str::from_utf8(&generated_css).unwrap());
    }
}

/// The main glue to generate code based on file types
/// This should call other funcitons to generate, and
/// itself should be minimal ideally.
pub fn build_project() {
    // initialize the target directory, where built site will go
    fs::create_dir_all::<PathBuf>([r".", "target"].iter().collect()).unwrap();

    generate_pages();
    // generate_md_pages();
}

fn generate_pages() {
    // let html_pages_folder = "./_pages";

    let content_folders = vec!["./pages".to_string(), "./posts".to_string()];

    for c in content_folders {
        // recursively follow all files in _pages and generate
        for source_path in WalkDir::new(c) {
            // entry is of form ./_pages/index.html
            let source_path = source_path.unwrap();
            let mut dest_path: PathBuf = [r".", "target"].iter().collect();

            if source_path.path().is_file() {
                // @TODO : Find better way to decide destination path without using str
                dest_path.push(&source_path.path().to_str().unwrap()[9..]);

                let layout_folder: PathBuf = [r".", "layouts"].iter().collect();
                if dest_path.extension().and_then(OsStr::to_str) == Some("html") {
                    generate_from_html(
                        &source_path.path().to_path_buf(),
                        &dest_path,
                        &layout_folder,
                    );
                } else if dest_path.extension().and_then(OsStr::to_str) == Some("md") {
                    dest_path.set_extension("html");
                    generate_from_md(
                        &source_path.path().to_path_buf(),
                        &dest_path,
                        &layout_folder,
                    )
                } else {
                    // only during testing
                    // to be fixed
                    println!(
                        "Not touching unknown file: {}",
                        source_path.path().to_str().unwrap()
                    );
                }
            } else {
                // the path is a folder
                dest_path.push(&source_path.path().to_str().unwrap()[8..]);
                create_dir_all(dest_path).unwrap();
            }
        }
    }
}

// Given a source html and destination path,
// this will use Handlebar to generate
// an HTML file with layout plugged in.
fn generate_from_html(source_path: &Path, dest_path: &Path, layout_folder: &Path) {
    let layout: String;

    let source_file = layout_utils::SourceFile {
        filetype: layout_utils::SourceFileType::Html,
        path: source_path.to_path_buf(),
    };

    let layout_detected = detect_layout(source_file, layout_folder);

    // will be none when no layout is detected
    // or when non existent layout is provided
    match layout_detected {
        Some(s) => layout = s,
        None => {
            println!(
                "[WARN] Failed to match layout in {}",
                source_path.to_str().unwrap()
            );
            return;
        }
    };

    let content = fs::read_to_string(source_path).expect("Could not read file");

    // build the layout template path
    let mut layout_template_path: PathBuf = layout_folder.to_path_buf();
    layout_template_path.push(layout);
    layout_template_path.set_extension("html");

    let layout_template = fs::read_to_string(layout_template_path).expect("error reading layout");

    // create a handlebars instance
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(source_path.to_str().unwrap(), &layout_template)
        .unwrap();

    // plug in the variable
    let mut data = BTreeMap::new();
    data.insert("content".to_string(), content);

    // finally reder and write the file
    fs::write(
        dest_path,
        handlebars
            .render(source_path.to_str().unwrap(), &data)
            .unwrap(),
    )
    .unwrap();
}

/// Given a source md and destination path,
/// this will use Handlebar to generate
/// an HTML file with layout plugged in.
pub fn generate_from_md(source_path: &Path, dest_path: &Path, layout_folder: &Path) {
    // read source md file
    let source_data = fs::read_to_string(source_path).unwrap();
    // get front matter and md without that
    let (front_matter, source_data) = parse_front_matter(&source_data);

    // let mut template_vars = vars.clone();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("markdown_data", source_data)
        .unwrap();

    // plug in any variables from front matter to md
    let plugged_md = handlebars.render("markdown_data", &front_matter).unwrap();

    let options = ComrakOptions::default();

    // convert md to html
    let html = markdown_to_html(&plugged_md, &options);

    let mut data = BTreeMap::new();
    data.insert("content".to_string(), html);

    // build layout path and read layout
    let mut layout_path: PathBuf = layout_folder.to_path_buf();
    layout_path.push(front_matter["data"]["layout"].as_str().unwrap());
    layout_path.set_extension("html");
    let layout_string = fs::read_to_string(layout_path).unwrap();

    handlebars
        .register_template_string("raw_html", layout_string)
        .unwrap();

    let html_layout_plugged = handlebars.render("raw_html", &front_matter).unwrap();

    handlebars
        .register_template_string("html", html_layout_plugged)
        .unwrap();

    // generate html with layout
    let final_html = handlebars.render("html", &data).unwrap();

    // and finally, write to file...
    fs::write(dest_path, final_html).unwrap();
}

/// Utility function to convert scss to css
fn _scss_to_css(source_path: &Path, dest_path: &Path) {
    // @TODO : Also remove underscore from function name

    let format = output::Format {
        style: output::Style::Compressed,
        ..Default::default()
    };

    let css = compile_scss_path(source_path, format).unwrap();
    fs::write(dest_path, str::from_utf8(&css).unwrap()).unwrap();
}

fn parse_front_matter(source_data: &str) -> (Value, String) {
    let mut extractor = Extractor::new(source_data);
    extractor.select_by_terminator("+++");
    extractor.discard_first_line();

    let (front_matter, document): (Vec<&str>, &str) = extractor.split();

    let document = document.trim().to_string();
    let front_matter = front_matter.join("\n");

    let front_matter_toml = front_matter.parse::<Value>().unwrap();

    (front_matter_toml, document)
}
