use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    generate_styles();
    generate_tests();
}

fn generate_styles() {
    let styles: Vec<PathBuf> = std::fs::read_dir("./resources/styles")
        .unwrap()
        .map(|direntry| direntry.unwrap().path().canonicalize().unwrap())
        .collect();

    let outfile_path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated_styles.rs");
    let mut outfile = fs::File::create(outfile_path).unwrap();
    writeln!(
        outfile,
        r#"/// Automatically generated styles.

use crate::render::GobanStyle;"#
    )
    .unwrap();

    writeln!(
        outfile,
        r#"
lazy_static::lazy_static! {{
    static ref STYLES: [GobanStyle; {}] = ["#,
        styles.len()
    )
    .unwrap();

    for path in &styles {
        writeln!(
            outfile,
            r#"        toml::from_str(include_str!(r"{path}")).unwrap(),"#,
            path = path.display(),
        )
        .unwrap();
    }

    writeln!(
        outfile,
        r#"    ];
}}"#
    )
    .unwrap();

    writeln!(
        outfile,
        r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum GeneratedStyle {{"#
    )
    .unwrap();
    for path in &styles {
        writeln!(outfile, "    {},", style_name(path)).unwrap();
    }
    writeln!(outfile, r#"}}"#).unwrap();

    writeln!(
        outfile,
        r#"
impl GeneratedStyle {{
    pub fn style(&self) -> &GobanStyle {{
        &STYLES[*self as usize]
    }}
}}"#
    )
    .unwrap();
}

fn style_name(path: &Path) -> String {
    let mut name = path
        .with_extension("")
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    name.get_mut(0..1).unwrap().make_ascii_uppercase();
    name
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

use clap::Parser;

use sgf_render::{{Goban, RenderArgs, OutputFormat, svg}};
"#,
    )
    .unwrap();
}

fn write_test(outfile: &mut fs::File, dir: &fs::DirEntry) {
    let dir = dir.path().canonicalize().unwrap();
    let path = dir.display();
    let separator = std::path::MAIN_SEPARATOR;
    let test_name = dir.file_name().unwrap().to_string_lossy();

    writeln!(
        outfile,
        r#"
#[test]
fn {test_name}() {{
    let mut arguments = shell_words::split(include_str!(r"{path}{separator}options.txt")).unwrap();
    if let Some(i) = arguments.iter().position(|s| s == "--custom-style") {{
        arguments[i + 1] = format!(r"{path}{separator}{{}}", arguments[i + 1]);
    }}
    arguments.insert(0, "sgf-render".to_string());
    let render_args = RenderArgs::parse_from(&arguments);
    let options = render_args.options(&OutputFormat::Svg).unwrap();
    let input = include_str!(r"{path}{separator}input.sgf");
    let expected = include_str!(r"{path}{separator}output.svg");

    let goban = Goban::from_sgf(input, &options.node_description, true).unwrap();
    let svg = svg::render(&goban, &options).unwrap();
    let mut buffer: Vec<u8> = vec![];
    svg.write_to(&mut buffer).unwrap();
    let result = std::str::from_utf8(&buffer).unwrap();

    assert_eq!(result, expected);
}}"#,
        test_name = test_name,
        path = path,
        separator = separator,
    )
    .unwrap();
}
