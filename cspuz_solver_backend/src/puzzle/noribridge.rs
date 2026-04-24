use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::Uniqueness;
use cspuz_rs::graph;
use cspuz_rs_puzzles::puzzles::noribridge;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let (borders, clues) = noribridge::deserialize_problem(url).ok_or("invalid url")?;

    let (h, w) = borders.base_shape();
    let result = noribridge::solve_noribridge(&borders, &clues);

    let uniqueness = match &result {
        Some((bridge_state, _)) => {
            if bridge_state.iter().all(|b| b.is_some()) {
                Uniqueness::Unique
            } else {
                Uniqueness::NonUnique
            }
        }
        None => Uniqueness::NoAnswer,
    };

    let mut board = Board::new(BoardKind::Grid, h, w, uniqueness);
    board.add_borders(&borders, "black");

    let rooms = graph::borders_to_rooms(&borders);
    for (i, cells) in rooms.iter().enumerate() {
        if let Some(n) = clues[i] {
            let (y, x) = cells[0];
            board.push(Item::cell(y, x, "black", ItemKind::Num(n)));
        }
    }

    if let Some((bridge_state, edges)) = &result {
        let mut cell_room = vec![vec![0usize; w]; h];
        for (room_id, cells) in rooms.iter().enumerate() {
            for &(y, x) in cells {
                cell_room[y][x] = room_id;
            }
        }

        for (edge_idx, &(ra, rb)) in edges.iter().enumerate() {
            if let Some(has_bridge) = bridge_state[edge_idx] {
                for y in 0..h {
                    for x in 0..w {
                        if x + 1 < w {
                            let a = cell_room[y][x];
                            let b = cell_room[y][x + 1];
                            if (a.min(b), a.max(b)) == (ra, rb) {
                                let kind = if has_bridge { ItemKind::BoldWall } else { ItemKind::Cross };
                                board.push(Item { y: 2 * y + 1, x: 2 * x + 2, color: "green", kind });
                            }
                        }
                        if y + 1 < h {
                            let a = cell_room[y][x];
                            let b = cell_room[y + 1][x];
                            if (a.min(b), a.max(b)) == (ra, rb) {
                                let kind = if has_bridge { ItemKind::BoldWall } else { ItemKind::Cross };
                                board.push(Item { y: 2 * y + 2, x: 2 * x + 1, color: "green", kind });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(board)
}
