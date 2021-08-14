use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    generate_styles();
    generate_tests();
}

fn generate_styles() {
    let outfile_path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated_styles.rs");
    let mut outfile = fs::File::create(outfile_path).unwrap();
    write!(
        outfile,
        r#"/// Automatically generated styles.

use std::collections::HashMap;

use super::GobanStyle;

lazy_static::lazy_static! {{
    pub static ref GENERATED_STYLES: HashMap<&'static str, GobanStyle> = {{
        let mut m = HashMap::new();"#
    )
    .unwrap();

    let styles = std::fs::read_dir("./resources/styles").unwrap();
    for result in styles {
        let path = result.unwrap().path().canonicalize().unwrap();
        if path.extension().and_then(OsStr::to_str) == Some("toml") {
            write_style(&mut outfile, &path);
        }
    }

    write!(
        outfile,
        r#"
        m
    }};
}}
"#
    )
    .unwrap();
}

fn write_style(outfile: &mut fs::File, path: &Path) {
    let style_name = path
        .with_extension("")
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    write!(
        outfile,
        r#"
        m.insert("{style_name}", toml::from_str(include_str!(r"{path}")).unwrap());"#,
        style_name = style_name,
        path = path.display(),
    )
    .unwrap();
}

fn generate_tests() {
    let outfile_path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated_tests.rs");
    let mut outfile = fs::File::create(outfile_path).unwrap();
    write_tests_header(&mut outfile);

    let test_data = std::fs::read_dir("./tests/data").unwrap();
    for result in test_data {
        let entry = result.unwrap();
        if entry.metadata().unwrap().is_dir() {
            write_test(&mut outfile, &entry);
        }
    }
}

fn write_tests_header(outfile: &mut fs::File) {
    write!(
        outfile,
        r#"/// Automatically generated tests.

use sgf_render::make_svg;
use sgf_render::args;
"#,
    )
    .unwrap();
}

fn write_test(outfile: &mut fs::File, dir: &fs::DirEntry) {
    let dir = dir.path().canonicalize().unwrap();
    let path = dir.display();
    let separator = std::path::MAIN_SEPARATOR;
    let test_name = dir.file_name().unwrap().to_string_lossy();

    write!(
        outfile,
        r#"
#[test]
fn {test_name}() {{
    let mut arguments = shell_words::split(include_str!(r"{path}{separator}options.txt")).unwrap();
    if let Some(i) = arguments.iter().position(|s| s == "--custom-style") {{
        arguments[i + 1] = format!(r"{path}{separator}{{}}", arguments[i + 1]);
    }}
    let matches = args::build_opts().parse(&arguments).unwrap();
    let options = args::extract_make_svg_options(&matches).unwrap();
    let input = include_str!(r"{path}{separator}input.sgf");
    let expected = include_str!(r"{path}{separator}output.svg");

    let svg = make_svg(input, &options).unwrap();
    let mut buffer: Vec<u8> = vec![];
    svg.write_to(&mut buffer).unwrap();
    let result = std::str::from_utf8(&buffer).unwrap();

    assert_eq!(result, expected);
}}
        "#,
        test_name = test_name,
        path = path,
        separator = separator,
    )
    .unwrap();
}
