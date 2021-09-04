use minidom::Element;
use std::ops::Range;

use super::{Goban, GobanRange, GobanSVGError, GobanStyle, NodeDescription, Stone, StoneColor};

pub static NAMESPACE: &'static str = "http://www.w3.org/2000/svg";

static BOARD_MARGIN: f64 = 0.64;
static LABEL_MARGIN: f64 = 0.8;

static FONT_FAMILY: &str = "Roboto";
static FONT_SIZE: f64 = 0.5;
static FONT_WEIGHT: usize = 700;

#[derive(Debug, Clone)]
pub struct MakeSvgOptions {
    pub node_description: NodeDescription,
    pub goban_range: GobanRange,
    pub style: GobanStyle,
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
}

pub fn make_svg(sgf: &str, options: &MakeSvgOptions) -> Result<Element, GobanSVGError> {
    let collection = sgf_parse::go::parse(sgf)?;
    let goban = Goban::from_node_in_collection(options.node_description, &collection)?;
    let (x_range, y_range) = options.goban_range.get_ranges(&goban)?;
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
        let clip_path = Element::builder("clipPath", NAMESPACE)
            .attr("id", "board-clip")
            .append(
                Element::builder("rect", NAMESPACE)
                    .attr("x", (f64::from(x_range.start) - 0.5).to_string())
                    .attr("y", (f64::from(y_range.start) - 0.5).to_string())
                    .attr("width", width.to_string())
                    .attr("height", height.to_string())
                    .build(),
            )
            .build();
        Element::builder("defs", NAMESPACE)
            .append(clip_path)
            .append_all(options.style.defs()?)
            .build()
    };
    let board_width = f64::from(width) - 1.0 + 2.0 * BOARD_MARGIN + label_margin;
    let board_height = f64::from(height) - 1.0 + 2.0 * BOARD_MARGIN + label_margin;

    let diagram = {
        let board = build_board(&goban, options);
        let board_view = {
            let offset = BOARD_MARGIN + label_margin;
            let board_view_transform = format!(
                "translate({}, {})",
                offset - f64::from(x_range.start),
                offset - f64::from(y_range.start)
            );
            Element::builder("g", NAMESPACE)
                .attr("id", "board-view")
                .attr("transform", board_view_transform)
                .append(board)
                .build()
        };

        let scale = options.viewbox_width / board_width;
        let transform = format!("scale({}, {})", scale, scale);
        let mut diagram_builder = Element::builder("g", NAMESPACE)
            .attr("id", "diagram")
            .attr("transform", transform)
            .append(board_view);

        if options.draw_board_labels {
            diagram_builder = diagram_builder.append(draw_board_labels(
                x_range,
                goban.size.1 - height - y_range.start + 1..goban.size.1 - y_range.start + 1,
                &options.style,
            ));
        }

        diagram_builder.build()
    };

    let background = Element::builder("rect", NAMESPACE)
        .attr("fill", options.style.background_fill())
        .attr("height", "100%")
        .attr("width", "100%")
        .attr("x", "0")
        .attr("y", "0")
        .build();

    let viewbox_height = options.viewbox_width * board_height / board_width;
    let viewbox_attr = format!("0 0 {} {}", options.viewbox_width, viewbox_height);
    let svg = Element::builder("svg", NAMESPACE)
        .attr("viewBox", viewbox_attr)
        .attr("width", options.viewbox_width.to_string())
        .attr("font-size", FONT_SIZE.to_string())
        .attr("font-family", FONT_FAMILY)
        .attr("font-weight", FONT_WEIGHT)
        .append(definitions)
        .append(background)
        .append(diagram)
        .build();
    Ok(svg)
}

/// Draws a goban with squares of unit size.
fn build_board(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "goban")
        .attr("clip-path", "url(#board-clip)")
        .append(build_board_lines_group(goban, options))
        .append(build_stones_group(goban, options));

    if options.draw_move_numbers {
        group_builder = group_builder.append(build_move_numbers_group(goban, options));
    }
    if options.draw_marks {
        group_builder = group_builder.append(build_marks_group(goban, options));
    }
    if options.draw_triangles {
        group_builder = group_builder.append(build_triangles_group(goban, options));
    }
    if options.draw_circles {
        group_builder = group_builder.append(build_circles_group(goban, options));
    }
    if options.draw_squares {
        group_builder = group_builder.append(build_squares_group(goban, options));
    }
    if options.draw_selected {
        group_builder = group_builder.append(build_selected_group(goban, options));
    }
    if options.draw_dimmed {
        group_builder = group_builder.append(build_dimmed_group(goban, options));
    }
    if options.draw_labels {
        group_builder = group_builder.append(build_label_group(goban, options));
    }
    if options.draw_lines {
        group_builder = group_builder.append(build_line_group(goban, options));
    }
    if options.draw_arrows {
        group_builder = group_builder.append(build_arrow_group(goban, options));
    }

    group_builder.build()
}

