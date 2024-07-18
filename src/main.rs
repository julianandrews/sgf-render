use std::error::Error;
use std::path::Path;

use clap::Parser;
use minidom::Element;

use sgf_render::{Command, OutputFormat, SgfRenderArgs};

fn main() {
    let parsed_args = SgfRenderArgs::parse();
    let input = match read_input(&parsed_args.infile) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to read input: {}", e);
            std::process::exit(1);
        }
    };

    match parsed_args.command {
        Some(Command::Query) => query(&input),
        None => render(&input, parsed_args),
    }
}

fn query(input: &str) {
    if let Err(e) = sgf_render::query(input) {
        eprintln!("Failed to parse SGF: {}", e);
        std::process::exit(1);
    }
}

fn render(input: &str, parsed_args: SgfRenderArgs) {
    let options = match parsed_args.make_svg_args.options() {
        Ok(options) => options,
        Err(e) => {
            eprintln!("Failed to parse arguments: {}", e);
            std::process::exit(1);
        }
    };
    let goban = match sgf_render::Goban::from_sgf(input, &options.node_description) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to generate goban: {}", e);
            std::process::exit(1);
        }
    };
    let svg = match sgf_render::make_svg(&goban, &options) {
        Ok(svg) => svg,
        Err(e) => {
            eprintln!("Failed to generate SVG: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = write_output(&svg, parsed_args.outfile, parsed_args.output_format) {
        eprintln!("Failed to write output: {}", e);
        std::process::exit(1);
    }
}

fn read_input<P: AsRef<Path>>(infile: &Option<P>) -> Result<String, Box<dyn Error>> {
    let mut reader: Box<dyn std::io::Read> = match infile {
        Some(filename) => Box::new(std::io::BufReader::new(std::fs::File::open(filename)?)),
        None => Box::new(std::io::stdin()),
    };
    let mut input = String::new();
    reader.read_to_string(&mut input)?;
    Ok(input)
}

fn write_output<P: AsRef<Path>>(
    svg: &Element,
    outfile: Option<P>,
    format: OutputFormat,
) -> Result<(), Box<dyn Error>> {
    let mut writer: Box<dyn std::io::Write> = match outfile {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };
    match format {
        OutputFormat::Svg => svg.write_to(&mut writer)?,
        #[cfg(feature = "png")]
        OutputFormat::Png => save_png(writer, svg)?,
    }
    Ok(())
}

#[cfg(feature = "png")]
fn save_png(mut writer: Box<dyn std::io::Write>, svg: &Element) -> Result<(), Box<dyn Error>> {
    let tree = {
        let mut buffer: Vec<u8> = vec![];
        svg.write_to(&mut buffer)?;
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_font_data(include_bytes!("../resources/Inter-Bold.ttf").to_vec());
        usvg::Tree::from_data(&buffer, &usvg::Options::default(), &fontdb)?
    };
    let data = {
        let pixmap_size = tree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
        pixmap.encode_png()?
    };

    writer.write_all(&data)?;
    Ok(())
}
