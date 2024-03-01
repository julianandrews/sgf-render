use std::error::Error;
use std::path::Path;

use clap::Parser;
use minidom::Element;

use sgf_render::{OutputFormat, SgfRenderArgs};

fn main() {
    let parsed_args = SgfRenderArgs::parse();
    let options = match parsed_args.make_svg_args.options() {
        Ok(options) => options,
        Err(e) => {
            eprintln!("Failed to read input: {}", e);
            std::process::exit(1);
        }
    };

    let input = match read_input(parsed_args.infile) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to read input: {}", e);
            std::process::exit(1);
        }
    };

    let svg = match sgf_render::make_svg(&input, &options) {
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

fn read_input<P: AsRef<Path>>(infile: Option<P>) -> Result<String, Box<dyn Error>> {
    let mut reader: Box<dyn std::io::Read> = match infile {
        Some(filename) => Box::new(std::io::BufReader::new(std::fs::File::open(&filename)?)),
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
    let mut out: Box<dyn std::io::Write> = match outfile {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };
    match format {
        OutputFormat::Svg => svg.write_to(&mut out)?,
        #[cfg(feature = "png")]
        OutputFormat::Png => save_png(out, svg)?,
    }
    Ok(())
}

#[cfg(feature = "png")]
fn save_png(mut out: Box<dyn std::io::Write>, svg: &Element) -> Result<(), Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![];
    svg.write_to(&mut buffer)?;
    let s = std::str::from_utf8(&buffer)?;
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_font_data(include_bytes!("../resources/Inter-Bold.ttf").to_vec());
    let tree = usvg::Tree::from_str(
        s,
        &usvg::Options {
            fontdb,
            ..usvg::Options::default()
        },
    )?;
    let pixmap_size = tree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, usvg::FitTo::Original, pixmap.as_mut()).ok_or(PngRenderFailure {})?;
    let data = pixmap.encode_png()?;
    out.write_all(&data)?;
    Ok(())
}

#[cfg(feature = "png")]
#[derive(Debug, Clone, Copy)]
struct PngRenderFailure {}

#[cfg(feature = "png")]
impl std::fmt::Display for PngRenderFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rendering png failed.")
    }
}

#[cfg(feature = "png")]
impl std::error::Error for PngRenderFailure {}
