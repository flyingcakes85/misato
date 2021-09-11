use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    vec,
};

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, create_dir_all, File},
        path::PathBuf,
    };

    use crate::build::layout_utils::{available_layouts, names_from_path};

    use super::{detect_layout, SourceFile, SourceFileType};

    #[test]
    /// Tests names_from_path()
    fn check_layout_names() {
        let mut path_list: Vec<PathBuf> = vec![];
        let mut path: PathBuf;

        // simple path
        path = [r"path", "to", "file.txt"].iter().collect();
        path_list.push(path);

        // path with spaces
        path = [r"path", "to", "a cool", "webpage.html"].iter().collect();
        path_list.push(path);

        // file without extension
        path = [r"path", "to", "data"].iter().collect();
        path_list.push(path);

        let expected_file_list: Vec<String> = vec![
            "file".to_string(),
            "webpage".to_string(),
            "data".to_string(),
        ];

        assert_eq!(names_from_path(path_list), expected_file_list);
    }

    /// Helper function to create common variables
    fn get_init_data(dir_name: &str) -> (PathBuf, Vec<String>) {
        let base_path: PathBuf = [r".", "test_cache", dir_name, "_layouts"].iter().collect();
        let layouts: Vec<String> = vec![
            "about".to_string(),
            "content_page".to_string(),
            "home".to_string(),
            "post".to_string(),
            "profile".to_string(),
        ];

        (base_path, layouts)
    }

    /// Helper function to create layout files
    /// and return a Vector of layout paths
    /// along with the init data
    fn create_layouts_directory(dir_name: &str) -> (Vec<String>, PathBuf, Vec<String>) {
        let (base_path, layouts) = get_init_data(dir_name);
        create_dir_all(&base_path).unwrap();

        let mut layout_path_list: Vec<String> = Vec::new();

        for l in layouts.clone() {
            let mut layout_path = base_path.clone();
            layout_path.push(l);
            layout_path.set_extension("html");

            // also build the expected path list simultaneously
            layout_path_list.push(layout_path.clone().to_str().unwrap().to_string());

            File::create(layout_path).unwrap();
        }

        (layout_path_list, base_path, layouts)
    }

    #[test]
    /// Tests available_layouts()
    fn check_layout_discovery() {
        // init the testing directory with layouts
        let (mut expected_path_list, base_path, _) =
            create_layouts_directory("layout_discovery_test");

        // convect PathBuf into String
        let mut discovered_layouts: Vec<String> = available_layouts(base_path)
            .unwrap()
            .into_iter()
            .map(|path| path.to_str().unwrap().to_string())
            .collect();

        // sort them, just to be sure
        discovered_layouts.sort_by_key(|a| a.to_lowercase());
        expected_path_list.sort_by_key(|a| a.to_lowercase());

        assert_eq!(discovered_layouts, expected_path_list);
    }

    #[test]
    /// Tests function detect_layout
    fn check_layout_detection() {
        let (_, base_path, _) = create_layouts_directory("layout_detection_test");

        let mut source_file_path = base_path.clone();

        // create the _pages directory
        source_file_path.pop();
        source_file_path.push("_pages");
        create_dir_all(&source_file_path).unwrap();

        // create the actual file
        source_file_path.push("product");
        source_file_path.set_extension("html");

        let source_file_contents = "
<!-- layout: content_page -->

product description
"
        .to_string();
        fs::write(&source_file_path, source_file_contents).unwrap();

        let source_file = SourceFile {
            filetype: SourceFileType::Html,
            path: source_file_path.clone(),
        };

        let detected_layout = detect_layout(source_file.clone(), &base_path);

        assert!(detected_layout.is_some());
        assert_eq!("content_page", detected_layout.unwrap());

        // now try testing with non existent layout

        let source_file_contents = "
<!-- layout: non_existent_layout -->

product description
"
        .to_string();

        fs::write(&source_file_path, source_file_contents).unwrap();

        let detected_layout = detect_layout(source_file, &base_path);
        assert!(detected_layout.is_none());
    }
}

#[derive(Clone)]
pub enum SourceFileType {
    Html,
    _Md,
}

#[derive(Clone)]
pub struct SourceFile {
    pub filetype: SourceFileType,
    pub path: PathBuf,
}

fn available_layouts(layout_folder: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut layouts = vec![];

    for path in fs::read_dir(layout_folder)? {
        let path = path?.path();
        if let Some("html") = path.extension().and_then(OsStr::to_str) {
            layouts.push(path.to_owned());
        }
    }
    Ok(layouts)
}

fn names_from_path(paths: Vec<PathBuf>) -> Vec<String> {
    let mut layout_list: Vec<String> = vec![];
    let mut file_name: String;

    for p in paths {
        file_name = p
            .file_name()
            .unwrap()
            .to_str()
            .to_owned()
            .ok_or("[ERR] Could not get filename of layout")
            .unwrap()
            .to_string();

        layout_list.push(file_name.split('.').collect::<Vec<&str>>()[0].to_string());
    }

    layout_list
}

pub fn detect_layout(source_file: SourceFile, layout_folder: &Path) -> Option<String> {
    let mut layout: String = String::new();
    let contents =
        fs::read_to_string(source_file.path).expect("Something went wrong reading the file");

    for line in contents.lines() {
        match source_file.filetype {
            SourceFileType::Html => {
                if line.contains("<!-- layout:") {
                    layout = String::from(&line[13..(line.len() - 4)]);
                    break;
                }
            }
            SourceFileType::_Md => {}
        }
    }

    let available_layouts =
        names_from_path(available_layouts(layout_folder.to_path_buf()).unwrap());
    if available_layouts.is_empty() {
        eprintln!("[ERR] No layouts defined in _layout.");
    }

    for l in available_layouts {
        if l == layout {
            return Some(layout);
        }
    }

    None
}
