use comrak::{markdown_to_html, plugins, ComrakOptions};
use extract_frontmatter::Extractor;
use handlebars::{to_json, Handlebars};
use rsass::{compile_scss_path, output};
use serde::Serialize;
use serde_json::value::Map;
use serde_json::Value as Json;
use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsStr,
    fmt::Result,
    fs,
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::exit,
    str,
};
use toml::Value as Toml;
use toml::{to_vec, Deserializer};
use walkdir::WalkDir;

pub fn build() {
    fs::create_dir_all("target").unwrap();

    // This will be the main attributes entity.
    let mut base_attributes: Map<String, Json> = get_attributes();

    // Handlebars is the main template managing part.
    // This instance of handlebars will be passed
    // around to functions that will clone it and
    // use for their funcitonality. This allows
    // easy overriding of attributes.
    let mut handlebars = Handlebars::new();

    let mut renderlist_pages = Vec::<(String, PathBuf)>::new();
    let mut renderlist_posts = Vec::<(String, PathBuf)>::new();

    // discover and get the renderlists
    discover_pages(&mut handlebars, &mut renderlist_pages);
    discover_posts(&mut handlebars, &mut renderlist_posts);

    discover_layouts(&mut handlebars);

    let post_list = render_posts(renderlist_posts, &handlebars, &base_attributes);

    base_attributes.insert("posts".to_string(), toml_to_json(Toml::Array(post_list)));

    render_pages(renderlist_pages, &handlebars, &base_attributes);
    generate_css(base_attributes["config"]["scss"].as_array().unwrap());
}

// Render pagse (html or hbs)
fn render_pages(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) {
    for (template_name, template_path) in renderlist {
        let dest_path = template_path.to_str().unwrap().replace("pages/", "target/");

        fs::write(
            Path::new(&dest_path).with_extension("html"),
            handlebars.render(&template_name, &data).unwrap(),
        )
        .unwrap();
    }
}

// Render posts and return a vector containing front matter
fn render_posts(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) -> Vec<Toml> {
    let mut html_to_write: String;
    let mut post_list: Vec<Toml> = Vec::<Toml>::new();
    let comark_options = ComrakOptions::default();
    let mut dest_path: PathBuf;
    let mut rel_path: PathBuf;

    for (file_name, template_path) in renderlist {
        // read the source md text
        let source_md_data = fs::read_to_string(&template_path).unwrap();

        // split source_md_data to front matter and actual text
        // front_matter is Toml
        let (mut front_matter, raw_md_text) = parse_front_matter(source_md_data);

        let raw_md_text = raw_md_text.trim().to_string();

        // plug data has all attributes
        let mut plug_data: Map<String, Json> = data.clone();

        // rel_path = PathBuf::new();
        dest_path = PathBuf::new();
        dest_path.push("target");

        // create a local clone of handlebars
        let mut md_handlebars = handlebars.clone();

        for (k, v) in front_matter.as_table().unwrap() {
            plug_data.insert(k.to_string(), value_to_json(v));
        }

        // TODO : this generates a relative path
        // configure it to use complete domain path
        rel_path = get_dest_path(&plug_data, &mut md_handlebars, file_name);
        rel_path.set_extension("html");
        dest_path.push(rel_path.clone());

        front_matter["data"].as_table_mut().unwrap().insert(
            "rel_path".to_string(),
            Toml::String(String::from(rel_path.to_str().unwrap())),
        );

        // register the raw markdown data
        md_handlebars
            .register_template_string("raw_markdown", raw_md_text)
            .unwrap();

        // render markdown with attributes
        let plugged_md_text = md_handlebars.render("raw_markdown", &plug_data).unwrap();

        // convert plugged markdown to html
        let generated_html = markdown_to_html(&plugged_md_text, &comark_options);

        // register the final markdown_data template
        // at this point this it technically html
        // but registering it as markdown_data makes
        // sense for end user
        md_handlebars
            .register_template_string("markdown_data", &generated_html)
            .unwrap();

        // generate final html
        // and set rel_path
        html_to_write = if json_map_key_exists(&plug_data, "data", "layout") {
            md_handlebars
                .render(front_matter["data"]["layout"].as_str().unwrap(), &plug_data)
                .unwrap()
        } else {
            generated_html
        };

        post_list.push(front_matter.clone());

        // write out the html file
        fs::create_dir_all(&dest_path.parent().unwrap()).unwrap();
        fs::write(&dest_path, &html_to_write).unwrap();
    }

    post_list
}

