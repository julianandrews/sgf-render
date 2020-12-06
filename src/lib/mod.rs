mod goban;
pub use goban::{Goban, Stone, StoneColor};

use std::collections::HashSet;

use std::ops::Range;
use svg::node::element;

static BOARD_MARGIN: f64 = 0.64;
static LABEL_MARGIN: f64 = 0.8;

static FONT_FAMILY: &str = "Roboto";
static FONT_SIZE: f64 = 0.5;
static FONT_WEIGHT: usize = 700;

static LINE_COLOR: &str = "black";
static LINE_WIDTH: f64 = 0.03;
static MARKUP_WIDTH: f64 = 0.1;
static HOSHI_RADIUS: f64 = 0.09;

#[derive(Debug)]
pub struct MakeSvgOptions {
    pub goban_range: GobanRange,
    pub viewbox_width: f64,
    pub draw_board_labels: bool,
    pub draw_move_numbers: bool,
    pub draw_marks: bool,
    pub draw_triangles: bool,
    pub draw_circles: bool,
    pub draw_squares: bool,
    pub draw_selected: bool,
    pub draw_dimmed: bool,
    pub draw_labels: bool,
    pub draw_lines: bool,
    pub draw_arrows: bool,
    pub first_move_number: u64,
    pub style: GobanStyle,
}

pub fn make_svg(goban: &Goban, options: &MakeSvgOptions) -> Result<svg::Document, GobanSVGError> {
    let (x_range, y_range) = options.goban_range.get_ranges(goban)?;
    let width = x_range.end - x_range.start;
    let height = y_range.end - y_range.start;
    let label_margin = if options.draw_board_labels {
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
                .set("x", f64::from(x_range.start) - 0.5)
                .set("y", f64::from(y_range.start) - 0.5)
                .set("width", f64::from(width))
                .set("height", f64::from(height)),
        );

        let mut defs = element::Definitions::new()
            .add(clip_path)
            .add(options.style.linehead().set("id", "linehead"))
            .add(options.style.arrowhead().set("id", "arrowhead"));
        for element in options.style.defs() {
            defs = defs.add(element);
        }

        defs
    };
    let board_width = f64::from(width) - 1.0 + 2.0 * BOARD_MARGIN + label_margin;
    let board_height = f64::from(height) - 1.0 + 2.0 * BOARD_MARGIN + label_margin;

    let diagram = {
        let board = build_board(goban, options).set("clip-path", "url(#board-clip)");
        let board_view = {
            let offset = BOARD_MARGIN + label_margin;
            let board_view_transform = format!(
                "translate({}, {})",
                offset - f64::from(x_range.start),
                offset - f64::from(y_range.start)
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

        if options.draw_board_labels {
            diagram = diagram.add(draw_board_labels(
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

/// Draws a goban with squares of unit size.
fn build_board(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new()
        .set("id", "goban")
        .add(build_board_lines_group(goban, options))
        .add(build_stones_group(goban, options));

    if options.draw_move_numbers {
        group = group.add(build_move_numbers_group(goban, options));
    }
    if options.draw_marks {
        group = group.add(build_marks_group(goban, options));
    }
    if options.draw_triangles {
        group = group.add(build_triangles_group(goban, options));
    }
    if options.draw_circles {
        group = group.add(build_circles_group(goban, options));
    }
    if options.draw_squares {
        group = group.add(build_squares_group(goban, options));
    }
    if options.draw_selected {
        group = group.add(build_selected_group(goban, options));
    }
    if options.draw_dimmed {
        group = group.add(build_dimmed_group(goban, options));
    }
    if options.draw_labels {
        group = group.add(build_label_group(goban, options));
    }
    if options.draw_lines {
        group = group.add(build_line_group(goban, options));
    }
    if options.draw_arrows {
        group = group.add(build_arrow_group(goban, options));
    }

    group
}

fn build_board_lines_group(goban: &Goban, _options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new()
        .set("id", "lines")
        .set("stroke", LINE_COLOR)
        .set("stroke-width", LINE_WIDTH)
        .set("stroke-linecap", "square");

    // Draw lines
    for x in 0..goban.size.0 as usize {
        group = group.add(
            element::Line::new()
                .set("x1", x)
                .set("y1", 0)
                .set("x2", x)
                .set("y2", goban.size.1 - 1),
        );
    }
    for y in 0..goban.size.1 as usize {
        group = group.add(
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
    group.add(hoshi)
}

fn build_stones_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new()
        .set("id", "stones")
        .set("stroke", "none");
    for stone in goban.stones() {
        group = group.add(draw_stone(stone, options.style));
    }
    group
}

fn build_move_numbers_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new()
        .set("id", "move-numbers")
        .set("text-anchor", "middle");
    for (point, nums) in &goban.move_numbers {
        let n = *nums
            .last()
            .expect("Move numbers should never be an empty vector");
        if n >= options.first_move_number {
            let stone_color = goban.stones.get(&point).copied();
            let starting_num = (n - options.first_move_number) % 99 + 1;
            group = group.add(draw_move_number(
                point.0,
                point.1,
                starting_num,
                stone_color,
                options.style,
            ));
        }
    }
    group
}

fn build_marks_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new().set("id", "markup-marks");
    for point in &goban.marks {
        let stone_color = goban.stones.get(&point).copied();
        group = group.add(draw_mark(point.0, point.1, stone_color, options.style));
    }
    group
}

fn build_triangles_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new().set("id", "markup-triangles");
    for point in &goban.triangles {
        let stone_color = goban.stones.get(&point).copied();
        group = group.add(draw_triangle(point.0, point.1, stone_color, options.style));
    }
    group
}

fn build_circles_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new().set("id", "markup-circles");
    for point in &goban.circles {
        let stone_color = goban.stones.get(&point).copied();
        group = group.add(draw_circle(point.0, point.1, stone_color, options.style));
    }
    group
}

fn build_squares_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new().set("id", "markup-squares");
    for point in &goban.squares {
        let stone_color = goban.stones.get(&point).copied();
        group = group.add(draw_square(point.0, point.1, stone_color, options.style));
    }
    group
}

fn build_selected_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new().set("id", "markup-selected");
    for point in &goban.selected {
        let stone_color = goban.stones.get(&point).copied();
        group = group.add(draw_selected(point.0, point.1, stone_color, options.style));
    }
    group
}

fn build_dimmed_group(goban: &Goban, _options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new().set("id", "markup-dimmed");
    for point in &goban.dimmed {
        group = group.add(dim_square(point.0, point.1));
    }
    group
}

fn build_label_group(goban: &Goban, options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new().set("id", "markup-labels");
    for (point, text) in &goban.labels {
        let stone_color = goban.stones.get(&point).copied();
        group = group.add(draw_label(
            point.0,
            point.1,
            text,
            stone_color,
            options.style,
        ));
    }
    group
}

fn build_line_group(goban: &Goban, _options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new()
        .set("id", "markup-lines")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH)
        .set("marker-start", "url(#linehead)")
        .set("marker-end", "url(#linehead)");
    for &(p1, p2) in &goban.lines {
        group = group.add(
            element::Line::new()
                .set("x1", p1.0)
                .set("x2", p2.0)
                .set("y1", p1.1)
                .set("y2", p2.1),
        );
    }
    group
}