fn build_board_lines_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "lines")
        .attr("stroke", options.style.line_color())
        .attr("stroke-width", options.style.line_width().to_string())
        .attr("stroke-linecap", "square");

    // Draw lines
    for x in 0..goban.size.0 as usize {
        group_builder = group_builder.append(
            Element::builder("line", NAMESPACE)
                .attr("x1", x.to_string())
                .attr("y1", 0.to_string())
                .attr("x2", x.to_string())
                .attr("y2", (goban.size.1 - 1).to_string()),
        );
    }
    for y in 0..goban.size.1 as usize {
        group_builder = group_builder.append(
            Element::builder("line", NAMESPACE)
                .attr("x1", 0.to_string())
                .attr("y1", y.to_string())
                .attr("x2", (goban.size.0 - 1).to_string())
                .attr("y2", y.to_string()),
        );
    }

    // Draw hoshi
    let hoshi_radius = options.style.hoshi_radius();
    let mut hoshi = Element::builder("g", NAMESPACE)
        .attr("id", "hoshi")
        .attr("stroke", "none")
        .attr("fill", options.style.line_color());
    for &(x, y) in goban.hoshi_points() {
        hoshi = hoshi.append(
            Element::builder("circle", NAMESPACE)
                .attr("cx", x.to_string())
                .attr("cy", y.to_string())
                .attr("r", hoshi_radius.to_string()),
        );
    }
    group_builder.append(hoshi).build()
}

fn build_stones_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "stones")
        .attr("stroke", "none");
    for stone in goban.stones() {
        group_builder = group_builder.append(draw_stone(stone, &options.style));
    }
    group_builder.build()
}

fn build_move_numbers_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "move-numbers")
        .attr("text-anchor", "middle");
    let mut move_numbers: Vec<_> = goban.move_numbers.iter().collect();
    move_numbers.sort_by_key(|(_, nums)| nums.iter().max());
    for (point, nums) in &move_numbers {
        let n = *nums
            .last()
            .expect("Move numbers should never be an empty vector");
        if n >= options.first_move_number {
            let stone_color = goban.stones.get(point).copied();
            let starting_num = (n - options.first_move_number) % 99 + 1;
            group_builder = group_builder.append(draw_move_number(
                point.0,
                point.1,
                starting_num,
                stone_color,
                &options.style,
            ));
        }
    }
    group_builder.build()
}

