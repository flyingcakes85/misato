use handlebars::Handlebars;
use std::fs::{create_dir_all, metadata};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{collections::BTreeMap, fs};
use walkdir::WalkDir;

use crate::build::layout_utils::detect_layout;

mod layout_utils;

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, create_dir_all},
        path::PathBuf,
    };

    use crate::build::generate_from_html;

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
}

/// The main glue to generate code based on file types
/// This should call other funcitons to generate, and
/// itself should be minimal ideally.
pub fn build_project() {
    // initialize the target directory, where built site will go
    fs::create_dir_all::<PathBuf>([r".", "target"].iter().collect()).unwrap();

    let html_pages_folder = "./_pages";

    // recursively follow all files in _pages and generate
    for source_path in WalkDir::new(html_pages_folder) {
        // entry is of form ./_pages/index.html
        let source_path = source_path.unwrap();
        let mut dest_path: PathBuf = [r".", "target"].iter().collect();

        // generate only if the path is a file (not folder)
        if metadata(source_path.path().to_str().unwrap())
            .unwrap()
            .is_file()
        {
            // @TODO : Find better way to decide destination path without using str
            dest_path.push(&source_path.path().to_str().unwrap()[9..]);

            let dest_path_str = dest_path.to_str().unwrap();
            if dest_path_str[dest_path_str.len() - 4..] == *"html" {
                let layout_folder: PathBuf = [r".", "_layouts"].iter().collect();
                generate_from_html(
                    &source_path.path().to_path_buf(),
                    &dest_path,
                    &layout_folder,
                );
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

// Given a source and destination path,
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
    layout = match layout_detected {
        Some(s) => s,
        None => {
            println!(
                "[ERR] Failed to match layout in {}",
                source_path.to_str().unwrap()
            );
            exit(1);
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
