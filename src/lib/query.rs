use sgf_parse::{go::Prop, SgfNode, SgfParseError};

use crate::traversal::{variation_roots, SgfTraversalNode};

pub fn query(sgf: &str) -> Result<(), SgfParseError> {
    let collection = sgf_parse::go::parse(sgf)?;
    for (game_num, node) in collection.iter().enumerate() {
        println!("Game #{}", game_num);
        print_node(node);
        if game_num < collection.len() - 1 {
            println!("");
        }
    }
    Ok(())
}

fn print_node(sgf_node: &SgfNode<Prop>) {
    for fork_node in variation_roots(sgf_node) {
        let SgfTraversalNode {
            sgf_node,
            move_number,
            variation,
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
            + move_number;
        println!("{branch_diagram_for_line}v{variation}, {move_number}-{last_move}",);
    }
}
