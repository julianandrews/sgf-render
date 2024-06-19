use std::io::{stdout, Write};

use sgf_parse::{go::Prop, SgfNode};

use crate::errors::QueryError;
use crate::traversal::{variation_roots, SgfTraversalNode};

pub fn query(sgf: &str) -> Result<(), QueryError> {
    write_query_text(sgf, stdout())
}

fn write_query_text(sgf: &str, mut writer: impl Write) -> Result<(), QueryError> {
    let collection = sgf_parse::go::parse(sgf)?;
    for (game_num, node) in collection.iter().enumerate() {
        writeln!(writer, "Game #{}", game_num)?;
        write_game_text(node, &mut writer)?;
        if game_num < collection.len() - 1 {
            writeln!(writer)?;
        }
    }
    Ok(())
}

fn write_game_text<W: Write>(sgf_node: &SgfNode<Prop>, writer: &mut W) -> Result<(), QueryError> {
    for fork_node in variation_roots(sgf_node) {
        let SgfTraversalNode {
            sgf_node,
            variation_node_number,
            variation,
            parent_variation: _,
            branch_number: _,
            is_variation_root: _,
            branches,
        } = fork_node;
        let branch_diagram_for_line = {
            let s: Vec<&str> = branches
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    if i < branches.len() - 1 {
                        match b {
                            true => "│   ",
                            false => "    ",
                        }
                    } else {
                        match b {
                            true => "├── ",
                            false => "└── ",
                        }
                    }
                })
                .collect();
            s.join("")
        };
        let last_move = std::iter::successors(Some(sgf_node), |n| n.children().next()).count() - 1
            + variation_node_number as usize;
        writeln!(
            writer,
            "{branch_diagram_for_line}v{variation}, {variation_node_number}-{last_move}",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_query_text;

    static TEST_DATA: &str = "\
(;GM[1]FF[4]
CA[UTF-8]
AP[Quarry:0.2.0]
SZ[9]
KM[6.5]
PB[Black]
PW[White]
;B[ee];W[ec]
(;B[eg];W[eh];B[dh]
(;W[cd];B[gd];W[fh])
(;W[gd];B[fh]))
(;B[fc];W[fb]
(;B[fd]
(;W[gb]
(;B[eg])
(;B[dg])
(;B[ce]))
(;W[eg]
(;B[gb])
(;B[fg];W[fh])))
(;B[gc])
(;B[eg]))
)

(;GM[1]FF[4]
CA[UTF-8]
AP[Quarry:0.2.0]
SZ[9]
KM[6.5]
PB[Black]
PW[White]
;B[de];W[fe]
(;B[ge])
(;B[fg])
)";

    #[test]
    fn query_diagram() {
        let expected = "\
Game #0
v0, 0-8
├── v0, 3-8
│   ├── v0, 6-8
│   └── v1, 6-7
└── v2, 3-7
    ├── v2, 5-7
    │   ├── v2, 6-7
    │   │   ├── v2, 7-7
    │   │   ├── v3, 7-7
    │   │   └── v4, 7-7
    │   └── v5, 6-7
    │       ├── v5, 7-7
    │       └── v6, 7-8
    ├── v7, 5-5
    └── v8, 5-5

Game #1
v0, 0-3
├── v0, 3-3
└── v1, 3-3
";

        let mut output = vec![];
        write_query_text(TEST_DATA, &mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }
}
