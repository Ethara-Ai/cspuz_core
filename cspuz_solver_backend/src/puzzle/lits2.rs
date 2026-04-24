use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::lits2;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let borders = lits2::deserialize_problem(url).ok_or("invalid url")?;
    let ans = lits2::solve_lits2(&borders);

    let height = borders.horizontal.len() + 1;
    let width = if borders.horizontal.is_empty() {
        0
    } else {
        borders.horizontal[0].len()
    };
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&ans));

    board.add_borders(&borders, "black");

    if let Some(is_black) = &ans {
        for y in 0..height {
            for x in 0..width {
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

    Ok(board)
}
