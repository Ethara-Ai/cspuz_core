use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs::graph;
use cspuz_rs_puzzles::puzzles::heyawake_custom;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let (borders, clues) = heyawake_custom::deserialize_problem(url).ok_or("invalid url")?;
    let is_black = heyawake_custom::solve_heyawake_custom(&borders, &clues);

    let height = borders.vertical.len();
    let width = if height > 0 {
        borders.horizontal[0].len()
    } else {
        0
    };
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&is_black));

    board.add_borders(&borders, "black");

    for y in 0..height {
        for x in 0..width {
            if let Some(is_black) = &is_black {
                if let Some(b) = is_black[y][x] {
                    board.push(Item::cell(
                        y,
                        x,
                        "green",
                        if b { ItemKind::Block } else { ItemKind::Dot },
                    ));
                }
            }
        }
    }
    let rooms = graph::borders_to_rooms(&borders);
    assert_eq!(rooms.len(), clues.len());
    for i in 0..rooms.len() {
        if let Some(n) = clues[i] {
            let (y, x) = rooms[i][0];
            board.push(Item::cell(y, x, "black", ItemKind::Num(n)));
        }
    }

    Ok(board)
}
