mod goban;
pub use goban::{Goban, Stone, StoneColor};

use std::ops::Range;
use svg::node::element;

static BOARD_MARGIN: f64 = 0.64;
static LABEL_MARGIN: f64 = 0.8;

static FONT_FAMILY: &str = "Roboto";
static FONT_SIZE: f64 = 0.5;
static FONT_WEIGHT: usize = 700;

static LINE_COLOR: &str = "black";
static LINE_WIDTH: f64 = 0.045;
static HOSHI_RADIUS: f64 = 0.09;

#[derive(Debug)]
pub struct MakeSvgOptions {
    pub goban_range: GobanRange,
    pub viewbox_width: f64,
    pub render_labels: bool,
    pub render_move_numbers: bool,
    pub first_move_number: u64,
    pub style: GobanStyle,
}

pub fn make_svg(goban: &Goban, options: &MakeSvgOptions) -> Result<svg::Document, GobanSVGError> {
    let (x_range, y_range) = options.goban_range.get_ranges(goban)?;
    let width = x_range.end - x_range.start;
    let height = y_range.end - y_range.start;
    let label_margin = if options.render_labels {
        if width > 25 || height > 99 {
            return Err(GobanSVGError::UnlabellableRange);
        }
        LABEL_MARGIN
    } else {
        0.0
    };

    let definitions = {
        let clip_path = element::ClipPath::new().set("id", "board-clip").add(
            element::Rectangle::new()
                .set("x", x_range.start as f64 - 0.5)
                .set("y", y_range.start as f64 - 0.5)
                .set("width", width as f64)
                .set("height", height as f64),
        );

        let mut defs = element::Definitions::new().add(clip_path);
        for element in options.style.defs() {
            defs = defs.add(element);
        }

        defs
    };
    let board_width = width as f64 - 1.0 + 2.0 * BOARD_MARGIN + label_margin;
    let board_height = height as f64 - 1.0 + 2.0 * BOARD_MARGIN + label_margin;

    let diagram = {
        let board = draw_board(goban, options).set("clip-path", "url(#board-clip)");
        let board_view = {
            let offset = BOARD_MARGIN + label_margin;
            let board_view_transform = format!(
                "translate({}, {})",
                offset - x_range.start as f64,
                offset - y_range.start as f64
            );
            element::Group::new()
                .set("id", "board-view")
                .add(board)
                .set("transform", board_view_transform)
        };

        let scale = options.viewbox_width / board_width;
        let transform = format!("scale({}, {})", scale, scale);

        let mut diagram = element::Group::new()
            .set("id", "diagram")
            .add(board_view)
            .set("transform", transform);

        if options.render_labels {
            diagram = diagram.add(draw_labels(
                x_range,
                goban.size.1 - height - y_range.start + 1..goban.size.1 - y_range.start + 1,
                options.style,
            ));
        }

        diagram
    };

    let background = element::Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", options.style.background_fill());

    let viewbox_height = options.viewbox_width * board_height / board_width;
    Ok(svg::Document::new()
        .set("viewBox", (0.0, 0.0, options.viewbox_width, viewbox_height))
        .set("width", options.viewbox_width)
        .set("font-size", FONT_SIZE)
        .set("font-family", FONT_FAMILY)
        .set("font-weight", FONT_WEIGHT)
        .add(definitions)
        .add(background)
        .add(diagram))
}

