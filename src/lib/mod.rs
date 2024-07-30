mod args;
mod board_side;
mod errors;
mod generated_styles;
mod goban;
mod goban_range;
mod goban_style;
mod make_svg;
mod node_description;
mod query;
mod text;
mod traversal;

pub use args::{Command, MakeSvgArgs, OutputFormat, SgfRenderArgs};
pub use goban::Goban;
pub use make_svg::{make_svg, MakeSvgOptions};
pub use query::query;
pub use text::text_diagram;

fn board_label_text(x: u8) -> String {
    if x + b'A' < b'I' {
        ((x + b'A') as char).to_string()
    } else {
        ((x + b'B') as char).to_string() // skip 'I'
    }
}
