#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use comrak::{markdown_to_html, ComrakOptions};
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
// use toml::Value;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Clone)]
struct Post {
    title: String,
    subtitle: String,
    author: String,
    layout: String,
    path: String,
    banner: String,
    categories: Vec<String>,
    styles: Vec<String>,
}

pub fn build() {
    fs::create_dir_all("target").unwrap();
    let mut base_attributes = get_attributes();

    let mut handlebars = Handlebars::new();

    let mut renderlist_pages = Vec::<(String, PathBuf)>::new();
    let mut renderlist_posts = Vec::<(String, PathBuf)>::new();

    discover_pages(&mut handlebars, &mut renderlist_pages);
    discover_posts(&mut handlebars, &mut renderlist_posts);

    discover_layouts(&mut handlebars);

    let post_list = render_posts(renderlist_posts, &handlebars, &base_attributes);
    // base_attributes.insert("categories".to_string(), serde_json::fropost_list);
    // let s = String::from_utf8(serde_json::ser::to_vec_pretty(&post_list).unwrap()).unwrap();
    // println!("{}", s);
    // println!("post list is {:?}", post_list);

    // println!("{:?}", post_list);

    // base_attributes.insert(
    //     "posts".to_string(),
    //     serde_json::from_str(
    //         &String::from_utf8(serde_json::ser::to_vec(&post_list).unwrap()).unwrap(),
    //     )
    //     .unwrap(),
    // );

    // println!("{:?}", base_attributes);

    // render_pages(renderlist_pages, &handlebars, &base_attributes);
    // generate_css();
}

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

// render posts and return a vector of posts
fn render_posts(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) -> Vec<Post> {
    let mut html_to_write: String = String::new();
    let mut post_list: Vec<Post> = Vec::<Post>::new();
    let mut post: Post;

    for (_, template_path) in renderlist {
        // read the source md text
        let source_md_data = fs::read_to_string(&template_path).unwrap();

        // split source_md_data to front matter and actual text
        // front_matter is Toml
        let (front_matter, md_text) = parse_front_matter(source_md_data);

        let md_text = md_text.trim().to_string();

        // plug data has all attributes
        let mut plug_data: Map<String, Json> = data.clone();

        // println!("{:#?}\n{}\n", plug_data, front_matter);

        for (k, v) in front_matter.as_table().unwrap() {
            plug_data.insert(k.to_string(), value_to_json(v));
        }
        println!("{:#?}\n", plug_data);
    }

    post_list
}

fn bruh(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) -> Vec<Post> {
    let mut html_to_write: String = String::new();
    let mut post_list: Vec<Post> = Vec::<Post>::new();
    let mut post: Post;

    for (_, template_path) in renderlist {
        let source_data = fs::read_to_string(&template_path).unwrap();
        // generate destination path
        let dest_path = template_path.to_str().unwrap().replace("posts/", "target/");

        // plug data contains all attributes to add to MD file
        let (front_matter, source_data) = parse_front_matter(source_data);
        let mut plug_data = data.clone();

        // copy global attributes
        for m in front_matter.as_table() {
            for (k, v) in m {
                plug_data.insert(k.to_string(), value_to_json(v));
            }
        }

        println!("{:?}", plug_data);

        // populate the post list
        post = front_matter_toml_to_post(front_matter);
        post_list.push(post.clone());
        // let post_t = post.clone();

        // println!("{:?}", post);

        // Override title from markdown file if it exists
        if plug_data.contains_key("info") {
            if plug_data["info"].as_object().unwrap().contains_key("title") {
                plug_data.insert("title".to_string(), plug_data["info"]["title"].clone());
            }
        }

        // create a local clone of handlebars
        let mut md_handlebars = handlebars.clone();

        // register the raw markdown data and rewrite source_data
        md_handlebars
            .register_template_string("raw_markdown", source_data)
            .unwrap();
        let source_data = md_handlebars.render("raw_markdown", &plug_data).unwrap();

        // convert plugged markdown to html
        let comark_options = ComrakOptions::default();
        let html = markdown_to_html(&source_data, &comark_options);

        // register the final markdown_data template
        md_handlebars
            .register_template_string("markdown_data", &html)
            .unwrap();

        // generate final html
        // check if layout exists
        if plug_data.contains_key("data") {
            if plug_data["data"]
                .as_object()
                .unwrap()
                .contains_key("layout")
            {
                // it has layout definition
                html_to_write = md_handlebars
                    .render(plug_data["data"]["layout"].as_str().unwrap(), &plug_data)
                    .unwrap();
            } else {
                // no layout definition
                // simply output html
                html_to_write = html;
            }
        }

        // write out the html file
        fs::write(Path::new(&dest_path).with_extension("html"), &html_to_write).unwrap();
    }

    post_list
}

