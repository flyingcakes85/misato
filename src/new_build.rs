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
    str,
};
use toml::Deserializer;
use toml::Value as Toml;
// use toml::Value;
use walkdir::WalkDir;

pub fn build() {
    fs::create_dir_all("target").unwrap();
    let mut base_attributes = get_attributes();

    let mut handlebars = Handlebars::new();

    let mut renderlist_pages = Vec::<(String, PathBuf)>::new();
    let mut renderlist_posts = Vec::<(String, PathBuf)>::new();

    discover_pages(&mut handlebars, &mut renderlist_pages);
    discover_posts(&mut handlebars, &mut renderlist_posts);

    discover_layouts(&mut handlebars);

    base_attributes.insert("title".to_string(), to_json("Test Title"));

    render_pages(renderlist_pages, &handlebars, &base_attributes);
    // render_posts(renderlist_posts, &handlebars, &base_attributes);

    generate_css();
}

fn render_pages(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) {
    println!("{:?}", renderlist);
    println!("{:?}", handlebars);
    println!("{:?}", data);
    for (template_name, template_path) in renderlist {
        let dest_path = template_path.to_str().unwrap().replace("pages/", "target/");
        println!("{:?}", dest_path);

        fs::write(
            Path::new(&dest_path).with_extension("html"),
            handlebars.render(&template_name, &data).unwrap(),
        )
        .unwrap();
    }
}

fn render_posts(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) {
    for (template_name, template_path) in renderlist {
        let source_data = fs::read_to_string(&template_path).unwrap();

        let (front_matter, source_data) = parse_front_matter(&source_data);

        let mut plug_data = data.clone();
        for m in front_matter.as_table() {
            for (k, v) in m {
                plug_data.insert(
                    k.to_string(),
                    serde_json::from_str(&serde_json::to_string(&toml_to_json(v.clone())).unwrap())
                        .unwrap(),
                );
            }
        }

        plug_data.insert("title".to_string(), plug_data["info"]["title"].clone());

        let mut s_handlebars = handlebars.clone();
        s_handlebars
            .register_template_string("raw_markdown", source_data)
            .unwrap();
        let source_data = s_handlebars.render("raw_markdown", &plug_data).unwrap();

        let comark_options = ComrakOptions::default();
        let html = markdown_to_html(&source_data, &comark_options);

        s_handlebars
            .register_template_string("markdown_data", &html)
            .unwrap();
        println!(
            "{}",
            s_handlebars
                .render(plug_data["data"]["layout"].as_str().unwrap(), &plug_data)
                .unwrap()
        );

        let dest_path = template_path.to_str().unwrap().replace("posts/", "target/");

        fs::write(
            Path::new(&dest_path).with_extension("html"),
            s_handlebars
                .render(plug_data["data"]["layout"].as_str().unwrap(), &plug_data)
                .unwrap(),
        )
        .unwrap();
    }
}

fn parse_front_matter(source_data: &str) -> (Toml, String) {
    let mut extractor = Extractor::new(source_data);
    extractor.select_by_terminator("+++");
    extractor.discard_first_line();

    let (front_matter, document): (Vec<&str>, &str) = extractor.split();

    let document = document.trim().to_string();
    let front_matter = front_matter.join("\n");

    let front_matter_toml = front_matter.parse::<Toml>().unwrap();

    (front_matter_toml, document)
}

fn get_attributes() -> Map<String, Json> {
    let mut attributes: Map<String, Json> = Map::new();

    let config_string = fs::read_to_string("config.toml").unwrap();
    let config_toml = config_string.parse::<toml::Value>().unwrap();

    for c in config_toml.as_table() {
        for t in c {
            attributes.insert(
                t.0.to_string(),
                serde_json::from_str(&serde_json::to_string(&toml_to_json(t.1.clone())).unwrap())
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