/// Draws a goban of with squares of unit size.
fn draw_board(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    // TODO: Add support for markup and comments
    let mut lines = element::Group::new()
        .set("id", "lines")
        .set("stroke", LINE_COLOR)
        .set("stroke-width", LINE_WIDTH)
        .set("stroke-linecap", "square");

    // Draw lines
    for x in 0..goban.size.0 as usize {
        lines = lines.add(
            element::Line::new()
                .set("x1", x)
                .set("y1", 0)
                .set("x2", x)
                .set("y2", goban.size.1 - 1),
        );
    }
    for y in 0..goban.size.1 as usize {
        lines = lines.add(
            element::Line::new()
                .set("x1", 0)
                .set("y1", y)
                .set("x2", goban.size.0 - 1)
                .set("y2", y),
        );
    }

    // Draw hoshi
    let mut hoshi = element::Group::new()
        .set("id", "hoshi")
        .set("stroke", "none")
        .set("fill", LINE_COLOR);
    for &(x, y) in goban.hoshi_points() {
        hoshi = hoshi.add(
            element::Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", HOSHI_RADIUS),
        );
    }
    lines = lines.add(hoshi);

    // Draw stones
    let mut stones = element::Group::new()
        .set("id", "stones")
        .set("stroke", "none");
    for stone in goban.stones() {
        stones = stones.add(options.style.draw_stone(stone));
    }
    if options.render_move_numbers {
        // TODO: Indicate older moves on the side.
        for (point, nums) in &goban.move_numbers {
            let n = *nums
                .last()
                .expect("Move numbers should never be an empty vector");
            if n >= options.first_move_number {
                let stone_color = goban.stones.get(&point).map(|&s| s);
                let starting_num = (n - options.first_move_number) % 99 + 1;
                stones = stones.add(options.style.draw_move_number(
                    point.0,
                    point.1,
                    starting_num,
                    stone_color,
                ));
            }
        }
    }

    element::Group::new()
        .set("id", "goban")
        .add(lines)
        .add(stones)
}

/// Draw labels for the provided ranges.
///
/// Assumes lines are a unit apart, offset by BOARD_MARGIN.
/// Respects LABEL_MARGIN.
fn draw_labels(x_range: Range<u8>, y_range: Range<u8>, style: GobanStyle) -> element::Group {
    let mut row_labels = element::Group::new().set("text-anchor", "middle");
    let start = x_range.start;
    for x in x_range {
        let text = svg::node::Text::new(label_text(x));
        row_labels = row_labels.add(
            element::Text::new()
                .set("x", (x - start) as f64 + BOARD_MARGIN)
                .set("y", 0.0)
                .add(text),
        );
    }
    let mut column_labels = element::Group::new().set("text-anchor", "end");
    let end = y_range.end;
    for y in y_range {
        let text = svg::node::Text::new(y.to_string());
        column_labels = column_labels.add(
            element::Text::new()
                .set("x", 0.0)
                .set("y", (end - y - 1) as f64 + BOARD_MARGIN)
                .set("dy", "0.35em")
                .add(text),
        );
    }

    let transform = format!("translate({}, {})", LABEL_MARGIN, LABEL_MARGIN);
    element::Group::new()
        .set("id", "board-labels")
        .set("fill", style.label_color())
        .set("transform", transform)
        .add(row_labels)
        .add(column_labels)
}