// Utility function to convert Toml Value to Json Value
fn value_to_json(toml: &Toml) -> Json {
    serde_json::from_str(&serde_json::to_string_pretty(&toml_to_json(toml.clone())).unwrap())
        .unwrap()
}

// Parse the front matter and return as Toml Value
// We eventually convert it to a Json Value
fn parse_front_matter(source_data: String) -> (Toml, String) {
    let mut extractor = Extractor::new(&source_data);
    extractor.select_by_terminator("+++");
    extractor.discard_first_line();

    let (front_matter, document): (Vec<&str>, &str) = extractor.split();

    let document = document.trim().to_string();
    let front_matter = front_matter.join("\n");

    let front_matter_toml = front_matter.parse::<Toml>().unwrap();

    (front_matter_toml, document)
}

// checks if keys exists in 2 level json map
fn json_map_key_exists(data: &Map<String, Json>, k1: &str, k2: &str) -> bool {
    if data.contains_key(k1) {
        if data[k1].as_object().unwrap().contains_key(k2) {
            return true;
        }
        return false;
    }
    false
}

fn get_dest_path(
    data: &Map<String, Json>,
    handlebars: &mut Handlebars,
    file_name: String,
) -> PathBuf {
    let mut dest_path = PathBuf::new();

    // Following the preference, first check override in the post itself
    if json_map_key_exists(&data, "data", "post_path") {
        println!("found custom path in post");
        for p in data["data"]["post_path"].as_str().unwrap().split("/") {
            dest_path.push(p.to_string());
        }
    }
    // if no overrides then use the global path in config.toml
    else if json_map_key_exists(&data, "config", "blog_path") {
        println!("Using path defined in global config");
        handlebars
            .register_template_string(
                "blog_path_internal",
                data["config"]["blog_path"].as_str().unwrap(),
            )
            .unwrap();

        for p in handlebars
            .render("blog_path_internal", &data)
            .unwrap()
            .split("/")
        {
            dest_path.push(p.to_string());
        }

        // manually push the file name because
        // blog_path only defines the folder
        dest_path.push(file_name);
    }
    // not having path in config.toml is an error
    // but we'll handle it. Todo.
    else {
        println!("[ERR] No post path found");
        // TODO : Don't make app exit when no path found;
        // use a generic path in this case
        exit(0);
    }

    dest_path
}

/// Reads global attributes from config.toml in project root
/// Returns as a map of string to json
/// because we feed that to handlebars
fn get_attributes() -> Map<String, Json> {
    let mut attributes: Map<String, Json> = Map::new();

    let config_string = fs::read_to_string("config.toml").unwrap();
    let config_toml = config_string.parse::<toml::Value>().unwrap();

    for c in config_toml.as_table() {
        for t in c {
            attributes.insert(
                t.0.to_string(),
                serde_json::from_str(
                    &serde_json::to_string_pretty(&toml_to_json(t.1.clone())).unwrap(),
                )
                .unwrap(),
            );
        }
    }

    attributes
}