fn build_arrow_group(goban: &Goban, _options: &MakeSvgOptions) -> element::Group {
    let mut group = element::Group::new()
        .set("id", "markup-arrows")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH)
        .set("marker-end", "url(#arrowhead)");
    for &(p1, p2) in &goban.arrows {
        group = group.add(
            element::Line::new()
                .set("x1", p1.0)
                .set("x2", p2.0)
                .set("y1", p1.1)
                .set("y2", p2.1),
        );
    }

    group
}

/// Draw labels for the provided ranges.
///
/// Assumes lines are a unit apart, offset by `BOARD_MARGIN`.
/// Respects `LABEL_MARGIN`.
fn draw_board_labels(x_range: Range<u8>, y_range: Range<u8>, style: GobanStyle) -> element::Group {
    let mut row_labels = element::Group::new().set("text-anchor", "middle");
    let start = x_range.start;
    for x in x_range {
        let text = svg::node::Text::new(label_text(x));
        row_labels = row_labels.add(
            element::Text::new()
                .set("x", f64::from(x - start) + BOARD_MARGIN)
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
                .set("y", f64::from(end - y - 1) + BOARD_MARGIN)
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

fn draw_stone(stone: Stone, style: GobanStyle) -> impl svg::node::Node {
    match style {
        GobanStyle::Fancy => {
            let shadow = element::Circle::new()
                .set("cx", f64::from(stone.x) + 0.025)
                .set("cy", f64::from(stone.y) + 0.025)
                .set("r", 0.475)
                .set("fill", "black")
                .set("fill-opacity", 0.5);
            let fill = match stone.color {
                StoneColor::Black => "url(#black-stone-fill)",
                StoneColor::White => "url(#white-stone-fill)",
            };
            let stone_element = element::Circle::new()
                .set("cx", f64::from(stone.x) - 0.017)
                .set("cy", f64::from(stone.y) - 0.017)
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
                    .set("cx", f64::from(stone.x))
                    .set("cy", f64::from(stone.y))
                    .set("r", 0.48)
                    .set("stroke", "black")
                    .set("stroke-width", LINE_WIDTH)
                    .set("fill", fill),
            )
        }
    }
}

fn draw_move_number(
    x: u8,
    y: u8,
    n: u64,
    color: Option<StoneColor>,
    style: GobanStyle,
) -> impl svg::node::Node {
    let text = svg::node::Text::new(n.to_string());
    let text_element = element::Text::new()
        .set("x", f64::from(x))
        .set("y", f64::from(y))
        .set("dy", "0.35em")
        .set("fill", style.markup_color(color))
        .add(text);
    let mut group = element::Group::new();
    if color.is_none() {
        group = group.add(
            element::Rectangle::new()
                .set("fill", style.background_fill())
                .set("x", f64::from(x) - 0.4)
                .set("y", f64::from(y) - 0.4)
                .set("width", 0.8)
                .set("height", 0.8),
        );
    }

    group.add(text_element)
}

fn draw_mark(x: u8, y: u8, color: Option<StoneColor>, style: GobanStyle) -> impl svg::node::Node {
    element::Group::new()
        .set("stroke", style.markup_color(color))
        .set("stroke-width", MARKUP_WIDTH)
        .add(
            element::Line::new()
                .set("x1", f64::from(x) - 0.25)
                .set("x2", f64::from(x) + 0.25)
                .set("y1", f64::from(y) - 0.25)
                .set("y2", f64::from(y) + 0.25),
        )
        .add(
            element::Line::new()
                .set("x1", f64::from(x) - 0.25)
                .set("x2", f64::from(x) + 0.25)
                .set("y1", f64::from(y) + 0.25)
                .set("y2", f64::from(y) - 0.25),
        )
}

fn draw_triangle(
    x: u8,
    y: u8,
    color: Option<StoneColor>,
    style: GobanStyle,
) -> impl svg::node::Node {
    let triangle_radius = 0.45;
    element::Group::new()
        .set("stroke", style.markup_color(color))
        .set("fill", "none")
        .set("stroke-width", LINE_WIDTH)
        .add(element::Polygon::new().set(
            "points",
            format!(
                "{},{} {},{} {},{}",
                f64::from(x),
                f64::from(y) - triangle_radius,
                f64::from(x) - 0.866 * triangle_radius,
                f64::from(y) + 0.5 * triangle_radius,
                f64::from(x) + 0.866 * triangle_radius,
                f64::from(y) + 0.5 * triangle_radius,
            ),
        ))
}

fn draw_circle(x: u8, y: u8, color: Option<StoneColor>, style: GobanStyle) -> impl svg::node::Node {
    let radius = 0.25;
    element::Group::new()
        .set("stroke", style.markup_color(color))
        .set("fill", "none")
        .set("stroke-width", LINE_WIDTH)
        .add(
            element::Circle::new()
                .set("cx", f64::from(x))
                .set("cy", f64::from(y))
                .set("r", radius),
        )
}

fn draw_square(x: u8, y: u8, color: Option<StoneColor>, style: GobanStyle) -> impl svg::node::Node {
    let width = 0.55;
    element::Group::new()
        .set("stroke", style.markup_color(color))
        .set("fill", "none")
        .set("stroke-width", LINE_WIDTH)
        .add(
            element::Rectangle::new()
                .set("x", f64::from(x) - 0.5 * width)
                .set("y", f64::from(y) - 0.5 * width)
                .set("width", width)
                .set("height", width),
        )
}

fn draw_selected(
    x: u8,
    y: u8,
    color: Option<StoneColor>,
    style: GobanStyle,
) -> impl svg::node::Node {
    let width = 0.25;
    element::Group::new()
        .set("stroke", "none")
        .set("fill", style.selected_color(color))
        .set("stroke-width", LINE_WIDTH)
        .add(
            element::Rectangle::new()
                .set("x", f64::from(x) - 0.5 * width)
                .set("y", f64::from(y) - 0.5 * width)
                .set("width", width)
                .set("height", width),
        )
}

fn dim_square(x: u8, y: u8) -> impl svg::node::Node {
    let width = 1.00;
    element::Group::new()
        .set("stroke", "none")
        .set("fill", "black")
        .set("fill-opacity", 0.5)
        .set("shape-rendering", "crispEdges")
        .add(
            element::Rectangle::new()
                .set("x", f64::from(x) - 0.5 * width)
                .set("y", f64::from(y) - 0.5 * width)
                .set("width", width)
                .set("height", width),
        )
}

fn draw_label(
    x: u8,
    y: u8,
    text: &str,
    color: Option<StoneColor>,
    style: GobanStyle,
) -> impl svg::node::Node {
    let text = svg::node::Text::new(text.chars().take(2).collect::<String>());
    let text_element = element::Text::new()
        .set("x", f64::from(x))
        .set("y", f64::from(y))
        .set("text-anchor", "middle")
        .set("dy", "0.35em")
        .set("fill", style.markup_color(color))
        .add(text);
    let mut group = element::Group::new();
    if color.is_none() {
        group = group.add(
            element::Rectangle::new()
                .set("fill", style.background_fill())
                .set("x", f64::from(x) - 0.4)
                .set("y", f64::from(y) - 0.4)
                .set("width", 0.8)
                .set("height", 0.8),
        );
    }

    group.add(text_element)
}
#[derive(Debug, Clone, Copy)]
pub enum GobanStyle {
    Fancy,
    Simple,
    Minimalist,
}

impl GobanStyle {
    fn label_color(self) -> String {
        match self {
            Self::Fancy | Self::Simple => "#6e5840".to_string(),
            Self::Minimalist => "black".to_string(),
        }
    }

    fn background_fill(self) -> String {
        match self {
            Self::Fancy | Self::Simple => "#cfa87e".to_string(),
            Self::Minimalist => "white".to_string(),
        }
    }

    fn markup_color(self, color: Option<StoneColor>) -> String {
        match color {
            None | Some(StoneColor::White) => "black".to_string(),
            Some(StoneColor::Black) => "white".to_string(),
        }
    }

    fn selected_color(self, color: Option<StoneColor>) -> String {
        match self {
            Self::Minimalist => match color {
                Some(StoneColor::Black) => "white".to_string(),
                _ => "black".to_string(),
            },
            _ => "blue".to_string(),
        }
    }

    fn arrowhead(self) -> element::Marker {
        element::Marker::new()
            .set("markerWidth", 7)
            .set("markerHeight", 5)
            .set("refX", 7)
            .set("refY", 2.5)
            .set("orient", "auto")
            .add(element::Polygon::new().set("points", "0 0, 7 2.5, 0 5"))
    }

    fn linehead(self) -> element::Marker {
        element::Marker::new()
            .set("markerWidth", 4)
            .set("markerHeight", 4)
            .set("refX", 2)
            .set("refY", 2)
            .add(element::Circle::new().set("cx", 2).set("cy", 2).set("r", 2))
    }

    fn defs(self) -> Vec<impl svg::node::Node> {
        match self {
            Self::Fancy => {
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
                            .set("stop-color", "#ddd"),
                    )
                    .add(
                        element::Stop::new()
                            .set("offset", "100%")
                            .set("stop-color", "#bbb"),
                    );
                vec![black_stone_fill, white_stone_fill]
            }
            Self::Simple | Self::Minimalist => vec![],
        }
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
                let points: HashSet<(u8, u8)> = goban
                    .stones()
                    .map(|s| (s.x, s.y))
                    .chain(goban.marks.iter().copied())
                    .chain(goban.triangles.iter().copied())
                    .chain(goban.circles.iter().copied())
                    .chain(goban.squares.iter().copied())
                    .chain(goban.selected.iter().copied())
                    .chain(goban.labels.keys().copied())
                    .chain(goban.lines.iter().flat_map(|&(p1, p2)| vec![p1, p2]))
                    .chain(goban.arrows.iter().flat_map(|&(p1, p2)| vec![p1, p2]))
                    // Don't necessarily include dimmed points!
                    .collect();
                let x_start = points
                    .iter()
                    .map(|&(x, _)| x)
                    .min()
                    .unwrap_or(0)
                    .saturating_sub(1);
                let x_end = points
                    .iter()
                    .map(|&(x, _)| (x + 2).min(goban.size.0))
                    .max()
                    .unwrap_or(goban.size.0);
                let y_start = points
                    .iter()
                    .map(|&(_, y)| y)
                    .min()
                    .unwrap_or(0)
                    .saturating_sub(1);
                let y_end = points
                    .iter()
                    .map(|&(_, y)| (y + 2).min(goban.size.1))
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
