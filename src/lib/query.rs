use std::io::{stdout, Write};

use sgf_parse::{go::Prop, SgfNode};

use crate::errors::QueryError;
use crate::sgf_traversal::{variation_nodes, variation_roots, SgfTraversalNode};
use crate::{QueryArgs, QueryMode};

pub fn query(sgf: &str, query_args: &QueryArgs) -> Result<(), QueryError> {
    let collection = sgf_parse::go::parse(sgf)?;
    match query_args.mode() {
        QueryMode::Default => write_query_text(&collection, stdout())?,
        QueryMode::LastGame => println!("{}", query_game_index(&collection)?),
        QueryMode::LastVariation => println!(
            "{}",
            query_variation_index(&collection, query_args.game_number)?
        ),
        QueryMode::LastNode => println!(
            "{}",
            query_node_index(&collection, query_args.game_number, query_args.variation)?
        ),
    }
    Ok(())
}

fn query_game_index(collection: &[SgfNode<Prop>]) -> Result<usize, QueryError> {
    match collection.len() {
        0 => Err(QueryError::GameNotFound),
        n => Ok(n - 1),
    }
}

fn query_variation_index(
    collection: &[SgfNode<Prop>],
    game_number: u64,
) -> Result<u64, QueryError> {
    let sgf_node = collection
        .get(game_number as usize)
        .ok_or(QueryError::GameNotFound)?;
    let node = variation_roots(sgf_node)
        .last()
        .ok_or(QueryError::VariationNotFound)?;
    Ok(node.variation)
}

fn query_node_index(
    collection: &[SgfNode<Prop>],
    game_number: u64,
    variation: u64,
) -> Result<usize, QueryError> {
    let sgf_node = collection
        .get(game_number as usize)
        .ok_or(QueryError::GameNotFound)?;
    let count = variation_nodes(sgf_node, variation)
        .map_err(|_| QueryError::VariationNotFound)?
        .count();
    match count {
        0 => Err(QueryError::VariationNotFound),
        n => Ok(n - 1),
    }
}

fn write_query_text(
    collection: &[SgfNode<Prop>],
    mut writer: impl Write,
) -> Result<(), QueryError> {
    for (game_num, node) in collection.iter().enumerate() {
        writeln!(writer, "Game #{game_num}")?;
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
    use sgf_parse::{go::Prop, SgfNode};

    use super::{
        query_game_index, query_node_index, query_variation_index, write_query_text, QueryError,
    };

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

    fn get_collection() -> Vec<SgfNode<Prop>> {
        sgf_parse::go::parse(TEST_DATA).unwrap()
    }

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
        write_query_text(&get_collection(), &mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn game_index() {
        let result = query_game_index(&get_collection()).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn game_index_error() {
        let result = query_game_index(&[]);
        assert!(matches!(result, Err(QueryError::GameNotFound)));
    }

    #[test]
    fn variation_index() {
        assert_eq!(query_variation_index(&get_collection(), 0).unwrap(), 8);
        assert_eq!(query_variation_index(&get_collection(), 1).unwrap(), 1);
    }

    #[test]
    fn variation_index_error() {
        let result = query_variation_index(&get_collection(), 2);
        assert!(matches!(result, Err(QueryError::GameNotFound)));
    }

    #[test]
    fn node_index() {
        assert_eq!(query_node_index(&get_collection(), 0, 0).unwrap(), 8);
        assert_eq!(query_node_index(&get_collection(), 0, 1).unwrap(), 7);
        assert_eq!(query_node_index(&get_collection(), 0, 2).unwrap(), 7);
        assert_eq!(query_node_index(&get_collection(), 0, 3).unwrap(), 7);
        assert_eq!(query_node_index(&get_collection(), 0, 4).unwrap(), 7);
        assert_eq!(query_node_index(&get_collection(), 0, 5).unwrap(), 7);
        assert_eq!(query_node_index(&get_collection(), 0, 6).unwrap(), 8);
        assert_eq!(query_node_index(&get_collection(), 0, 7).unwrap(), 5);
        assert_eq!(query_node_index(&get_collection(), 0, 8).unwrap(), 5);
        assert_eq!(query_node_index(&get_collection(), 1, 0).unwrap(), 3);
        assert_eq!(query_node_index(&get_collection(), 1, 1).unwrap(), 3);
    }

    #[test]
    fn node_index_error() {
        let result = query_node_index(&get_collection(), 0, 9);
        assert!(matches!(result, Err(QueryError::VariationNotFound)));
    }
}