/// Searches for available HTML pages and registers them
pub fn discover_pages(handlebars: &mut Handlebars, renderlist: &mut Vec<(String, PathBuf)>) {
    if Path::new("pages").exists() {
        for source_path in WalkDir::new("pages") {
            let source_path = source_path.unwrap();

            // File extension doesn't matter, but we will read only html or hbs files
            if source_path.path().extension().and_then(OsStr::to_str) == Some("html")
                || source_path.path().extension().and_then(OsStr::to_str) == Some("hbs")
            {
                let mut template_name = get_file_name(&source_path.path());

                // suffix name with "_pages" to prevent
                // possible clash with other temlpates
                template_name.push_str("_page");
                renderlist.push((template_name.clone(), source_path.path().to_path_buf()));
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

/// Searches for available md posts and registers them
/// Also returns a vector with filenames and their paths
pub fn discover_posts(handlebars: &mut Handlebars, renderlist: &mut Vec<(String, PathBuf)>) {
    if Path::new("posts").exists() {
        for source_path in WalkDir::new("posts") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("md") {
                let mut template_name = get_file_name(&source_path.path());

                // suffix name with "_post" to prevent
                // possible clash with other temlpates
                template_name.push_str("_post");
                renderlist.push((template_name.clone(), source_path.path().to_path_buf()));
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

/// Searches for available layouts and registers them
pub fn discover_layouts(handlebars: &mut Handlebars) {
    if Path::new("layouts").exists() {
        for source_path in WalkDir::new("layouts") {
            let source_path = source_path.unwrap();

            // File extension doesn't matter, but we will read only html or hbs files
            if source_path.path().extension().and_then(OsStr::to_str) == Some("html")
                || source_path.path().extension().and_then(OsStr::to_str) == Some("hbs")
            {
                let mut template_name = get_file_name(&source_path.path());

                // suffix name with "_layout" to prevent
                // possible clash with other temlpates
                template_name.push_str("_layout");
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

/// Converts Toml Value to Json Value
fn toml_to_json(toml: Toml) -> Json {
    match toml {
        Toml::String(s) => Json::String(s),
        Toml::Integer(i) => Json::Number(i.into()),
        Toml::Float(f) => {
            let n = serde_json::Number::from_f64(f).expect("float infinite and nan not allowed");
            Json::Number(n)
        }
        Toml::Boolean(b) => Json::Bool(b),
        Toml::Array(arr) => Json::Array(arr.into_iter().map(toml_to_json).collect()),
        Toml::Table(table) => Json::Object(
            table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect(),
        ),
        Toml::Datetime(dt) => Json::String(dt.to_string()),
    }
}

/// Gets file name from a file path
fn get_file_name(p: &Path) -> String {
    Path::new(p.file_stem().unwrap())
        .file_name()
        .unwrap()
        .to_str()
        .to_owned()
        .ok_or("[ERR] Could not get file name")
        // TODO : Handle error
        .unwrap()
        .to_string()
}

/// Generate PathBuf from a unix path (forward slash)
fn path_from_string(path_str: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for p in path_str.split("/") {
        path.push(p);
    }

    path
}

/// Replace prefix of a PathBuf
fn switch_path_prefix(path: &PathBuf, old_prefix: &str, new_prefix: &str) -> PathBuf {
    let mut final_path = PathBuf::new();
    final_path.push(new_prefix);

    for p in path
        .clone()
        .to_str()
        .unwrap()
        .to_string()
        .strip_prefix(old_prefix)
        .unwrap()
        .split("/")
    {
        final_path.push(p);
    }
    final_path
}

/// Generates CSS from SCSS
/// Copies CSS files as they are
fn generate_css(scss_list: &Vec<Json>) {
    // first copy the CSS files
    for source_path in WalkDir::new("styles") {
        let source_path = source_path.unwrap();
        let source_path = source_path.path();

        if source_path.extension().and_then(OsStr::to_str) == Some("css") {
            let dest_path = switch_path_prefix(&source_path.to_path_buf(), "styles", "target");

            fs::copy(source_path, dest_path).unwrap();
        }
    }

    // Now work on the SCSS files
    // Create a list of paths of SCSS files
    let mut scss_to_render = Vec::<PathBuf>::new();
    for entry in scss_list {
        let e_path = entry.as_str().unwrap();
        let e_path = path_from_string(e_path);

        let mut scss_path = PathBuf::new();
        scss_path.push("styles");
        scss_path.push(e_path);
        scss_to_render.push(scss_path);
    }

    // Define the format options for SCSS generator
    let scss_format = output::Format {
        style: output::Style::Compressed,
        ..Default::default()
    };

    // Iterate thourgh the list and render one by one
    for scss_file in scss_to_render {
        let css = compile_scss_path(&scss_file, scss_format).unwrap();
        let dest_path = switch_path_prefix(&scss_file.to_path_buf(), "styles", "target");

        fs::write(dest_path, css).unwrap();
    }
}
