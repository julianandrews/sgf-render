use std::env;
use std::fs::{DirEntry, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let outfile_path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated_tests.rs");
    let mut outfile = File::create(outfile_path).unwrap();
    write_header(&mut outfile);

    let test_data = std::fs::read_dir("./tests/data").unwrap();
    for result in test_data {
        let entry = result.unwrap();
        if entry.metadata().unwrap().is_dir() {
            write_test(&mut outfile, &entry);
        }
    }
}

fn write_header(outfile: &mut File) {
    write!(
        outfile,
        r#"/// Automatically generated tests.

use sgf_render::make_svg;
"#,
    )
    .unwrap();
}

fn write_test(outfile: &mut File, dir: &DirEntry) {
    let dir = dir.path().canonicalize().unwrap();
    let path = dir.display();
    let test_name = dir.file_name().unwrap().to_string_lossy();

    write!(
        outfile,
        r#"
#[test]
fn {test_name}() {{
    let input = include_str!("{path}/input.sgf");
    let options = toml::from_str(include_str!("{path}/options.toml")).unwrap();
    let expected = include_str!("{path}/output.svg");

    let svg = make_svg(input, &options).unwrap();
    let mut buffer: Vec<u8> = vec![];
    svg::write(&mut buffer, &svg).unwrap();
    let result = std::str::from_utf8(&buffer).unwrap();

    assert_eq!(result, expected);
}}
        "#,
        test_name = test_name,
        path = path,
    )
    .unwrap();
}