fn label_text(x: u8) -> String {
    if x + b'A' < b'I' {
        ((x + b'A') as char).to_string()
    } else {
        ((x + b'B') as char).to_string() // skip 'I'
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GobanStyle {
    Default,
    Simple,
    Minimalist,
}

impl GobanStyle {
    fn label_color(&self) -> String {
        match self {
            Self::Default | Self::Simple => "#6e5840".to_string(),
            Self::Minimalist => "black".to_string(),
        }
    }

    fn background_fill(&self) -> String {
        match self {
            Self::Default | Self::Simple => "#cfa87e".to_string(),
            Self::Minimalist => "white".to_string(),
        }
    }

    fn defs(&self) -> Vec<impl svg::node::Node> {
        match self {
            Self::Default => {
                let black_stone_fill = element::RadialGradient::new()
                    .set("id", "black-stone-fill")
                    .set("cx", "35%")
                    .set("cy", "35%")
                    .add(
                        element::Stop::new()
                            .set("offset", "0%")
                            .set("stop-color", "#666"),
                    )
                    .add(
                        element::Stop::new()
                            .set("offset", "100%")
                            .set("stop-color", "black"),
                    );
                let white_stone_fill = element::RadialGradient::new()
                    .set("id", "white-stone-fill")
                    .set("cx", "35%")
                    .set("cy", "35%")
                    .add(
                        element::Stop::new()
                            .set("offset", "0%")
                            .set("stop-color", "#eee"),
                    )
                    .add(
                        element::Stop::new()
                            .set("offset", "30%")
                            .set("stop-color", "#b9b9b9"),
                    )
                    .add(
                        element::Stop::new()
                            .set("offset", "100%")
                            .set("stop-color", "#aaa"),
                    );
                vec![black_stone_fill, white_stone_fill]
            }
            Self::Simple => vec![],
            Self::Minimalist => vec![],
        }
    }

    fn draw_stone(&self, stone: Stone) -> impl svg::node::Node {
        let stone_group = match self {
            GobanStyle::Default => {
                let shadow = element::Circle::new()
                    .set("cx", stone.x as f64 + 0.025)
                    .set("cy", stone.y as f64 + 0.025)
                    .set("r", 0.475)
                    .set("fill", "black")
                    .set("fill-opacity", 0.5);
                let fill = match stone.color {
                    StoneColor::Black => "url(#black-stone-fill)",
                    StoneColor::White => "url(#white-stone-fill)",
                };
                let stone_element = element::Circle::new()
                    .set("cx", stone.x as f64 - 0.017)
                    .set("cy", stone.y as f64 - 0.017)
                    .set("r", 0.475)
                    .set("fill", fill);
                element::Group::new().add(shadow).add(stone_element)
            }
            GobanStyle::Minimalist | GobanStyle::Simple => {
                let fill = match stone.color {
                    StoneColor::Black => "black",
                    StoneColor::White => "white",
                };
                element::Group::new().add(
                    element::Circle::new()
                        .set("cx", stone.x as f64)
                        .set("cy", stone.y as f64)
                        .set("r", 0.48)
                        .set("stroke", "black")
                        .set("stroke-width", LINE_WIDTH)
                        .set("fill", fill),
                )
            }
        };

        stone_group
    }

    fn draw_move_number(
        &self,
        x: u8,
        y: u8,
        n: u64,
        color: Option<StoneColor>,
    ) -> impl svg::node::Node {
        let text = svg::node::Text::new(n.to_string());
        let fill = match color {
            None | Some(StoneColor::White) => "black",
            Some(StoneColor::Black) => "white",
        };
        let text_element = element::Text::new()
            .set("x", x as f64)
            .set("y", y as f64)
            .set("text-anchor", "middle")
            .set("dy", "0.35em")
            // .set("dominant-baseline", "central")
            .set("fill", fill)
            .add(text);
        let mut group = element::Group::new();
        if color.is_none() {
            group = group.add(
                element::Rectangle::new()
                    .set("fill", self.background_fill())
                    .set("x", x as f64 - 0.4)
                    .set("y", y as f64 - 0.4)
                    .set("width", 0.8)
                    .set("height", 0.8),
            );
        }

        group.add(text_element)
    }
}

#[derive(Debug)]
pub enum GobanRange {
    ShrinkWrap,
    FullBoard,
    Ranged(Range<u8>, Range<u8>),
}

impl GobanRange {
    fn get_ranges(&self, goban: &Goban) -> Result<(Range<u8>, Range<u8>), GobanSVGError> {
        match self {
            Self::FullBoard => Ok((0..goban.size.0, 0..goban.size.1)),
            Self::ShrinkWrap => {
                let x_start = goban
                    .stones()
                    .map(|s| s.x)
                    .min()
                    .unwrap_or(0)
                    .saturating_sub(1);
                let x_end = goban
                    .stones()
                    .map(|s| s.x + 2)
                    .max()
                    .unwrap_or(goban.size.0);
                let y_start = goban
                    .stones()
                    .map(|s| s.y)
                    .min()
                    .unwrap_or(0)
                    .saturating_sub(1);
                let y_end = goban
                    .stones()
                    .map(|s| s.y + 2)
                    .max()
                    .unwrap_or(goban.size.1);
                Ok((x_start..x_end, y_start..y_end))
            }
            Self::Ranged(a, b) => {
                if a.end > goban.size.0 || b.end > goban.size.1 {
                    Err(GobanSVGError::InvalidRange)
                } else {
                    Ok((a.clone(), b.clone()))
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum GobanSVGError {
    InvalidRange,
    UnlabellableRange,
}

impl std::fmt::Display for GobanSVGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRange => write!(f, "Invalid range to render in goban."),
            Self::UnlabellableRange => write!(f, "Range too large for use with labels."),
        }
    }
}

impl std::error::Error for GobanSVGError {}