fn build_marks_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-marks");
    let mut marks: Vec<_> = goban.marks.iter().collect();
    marks.sort();
    for point in marks {
        let stone_color = goban.stones.get(point).copied();
        group_builder =
            group_builder.append(draw_mark(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_triangles_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-triangles");
    let mut triangles: Vec<_> = goban.triangles.iter().collect();
    triangles.sort();
    for point in triangles {
        let stone_color = goban.stones.get(point).copied();
        group_builder =
            group_builder.append(draw_triangle(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_circles_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-circles");
    let mut circles: Vec<_> = goban.circles.iter().collect();
    circles.sort();
    for point in circles {
        let stone_color = goban.stones.get(point).copied();
        group_builder =
            group_builder.append(draw_circle(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_squares_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-squares");
    let mut squares: Vec<_> = goban.squares.iter().collect();
    squares.sort();
    for point in squares {
        let stone_color = goban.stones.get(point).copied();
        group_builder =
            group_builder.append(draw_square(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_selected_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-selected");
    let mut selected: Vec<_> = goban.selected.iter().collect();
    selected.sort();
    for point in selected {
        let stone_color = goban.stones.get(point).copied();
        group_builder =
            group_builder.append(draw_selected(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_dimmed_group(goban: &Goban, _options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-dimmed");
    let mut dimmed: Vec<_> = goban.dimmed.iter().collect();
    dimmed.sort();
    for point in dimmed {
        group_builder = group_builder.append(dim_square(point.0, point.1));
    }
    group_builder.build()
}

fn build_label_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-labels");
    let mut labels: Vec<_> = goban.labels.iter().collect();
    labels.sort();
    for (point, text) in labels {
        let stone_color = goban.stones.get(point).copied();
        group_builder = group_builder.append(draw_label(
            point.0,
            point.1,
            text,
            stone_color,
            &options.style,
        ));
    }
    group_builder.build()
}

fn build_line_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "markup-lines")
        .attr("stroke", "black")
        .attr("stroke-width", options.style.line_width().to_string())
        .attr("marker-start", "url(#linehead)")
        .attr("marker-end", "url(#linehead)");
    let mut lines: Vec<_> = goban.lines.iter().collect();
    lines.sort();
    for &(p1, p2) in lines {
        group_builder = group_builder.append(
            Element::builder("line", NAMESPACE)
                .attr("x1", p1.0)
                .attr("x2", p2.0)
                .attr("y1", p1.1)
                .attr("y2", p2.1),
        );
    }
    group_builder.build()
}

fn build_arrow_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "markup-arrows")
        .attr("stroke", "black")
        .attr("stroke-width", options.style.line_width().to_string())
        .attr("marker-end", "url(#arrowhead)");
    let mut arrows: Vec<_> = goban.arrows.iter().collect();
    arrows.sort();
    for &(p1, p2) in arrows {
        group_builder = group_builder.append(
            Element::builder("line", NAMESPACE)
                .attr("x1", p1.0)
                .attr("x2", p2.0)
                .attr("y1", p1.1)
                .attr("y2", p2.1),
        );
    }

    group_builder.build()
}

/// Draw labels for the provided ranges.
///
/// Assumes lines are a unit apart, offset by `BOARD_MARGIN`.
/// Respects `LABEL_MARGIN`.
fn draw_board_labels(x_range: Range<u8>, y_range: Range<u8>, style: &GobanStyle) -> Element {
    let row_labels = {
        let mut builder = Element::builder("g", NAMESPACE).attr("text-anchor", "middle");
        let start = x_range.start;
        for x in x_range {
            builder = builder.append(
                Element::builder("text", NAMESPACE)
                    .attr("x", (f64::from(x - start) + BOARD_MARGIN).to_string())
                    .attr("y", "0.0")
                    .append(label_text(x))
                    .build(),
            );
        }
        builder.build()
    };
    let column_labels = {
        let mut builder = Element::builder("g", NAMESPACE).attr("text-anchor", "end");
        let end = y_range.end;
        for y in y_range {
            builder = builder.append(
                Element::builder("text", NAMESPACE)
                    .attr("x", "0.0")
                    .attr("y", (f64::from(end - y - 1) + BOARD_MARGIN).to_string())
                    .attr("dy", "0.35em")
                    .append(y.to_string())
                    .build(),
            );
        }
        builder.build()
    };

    let transform = format!("translate({}, {})", LABEL_MARGIN, LABEL_MARGIN);
    Element::builder("g", NAMESPACE)
        .attr("id", "board-labels")
        .attr("fill", style.label_color())
        .attr("transform", transform)
        .append(row_labels)
        .append(column_labels)
        .build()
}

fn label_text(x: u8) -> String {
    if x + b'A' < b'I' {
        ((x + b'A') as char).to_string()
    } else {
        ((x + b'B') as char).to_string() // skip 'I'
    }
}

fn draw_stone(stone: Stone, style: &GobanStyle) -> Element {
    let mut circle_builder = Element::builder("circle", NAMESPACE)
        .attr("cx", f64::from(stone.x).to_string())
        .attr("cy", f64::from(stone.y).to_string())
        .attr("r", "0.48");
    if let Some(stroke) = style.stone_stroke(stone.color) {
        circle_builder = circle_builder
            .attr("stroke", stroke)
            .attr("stroke-width", style.line_width().to_string())
    }
    if let Some(fill) = style.stone_fill(stone.color) {
        circle_builder = circle_builder.attr("fill", fill);
    }
    circle_builder.build()
}

fn draw_move_number(
    x: u8,
    y: u8,
    n: u64,
    color: Option<StoneColor>,
    style: &GobanStyle,
) -> Element {
    // let text = svg::node::Text::new(n.to_string());
    let text_element = Element::builder("text", NAMESPACE)
        .attr("x", x.to_string())
        .attr("y", y.to_string())
        .attr("dy", "0.35em")
        .attr("fill", style.markup_color(color))
        .append(n.to_string());
    let mut group_builder = Element::builder("g", NAMESPACE);
    if color.is_none() {
        group_builder = group_builder.append(
            Element::builder("rect", NAMESPACE)
                .attr("fill", style.background_fill())
                .attr("x", (f64::from(x) - 0.4).to_string())
                .attr("y", (f64::from(y) - 0.4).to_string())
                .attr("width", "0.8")
                .attr("height", "0.8"),
        );
    }

    group_builder.append(text_element).build()
}

fn draw_mark(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    Element::builder("g", NAMESPACE)
        .attr("stroke", style.markup_color(color))
        .attr("stroke-width", style.markup_stroke_width().to_string())
        .append(
            Element::builder("line", NAMESPACE)
                .attr("x1", (f64::from(x) - 0.25).to_string())
                .attr("x2", (f64::from(x) + 0.25).to_string())
                .attr("y1", (f64::from(y) - 0.25).to_string())
                .attr("y2", (f64::from(y) + 0.25).to_string()),
        )
        .append(
            Element::builder("line", NAMESPACE)
                .attr("x1", (f64::from(x) - 0.25).to_string())
                .attr("x2", (f64::from(x) + 0.25).to_string())
                .attr("y1", (f64::from(y) + 0.25).to_string())
                .attr("y2", (f64::from(y) - 0.25).to_string()),
        )
        .build()
}

fn draw_triangle(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let triangle_radius = 0.45;
    let points = format!(
        "{},{} {},{} {},{}",
        f64::from(x),
        f64::from(y) - triangle_radius,
        f64::from(x) - 0.866 * triangle_radius,
        f64::from(y) + 0.5 * triangle_radius,
        f64::from(x) + 0.866 * triangle_radius,
        f64::from(y) + 0.5 * triangle_radius,
    );
    Element::builder("g", NAMESPACE)
        .attr("stroke", style.markup_color(color))
        .attr("fill", "none")
        .attr("stroke-width", style.line_width().to_string())
        .append(Element::builder("polygon", NAMESPACE).attr("points", points))
        .build()
}

fn draw_circle(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let radius = 0.25;
    Element::builder("g", NAMESPACE)
        .attr("stroke", style.markup_color(color))
        .attr("fill", "none")
        .attr("stroke-width", style.line_width().to_string())
        .append(
            Element::builder("circle", NAMESPACE)
                .attr("cx", f64::from(x).to_string())
                .attr("cy", f64::from(y).to_string())
                .attr("r", radius.to_string()),
        )
        .build()
}

fn draw_square(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let width = 0.55;
    Element::builder("g", NAMESPACE)
        .attr("stroke", style.markup_color(color))
        .attr("fill", "none")
        .attr("stroke-width", style.line_width().to_string())
        .append(
            Element::builder("rect", NAMESPACE)
                .attr("x", (f64::from(x) - 0.5 * width).to_string())
                .attr("y", (f64::from(y) - 0.5 * width).to_string())
                .attr("width", width.to_string())
                .attr("height", width.to_string()),
        )
        .build()
}

fn draw_selected(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let width = 0.25;
    Element::builder("g", NAMESPACE)
        .attr("stroke", "none")
        .attr("fill", style.selected_color(color))
        .attr("stroke-width", style.line_width().to_string())
        .append(
            Element::builder("rect", NAMESPACE)
                .attr("x", (f64::from(x) - 0.5 * width).to_string())
                .attr("y", (f64::from(y) - 0.5 * width).to_string())
                .attr("width", width.to_string())
                .attr("height", width.to_string()),
        )
        .build()
}

fn dim_square(x: u8, y: u8) -> Element {
    let width = 1.00;
    Element::builder("g", NAMESPACE)
        .attr("stroke", "none")
        .attr("fill", "black")
        .attr("fill-opacity", "0.5")
        .attr("shape-rendering", "crispEdges")
        .append(
            Element::builder("rect", NAMESPACE)
                .attr("x", (f64::from(x) - 0.5 * width).to_string())
                .attr("y", (f64::from(y) - 0.5 * width).to_string())
                .attr("width", width.to_string())
                .attr("height", width.to_string()),
        )
        .build()
}

fn draw_label(x: u8, y: u8, text: &str, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let text = text.chars().take(2).collect::<String>();
    let text_element = Element::builder("text", NAMESPACE)
        .attr("x", f64::from(x).to_string())
        .attr("y", f64::from(y).to_string())
        .attr("text-anchor", "middle")
        .attr("dy", "0.35em")
        .attr("fill", style.markup_color(color))
        .append(text);
    let mut group_builder = Element::builder("g", NAMESPACE);
    if color.is_none() {
        group_builder = group_builder.append(
            Element::builder("rect", NAMESPACE)
                .attr("fill", style.background_fill())
                .attr("x", (f64::from(x) - 0.4).to_string())
                .attr("y", (f64::from(y) - 0.4).to_string())
                .attr("width", "0.8")
                .attr("height", "0.8"),
        );
    }

    group_builder.append(text_element).build()
}