fn value_to_json(toml: &Toml) -> Json {
    serde_json::from_str(&serde_json::to_string_pretty(&toml_to_json(toml.clone())).unwrap())
        .unwrap()
}

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

fn front_matter_toml_to_post(fm: Toml) -> Post {
    let front_matter = fm.as_table().unwrap();
    let mut title = String::new();
    let mut subtitle = String::new();
    let mut author = String::new();
    let mut layout = String::new();
    let mut path = String::new();
    let mut banner = String::new();
    let mut categories = Vec::<String>::new();
    let mut styles = Vec::<String>::new();

    if front_matter.contains_key("info") {
        if front_matter["info"]
            .as_table()
            .unwrap()
            .contains_key("title")
        {
            title = front_matter["info"]["title"]
                .to_string()
                .strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .to_string();
        }

        if front_matter["info"]
            .as_table()
            .unwrap()
            .contains_key("subtitle")
        {
            subtitle = front_matter["info"]["subtitle"]
                .to_string()
                .strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .to_string();
        }

        if front_matter["info"]
            .as_table()
            .unwrap()
            .contains_key("author")
        {
            author = front_matter["info"]["author"]
                .to_string()
                .strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .to_string();
        }

        if front_matter["info"]
            .as_table()
            .unwrap()
            .contains_key("categories")
        {
            categories = front_matter["info"]["categories"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.to_string())
                .collect();
        }
    }

    if front_matter.contains_key("data") {
        if front_matter["data"]
            .as_table()
            .unwrap()
            .contains_key("styles")
        {
            styles = front_matter["data"]["styles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.to_string())
                .collect();
        }

        if front_matter["data"]
            .as_table()
            .unwrap()
            .contains_key("layout")
        {
            layout = front_matter["data"]["layout"].to_string();
        }

        if front_matter["data"]
            .as_table()
            .unwrap()
            .contains_key("path")
        {
            path = front_matter["data"]["path"].to_string();
        }

        if front_matter["data"]
            .as_table()
            .unwrap()
            .contains_key("banner")
        {
            banner = front_matter["data"]["banner"].to_string();
        }
    }

    let p: Post = Post {
        title,
        subtitle,
        author,
        layout,
        path,
        banner,
        categories,
        styles,
    };
    p
}

// checks if keys exists in 2 level json map
fn json_map_key_exists(data: &Map<String, Json>, k1: &String, k2: &String) -> bool {
    if data.contains_key(k1) {
        if data[k1].as_object().unwrap().contains_key(k2) {
            return true;
        }
        return false;
    }
    false
}

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

pub fn discover_pages(handlebars: &mut Handlebars, renderlist: &mut Vec<(String, PathBuf)>) {
    if Path::new("pages").exists() {
        for source_path in WalkDir::new("pages") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("html")
                || source_path.path().extension().and_then(OsStr::to_str) == Some("hbs")
            {
                let mut template_name = get_file_name(&source_path.path());
                template_name.push_str("_page");
                renderlist.push((template_name.clone(), source_path.path().to_path_buf()));
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

pub fn discover_posts(handlebars: &mut Handlebars, renderlist: &mut Vec<(String, PathBuf)>) {
    if Path::new("posts").exists() {
        for source_path in WalkDir::new("posts") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("md") {
                let mut template_name = get_file_name(&source_path.path());
                template_name.push_str("_post");
                renderlist.push((template_name.clone(), source_path.path().to_path_buf()));
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

pub fn discover_layouts(handlebars: &mut Handlebars) {
    if Path::new("layouts").exists() {
        for source_path in WalkDir::new("layouts") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("html")
                || source_path.path().extension().and_then(OsStr::to_str) == Some("hbs")
            {
                let mut template_name = get_file_name(&source_path.path());
                template_name.push_str("_layout");
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

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

fn generate_css() {
    for p in WalkDir::new("styles") {
        let p = p.unwrap();
        let p = p.path();
        if p.extension().and_then(OsStr::to_str) == Some("css") {
            fs::copy(p, p.to_str().unwrap().replace("styles/", "target/")).unwrap();
        }
    }
}
