use crate::board_side::BoardSide;
use crate::errors::MakeSvgError;
use crate::goban::StoneColor;
use crate::{board_label_text, Goban, MakeSvgOptions};

pub fn text_diagram(goban: &Goban, options: &MakeSvgOptions) -> Result<String, MakeSvgError> {
    let (x_range, y_range) = options.goban_range.get_ranges(goban, options)?;
    let width = x_range.end - x_range.start;
    let height = y_range.end - y_range.start;
    if !options.label_sides.is_empty() && width > 25 || height > 99 {
        return Err(MakeSvgError::UnlabellableRange);
    }
    let mut lines: Vec<String> = vec![];
    let label_padding = if options.label_sides.contains(BoardSide::West) {
        "   "
    } else {
        ""
    };
    if options.label_sides.contains(BoardSide::North) {
        let line: String = x_range.clone().map(board_label_text).collect();
        lines.push(format!("{}{}", label_padding, line));
    }
    for y in y_range {
        let mut line = x_range.clone().map(|x| char_at(goban, x, y)).collect();
        if options.label_sides.contains(BoardSide::West) {
            line = format!("{: >2} {}", y + 1, line);
        }
        if options.label_sides.contains(BoardSide::East) {
            line.push_str(&format!(" {}", y + 1));
        }
        lines.push(line);
    }
    if options.label_sides.contains(BoardSide::South) {
        let line: String = x_range.clone().map(board_label_text).collect();
        lines.push(format!("{}{}", label_padding, line));
    }
    Ok(lines.join("\n"))
}

fn char_at(goban: &Goban, x: u8, y: u8) -> char {
    let max_x = goban.size().0 - 1;
    let max_y = goban.size().1 - 1;
    match goban.stone_color(x, y) {
        Some(StoneColor::White) => '●',
        Some(StoneColor::Black) => '○',
        None => match (x, y) {
            (0, 0) => '┏',
            (x, 0) if x == max_x => '┓',
            (0, y) if y == max_y => '┗',
            (x, y) if x == max_x && y == max_y => '┛',
            (_, 0) => '┯',
            (0, _) => '┠',
            (_, y) if y == max_y => '┷',
            (x, _) if x == max_x => '┨',
            (_, _) => '┼',
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::goban_range::GobanRange;
    use crate::{Goban, MakeSvgOptions};

    use super::text_diagram;

    fn build_diagram(sgf_dir: &str, options: &MakeSvgOptions) -> String {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "data",
            sgf_dir,
            "input.sgf",
        ]
        .iter()
        .collect();
        let sgf = std::fs::read_to_string(d).unwrap();
        let goban = Goban::from_sgf(&sgf, &options.node_description).unwrap();
        text_diagram(&goban, &options).unwrap()
    }

    #[test]
    fn full_board() {
        let options = MakeSvgOptions::default();
        let diagram = build_diagram("last_move", &options);
        let expected = "\
┏┯┯┯┯┯┯┯┯┯┯○●●●●┯┯┓
┠┼┼┼┼┼┼┼┼┼┼○○○●○●┼┨
┠┼┼┼○○●○○○┼┼○●●○●●┨
○○○○○●○○●┼○┼┼○●○○○●
○●●●○●●●┼┼┼┼○┼┼┼○●┨
●┼●○┼┼┼┼○┼┼┼●○┼●●┼●
┠●●○┼┼○○┼●○○○●●┼┼●┨
┠●○○○○●○┼○●●●○┼┼┼┼┨
┠●○●┼○●○○○○○●●●●●┼┨
┠┼●●●┼●○●●●●○○○○○●┨
┠┼┼●○┼●●○┼┼○┼┼┼┼┼●┨
┠┼●┼●○┼○┼○○┼┼┼┼┼○●┨
┠●●●○○┼○○┼●○┼┼○┼○●┨
┠●○○○●●●●●●○┼┼○●●┼┨
○●●○●┼●○○○●●○┼┼○●┼┨
┠○○●●●┼●●○○○○○○┼●┼┨
┠○┼○┼●●●○┼┼●●○●●┼┼┨
┠┼○○●┼┼●○┼○●┼●┼┼┼┼┨
┗┷┷┷┷┷┷┷┷┷○┷●┷┷┷┷┷┛";
        assert_eq!(diagram, expected);
    }

    #[test]
    fn labels() {
        let mut options = MakeSvgOptions::default();
        options.label_sides = "nw".parse().unwrap();
        let diagram = build_diagram("last_move", &options);
        let expected = "   ABCDEFGHJKLMNOPQRST
 1 ┏┯┯┯┯┯┯┯┯┯┯○●●●●┯┯┓
 2 ┠┼┼┼┼┼┼┼┼┼┼○○○●○●┼┨
 3 ┠┼┼┼○○●○○○┼┼○●●○●●┨
 4 ○○○○○●○○●┼○┼┼○●○○○●
 5 ○●●●○●●●┼┼┼┼○┼┼┼○●┨
 6 ●┼●○┼┼┼┼○┼┼┼●○┼●●┼●
 7 ┠●●○┼┼○○┼●○○○●●┼┼●┨
 8 ┠●○○○○●○┼○●●●○┼┼┼┼┨
 9 ┠●○●┼○●○○○○○●●●●●┼┨
10 ┠┼●●●┼●○●●●●○○○○○●┨
11 ┠┼┼●○┼●●○┼┼○┼┼┼┼┼●┨
12 ┠┼●┼●○┼○┼○○┼┼┼┼┼○●┨
13 ┠●●●○○┼○○┼●○┼┼○┼○●┨
14 ┠●○○○●●●●●●○┼┼○●●┼┨
15 ○●●○●┼●○○○●●○┼┼○●┼┨
16 ┠○○●●●┼●●○○○○○○┼●┼┨
17 ┠○┼○┼●●●○┼┼●●○●●┼┼┨
18 ┠┼○○●┼┼●○┼○●┼●┼┼┼┼┨
19 ┗┷┷┷┷┷┷┷┷┷○┷●┷┷┷┷┷┛";
        assert_eq!(diagram, expected);
    }

    #[test]
    fn range() {
        let mut options = MakeSvgOptions::default();
        options.goban_range = GobanRange::Ranged(1..7, 0..5);
        let diagram = build_diagram("prob45", &options);
        let expected = "\
┯○○●●┯
○┼○○●┼
┼○●●●┼
○○●┼┼┼
●●┼┼┼┼";
        assert_eq!(diagram, expected);
    }

    #[test]
    fn range_with_labels() {
        let mut options = MakeSvgOptions::default();
        options.label_sides = "nwes".parse().unwrap();
        options.goban_range = GobanRange::Ranged(1..7, 0..5);
        let diagram = build_diagram("prob45", &options);
        println!("{}", diagram);
        let expected = "   BCDEFG
 1 ┯○○●●┯ 1
 2 ○┼○○●┼ 2
 3 ┼○●●●┼ 3
 4 ○○●┼┼┼ 4
 5 ●●┼┼┼┼ 5
   BCDEFG";
        assert_eq!(diagram, expected);
    }

    #[test]
    fn shrink_wrap() {
        let mut options = MakeSvgOptions::default();
        options.goban_range = GobanRange::ShrinkWrap;
        let diagram = build_diagram("prob45", &options);
        let expected = "\
┏┯○○●●┯
○○┼○○●┼
●┼○●●●┼
●○○●┼┼┼
┠●●┼┼┼┼
┠┼┼┼┼┼┼";
        assert_eq!(diagram, expected);
    }
}
