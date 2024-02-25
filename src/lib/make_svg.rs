use std::collections::{HashMap, HashSet};
use std::ops::Range;

use minidom::Element;

use super::{Goban, GobanRange, GobanStyle, MakeSvgError, NodeDescription, Stone, StoneColor};

pub static NAMESPACE: &str = "http://www.w3.org/2000/svg";

static BOARD_MARGIN: f64 = 0.64;
static LABEL_MARGIN: f64 = 0.8;
static REPEATED_MOVES_MARGIN: f64 = 0.32;

static FONT_FAMILY: &str = "Roboto";
static FONT_SIZE: f64 = 0.5;
static FONT_WEIGHT: usize = 700;

#[derive(Debug, Clone)]
pub struct MakeSvgOptions {
    pub node_description: NodeDescription,
    pub goban_range: GobanRange,
    pub style: GobanStyle,
    pub viewbox_width: f64,
    pub label_sides: HashSet<BoardSide>,
    pub draw_move_numbers: bool,
    pub first_move_number: u64,
    pub draw_marks: bool,
    pub draw_triangles: bool,
    pub draw_circles: bool,
    pub draw_squares: bool,
    pub draw_selected: bool,
    pub draw_dimmed: bool,
    pub draw_labels: bool,
    pub draw_lines: bool,
    pub draw_arrows: bool,
    pub kifu_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardSide {
    North,
    East,
    South,
    West,
}

pub fn make_svg(sgf: &str, options: &MakeSvgOptions) -> Result<Element, MakeSvgError> {
    let collection = sgf_parse::go::parse(sgf)?;
    let goban = Goban::from_node_in_collection(&options.node_description, &collection)?;
    let (x_range, y_range) = options.goban_range.get_ranges(&goban, options)?;
    let width = x_range.end - x_range.start;
    let height = y_range.end - y_range.start;
    if !options.label_sides.is_empty() && width > 25 || height > 99 {
        return Err(MakeSvgError::UnlabellableRange);
    }
    let (top_margin, right_margin, bottom_margin, left_margin) = get_margins(&options.label_sides);

    let definitions = {
        let clip_path = Element::builder("clipPath", NAMESPACE)
            .attr("id", "board-clip")
            .append(
                Element::builder("rect", NAMESPACE)
                    .attr("x", format_float(f64::from(x_range.start) - 0.5))
                    .attr("y", format_float(f64::from(y_range.start) - 0.5))
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
    let diagram_width = f64::from(width) - 1.0 + 2.0 * BOARD_MARGIN + left_margin + right_margin;

    let (diagram, diagram_height) = {
        let board = build_board(&goban, options);
        let board_view = {
            let board_view_transform = format!(
                "translate({}, {})",
                format_float(BOARD_MARGIN + left_margin - f64::from(x_range.start)),
                format_float(BOARD_MARGIN + top_margin - f64::from(y_range.start))
            );
            Element::builder("g", NAMESPACE)
                .attr("id", "board-view")
                .attr("transform", board_view_transform)
                .append(board)
                .build()
        };

        let scale = format_float(options.viewbox_width / diagram_width);
        let transform = format!("scale({}, {})", scale, scale);
        let mut diagram_builder = Element::builder("g", NAMESPACE)
            .attr("id", "diagram")
            .attr("transform", transform)
            .append(board_view);

        if !options.label_sides.is_empty() {
            let goban_size = goban.size();
            diagram_builder = diagram_builder.append(draw_board_labels(
                x_range,
                goban_size.1 - height - y_range.start + 1..goban_size.1 - y_range.start + 1,
                options,
            ));
        }

        let mut diagram_height =
            f64::from(height) - 1.0 + 2.0 * BOARD_MARGIN + top_margin + bottom_margin;
        if options.kifu_mode {
            if let Some((element, element_height)) = draw_repeated_stones(
                &goban,
                width,
                diagram_height + REPEATED_MOVES_MARGIN,
                options,
            ) {
                diagram_builder = diagram_builder.append(element);
                diagram_height += element_height + REPEATED_MOVES_MARGIN * 2.0;
            }
        }

        (diagram_builder.build(), diagram_height)
    };

    let background = Element::builder("rect", NAMESPACE)
        .attr("fill", options.style.background_fill())
        .attr("height", "100%")
        .attr("width", "100%")
        .attr("x", "0")
        .attr("y", "0")
        .build();

    let viewbox_height = options.viewbox_width * diagram_height / diagram_width;
    let viewbox_attr = format!(
        "0 0 {} {}",
        format_float(options.viewbox_width),
        format_float(viewbox_height)
    );
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

    let move_numbers = get_move_numbers_to_draw(goban, options);
    let no_markup_points: HashSet<(u8, u8)> = move_numbers
        .iter()
        .map(|(_, stone)| (stone.x, stone.y))
        .collect();
    if options.draw_move_numbers {
        group_builder =
            group_builder.append(build_move_numbers_group(goban, options, &move_numbers));
    }
    if options.draw_marks {
        group_builder = group_builder.append(build_marks_group(goban, options, &no_markup_points));
    }
    if options.draw_triangles {
        group_builder =
            group_builder.append(build_triangles_group(goban, options, &no_markup_points));
    }
    if options.draw_circles {
        group_builder =
            group_builder.append(build_circles_group(goban, options, &no_markup_points));
    }
    if options.draw_squares {
        group_builder =
            group_builder.append(build_squares_group(goban, options, &no_markup_points));
    }
    if options.draw_selected {
        group_builder =
            group_builder.append(build_selected_group(goban, options, &no_markup_points));
    }
    if options.draw_dimmed {
        group_builder = group_builder.append(build_dimmed_group(goban, options));
    }
    if options.draw_labels {
        group_builder = group_builder.append(build_label_group(goban, options, &no_markup_points));
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
        .attr("stroke-width", format_float(options.style.line_width()))
        .attr("stroke-linecap", "square");

    // Draw lines
    let goban_size = goban.size();
    for x in 0..goban_size.0 as usize {
        group_builder = group_builder.append(
            Element::builder("line", NAMESPACE)
                .attr("x1", x.to_string())
                .attr("y1", "0")
                .attr("x2", x.to_string())
                .attr("y2", (goban_size.1 - 1).to_string()),
        );
    }
    for y in 0..goban_size.1 as usize {
        group_builder = group_builder.append(
            Element::builder("line", NAMESPACE)
                .attr("x1", 0.to_string())
                .attr("y1", y.to_string())
                .attr("x2", (goban_size.0 - 1).to_string())
                .attr("y2", y.to_string()),
        );
    }

    // Draw hoshi
    let hoshi_radius = options.style.hoshi_radius();
    let mut hoshi = Element::builder("g", NAMESPACE)
        .attr("id", "hoshi")
        .attr("stroke", "none")
        .attr("fill", options.style.line_color());
    for (x, y) in goban.hoshi_points() {
        hoshi = hoshi.append(
            Element::builder("circle", NAMESPACE)
                .attr("cx", x.to_string())
                .attr("cy", y.to_string())
                .attr("r", format_float(hoshi_radius)),
        );
    }
    group_builder.append(hoshi).build()
}

fn build_stones_group(goban: &Goban, options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "stones")
        .attr("stroke", "none");
    let mut stones: Vec<Stone> = if options.kifu_mode {
        let mut stones: HashMap<(u8, u8), Stone> = goban
            .stones_before_move(options.first_move_number)
            .map(|stone| ((stone.x, stone.y), stone))
            .collect();
        let new_stones = get_move_numbers(goban, options)
            .into_iter()
            .map(|(_, stone)| stone);
        for stone in new_stones {
            stones.entry((stone.x, stone.y)).or_insert(stone);
        }
        stones.into_values().collect()
    } else {
        goban.stones().collect()
    };
    stones.sort_by_key(|stone| (stone.y, stone.x));
    for stone in stones {
        group_builder = group_builder.append(draw_stone(stone, &options.style));
    }
    group_builder.build()
}

fn build_move_numbers_group(
    goban: &Goban,
    options: &MakeSvgOptions,
    move_numbers: &[(u64, Stone)],
) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "move-numbers")
        .attr("text-anchor", "middle");
    for (n, stone) in move_numbers {
        let stone_color = if options.kifu_mode {
            // In kifu mode, the first numbered stone played will be shown.
            Some(stone.color)
        } else {
            // Otherwise, we can look at the board.
            goban.stone_color(stone.x, stone.y)
        };
        let starting_num = (n - options.first_move_number) + 1;
        group_builder = group_builder.append(draw_move_number(
            stone.x,
            stone.y,
            starting_num,
            stone_color,
            &options.style,
        ));
    }
    group_builder.build()
}

fn get_move_numbers_to_draw(goban: &Goban, options: &MakeSvgOptions) -> Vec<(u64, Stone)> {
    get_move_numbers(goban, options)
        .into_iter()
        .filter(|(n, _)| n >= &options.first_move_number)
        .collect()
}

fn get_move_numbers(goban: &Goban, options: &MakeSvgOptions) -> Vec<(u64, Stone)> {
    if !options.draw_move_numbers {
        return Vec::new();
    }
    let numbered_moves = goban
        .moves()
        .skip_while(|(n, _)| n < &options.first_move_number);
    let mut move_numbers: HashMap<(u8, u8), (u64, Stone)> = HashMap::new();
    for (n, stone) in numbered_moves {
        if options.kifu_mode {
            // In Kifu mode we care about the first numbered stone played.
            move_numbers.entry((stone.x, stone.y)).or_insert((n, stone));
        } else {
            // Otherwise we care about the last numbered stone played.
            move_numbers.insert((stone.x, stone.y), (n, stone));
        }
    }
    let mut move_numbers: Vec<(u64, Stone)> = move_numbers.values().copied().collect();
    move_numbers.sort_unstable_by_key(|(n, _)| *n);
    move_numbers
}

fn build_marks_group(
    goban: &Goban,
    options: &MakeSvgOptions,
    no_markup_points: &HashSet<(u8, u8)>,
) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-marks");
    let mut marks: Vec<_> = goban.marks().collect();
    marks.sort_unstable();
    for point in marks.iter().filter(|p| !no_markup_points.contains(p)) {
        let stone_color = goban.stone_color(point.0, point.1);
        group_builder =
            group_builder.append(draw_mark(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_triangles_group(
    goban: &Goban,
    options: &MakeSvgOptions,
    no_markup_points: &HashSet<(u8, u8)>,
) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-triangles");
    let mut triangles: Vec<_> = goban.triangles().collect();
    triangles.sort_unstable();
    for point in triangles.iter().filter(|p| !no_markup_points.contains(p)) {
        let stone_color = goban.stone_color(point.0, point.1);
        group_builder =
            group_builder.append(draw_triangle(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_circles_group(
    goban: &Goban,
    options: &MakeSvgOptions,
    no_markup_points: &HashSet<(u8, u8)>,
) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-circles");
    let mut circles: Vec<_> = goban.circles().collect();
    circles.sort_unstable();
    for point in circles.iter().filter(|p| !no_markup_points.contains(p)) {
        let stone_color = goban.stone_color(point.0, point.1);
        group_builder =
            group_builder.append(draw_circle(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_squares_group(
    goban: &Goban,
    options: &MakeSvgOptions,
    no_markup_points: &HashSet<(u8, u8)>,
) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-squares");
    let mut squares: Vec<_> = goban.squares().collect();
    squares.sort_unstable();
    for point in squares.iter().filter(|p| !no_markup_points.contains(p)) {
        let stone_color = goban.stone_color(point.0, point.1);
        group_builder =
            group_builder.append(draw_square(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_selected_group(
    goban: &Goban,
    options: &MakeSvgOptions,
    no_markup_points: &HashSet<(u8, u8)>,
) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-selected");
    let mut selected: Vec<_> = goban.selected().collect();
    selected.sort_unstable();
    for point in selected.iter().filter(|p| !no_markup_points.contains(p)) {
        let stone_color = goban.stone_color(point.0, point.1);
        group_builder =
            group_builder.append(draw_selected(point.0, point.1, stone_color, &options.style));
    }
    group_builder.build()
}

fn build_dimmed_group(goban: &Goban, _options: &MakeSvgOptions) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-dimmed");
    let mut dimmed: Vec<_> = goban.dimmed().collect();
    dimmed.sort_unstable();
    for point in dimmed {
        group_builder = group_builder.append(dim_square(point.0, point.1));
    }
    group_builder.build()
}

fn build_label_group(
    goban: &Goban,
    options: &MakeSvgOptions,
    no_markup_points: &HashSet<(u8, u8)>,
) -> Element {
    let mut group_builder = Element::builder("g", NAMESPACE).attr("id", "markup-labels");
    let mut labels: Vec<_> = goban.labels().collect();
    labels.sort_unstable();
    for (point, text) in labels.iter().filter(|(p, _)| !no_markup_points.contains(p)) {
        let stone_color = goban.stone_color(point.0, point.1);
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
        .attr("stroke-width", format_float(options.style.line_width()))
        .attr("marker-start", "url(#linehead)")
        .attr("marker-end", "url(#linehead)");
    let mut lines: Vec<_> = goban.lines().collect();
    lines.sort_unstable();
    for (p1, p2) in lines {
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
        .attr("stroke-width", format_float(options.style.line_width()))
        .attr("marker-end", "url(#arrowhead)");
    let mut arrows: Vec<_> = goban.arrows().collect();
    arrows.sort_unstable();
    for (p1, p2) in arrows {
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
fn draw_board_labels(x_range: Range<u8>, y_range: Range<u8>, options: &MakeSvgOptions) -> Element {
    let (top_margin, _, _, left_margin) = get_margins(&options.label_sides);
    let transform = format!(
        "translate({}, {})",
        format_float(left_margin),
        format_float(top_margin)
    );
    let mut group_builder = Element::builder("g", NAMESPACE)
        .attr("id", "board-labels")
        .attr("fill", options.style.label_color())
        .attr("transform", transform);

    if options.label_sides.contains(&BoardSide::North) {
        let mut builder = Element::builder("g", NAMESPACE).attr("text-anchor", "middle");
        let start = x_range.start;
        for x in x_range.clone() {
            builder = builder.append(
                Element::builder("text", NAMESPACE)
                    .attr("x", format_float(f64::from(x - start) + BOARD_MARGIN))
                    .attr("y", "0")
                    .append(label_text(x))
                    .build(),
            );
        }
        group_builder = group_builder.append(builder);
    };
    if options.label_sides.contains(&BoardSide::West) {
        let mut builder = Element::builder("g", NAMESPACE).attr("text-anchor", "end");
        let end = y_range.end;
        for y in y_range.clone() {
            builder = builder.append(
                Element::builder("text", NAMESPACE)
                    .attr("x", "0")
                    .attr("y", format_float(f64::from(end - y - 1) + BOARD_MARGIN))
                    .attr("dy", "0.35em")
                    .append(y.to_string())
                    .build(),
            );
        }
        group_builder = group_builder.append(builder);
    };
    if options.label_sides.contains(&BoardSide::South) {
        let mut builder = Element::builder("g", NAMESPACE).attr("text-anchor", "middle");
        let start = x_range.start;
        let y = f64::from(y_range.end - y_range.start + 1) - BOARD_MARGIN;
        for x in x_range.clone() {
            builder = builder.append(
                Element::builder("text", NAMESPACE)
                    .attr("x", format_float(f64::from(x - start) + BOARD_MARGIN))
                    .attr("y", format_float(y))
                    .attr("alignment-baseline", "hanging")
                    .append(label_text(x))
                    .build(),
            );
        }
        group_builder = group_builder.append(builder);
    };
    if options.label_sides.contains(&BoardSide::East) {
        let mut builder = Element::builder("g", NAMESPACE).attr("text-anchor", "start");
        let end = y_range.end;
        let x = f64::from(x_range.end - x_range.start + 1) - BOARD_MARGIN;
        for y in y_range {
            builder = builder.append(
                Element::builder("text", NAMESPACE)
                    .attr("x", format_float(x))
                    .attr("y", format_float(f64::from(end - y - 1) + BOARD_MARGIN))
                    .attr("dy", "0.35em")
                    .append(y.to_string())
                    .build(),
            );
        }
        group_builder = group_builder.append(builder);
    };

    group_builder.build()
}

fn draw_repeated_stones(
    goban: &Goban,
    width: u8,
    diagram_height: f64,
    options: &MakeSvgOptions,
) -> Option<(Element, f64)> {
    let entry_padding = 0.2;
    let entry_width = 1.9;
    let entry_height = 0.4;
    let width = f64::from(width);
    let (_, _, _, left_margin) = get_margins(&options.label_sides);
    let columns = ((width - 1.0 - (2.0 * entry_padding)) / entry_width).floor() as usize;
    let x = BOARD_MARGIN
        + left_margin
        + entry_padding
        + (width - 1.0 - 2.0 * entry_padding - entry_width * f64::from(columns as u32)) / 2.0;
    let y = diagram_height + entry_padding + entry_height;
    let mut text_builder = Element::builder("text", NAMESPACE)
        .attr("y", format_float(y))
        .attr("font-size", format_float(entry_height))
        .attr("fill", options.style.line_color()); // TODO: Evaluate this choice
    let repeated_moves: Vec<(u64, u64)> = {
        let mut repeated_moves = Vec::new();
        let mut seen_moves: HashMap<(u8, u8), u64> = HashMap::new();
        let numbered_moves = goban
            .moves()
            .skip_while(|(n, _)| n < &options.first_move_number)
            .map(|(n, stone)| (n + 1 - options.first_move_number, stone));
        for (n, stone) in numbered_moves {
            match seen_moves.entry((stone.x, stone.y)) {
                std::collections::hash_map::Entry::Occupied(entry) => {
                    repeated_moves.push((n, *entry.get()))
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(n);
                }
            }
        }
        repeated_moves.sort_unstable();
        repeated_moves
    };
    if repeated_moves.is_empty() {
        return None;
    }
    let rows = (f64::from(repeated_moves.len() as u32) / f64::from(columns as u32)).ceil();
    for (i, (move_num, original)) in repeated_moves.iter().enumerate() {
        let column = f64::from((i % columns) as u32);
        let mut tspan_builder = Element::builder("tspan", NAMESPACE)
            .append(format!("{}→{}", move_num, original))
            .attr("x", format_float(x + entry_width * column));
        if i % columns == 0 && i != 0 {
            tspan_builder = tspan_builder.attr("dy", format_float(entry_height));
        }
        text_builder = text_builder.append(tspan_builder);
    }
    let rect_height = 2.0 * entry_padding + entry_height * rows + options.style.line_width() * 2.0;
    let group = Element::builder("g", NAMESPACE)
        .attr("id", "repeated-stones")
        .append(
            Element::builder("rect", NAMESPACE)
                .attr("fill", "white")
                .attr("stroke", options.style.line_color())
                .attr("stroke-width", format_float(options.style.line_width()))
                .attr("x", format_float(BOARD_MARGIN + left_margin))
                .attr("y", format_float(diagram_height))
                .attr("width", format_float(width - 1.0))
                .attr("height", format_float(rect_height)),
        )
        .append(text_builder)
        .build();

    Some((group, rect_height))
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
        .attr("cx", stone.x)
        .attr("cy", stone.y)
        .attr("r", "0.48");
    if let Some(stroke) = style.stone_stroke(stone.color) {
        circle_builder = circle_builder
            .attr("stroke", stroke)
            .attr("stroke-width", format_float(style.line_width()))
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
        .attr("x", x)
        .attr("y", y)
        .attr("dy", "0.35em")
        .attr("fill", style.markup_color(color))
        .append(n.to_string());
    let mut group_builder = Element::builder("g", NAMESPACE);
    if color.is_none() {
        group_builder = group_builder.append(
            Element::builder("rect", NAMESPACE)
                .attr("fill", style.background_fill())
                .attr("x", format_float(f64::from(x) - 0.4))
                .attr("y", format_float(f64::from(y) - 0.4))
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
                .attr("x1", format_float(f64::from(x) - 0.25))
                .attr("x2", format_float(f64::from(x) + 0.25))
                .attr("y1", format_float(f64::from(y) - 0.25))
                .attr("y2", format_float(f64::from(y) + 0.25)),
        )
        .append(
            Element::builder("line", NAMESPACE)
                .attr("x1", format_float(f64::from(x) - 0.25))
                .attr("x2", format_float(f64::from(x) + 0.25))
                .attr("y1", format_float(f64::from(y) + 0.25))
                .attr("y2", format_float(f64::from(y) - 0.25)),
        )
        .build()
}

fn draw_triangle(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let triangle_radius = 0.45;
    let points = format!(
        "{},{} {},{} {},{}",
        x,
        format_float(f64::from(y) - triangle_radius),
        format_float(f64::from(x) - 0.866 * triangle_radius),
        format_float(f64::from(y) + 0.5 * triangle_radius),
        format_float(f64::from(x) + 0.866 * triangle_radius),
        format_float(f64::from(y) + 0.5 * triangle_radius),
    );
    Element::builder("g", NAMESPACE)
        .attr("stroke", style.markup_color(color))
        .attr("fill", "none")
        .attr("stroke-width", format_float(style.line_width()))
        .append(Element::builder("polygon", NAMESPACE).attr("points", points))
        .build()
}

fn draw_circle(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let radius = 0.25;
    Element::builder("g", NAMESPACE)
        .attr("stroke", style.markup_color(color))
        .attr("fill", "none")
        .attr("stroke-width", format_float(style.line_width()))
        .append(
            Element::builder("circle", NAMESPACE)
                .attr("cx", x)
                .attr("cy", y)
                .attr("r", format_float(radius)),
        )
        .build()
}

fn draw_square(x: u8, y: u8, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let width = 0.55;
    Element::builder("g", NAMESPACE)
        .attr("stroke", style.markup_color(color))
        .attr("fill", "none")
        .attr("stroke-width", format_float(style.line_width()))
        .append(
            Element::builder("rect", NAMESPACE)
                .attr("x", format_float(f64::from(x) - 0.5 * width))
                .attr("y", format_float(f64::from(y) - 0.5 * width))
                .attr("width", format_float(width))
                .attr("height", format_float(width)),
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
    Element::builder("g", NAMESPACE)
        .attr("stroke", "none")
        .attr("fill", "black")
        .attr("fill-opacity", "0.5")
        .attr("shape-rendering", "crispEdges")
        .append(
            Element::builder("rect", NAMESPACE)
                .attr("x", format_float(f64::from(x) - 0.5))
                .attr("y", format_float(f64::from(y) - 0.5))
                .attr("width", "1")
                .attr("height", "1"),
        )
        .build()
}

fn draw_label(x: u8, y: u8, text: &str, color: Option<StoneColor>, style: &GobanStyle) -> Element {
    let text = text.chars().take(2).collect::<String>();
    let text_element = Element::builder("text", NAMESPACE)
        .attr("x", x)
        .attr("y", y)
        .attr("text-anchor", "middle")
        .attr("dy", "0.35em")
        .attr("fill", style.markup_color(color))
        .append(text);
    let mut group_builder = Element::builder("g", NAMESPACE);
    if color.is_none() {
        group_builder = group_builder.append(
            Element::builder("rect", NAMESPACE)
                .attr("fill", style.background_fill())
                .attr("x", format_float(f64::from(x) - 0.4))
                .attr("y", format_float(f64::from(y) - 0.4))
                .attr("width", "0.8")
                .attr("height", "0.8"),
        );
    }

    group_builder.append(text_element).build()
}

fn get_margins(label_sides: &HashSet<BoardSide>) -> (f64, f64, f64, f64) {
    let top = if label_sides.contains(&BoardSide::North) {
        LABEL_MARGIN
    } else {
        0.0
    };
    let right = if label_sides.contains(&BoardSide::East) {
        LABEL_MARGIN
    } else {
        0.0
    };
    let bottom = if label_sides.contains(&BoardSide::South) {
        LABEL_MARGIN
    } else {
        0.0
    };
    let left = if label_sides.contains(&BoardSide::West) {
        LABEL_MARGIN
    } else {
        0.0
    };
    (top, right, bottom, left)
}

fn format_float(x: f64) -> String {
    format!("{:.4}", x)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}
