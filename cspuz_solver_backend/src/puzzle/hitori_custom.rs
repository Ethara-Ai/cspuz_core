use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::hitori_custom;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = hitori_custom::deserialize_problem(url).ok_or("invalid url")?;
    let is_black = hitori_custom::solve_hitori_custom(&problem);

    let height = problem.len();
    let width = problem[0].len();
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&is_black));

    for y in 0..height {
        for x in 0..width {
            if let Some(is_black) = &is_black {
                match (problem[y][x], is_black[y][x]) {
                    (0, None) => (),
                    (0, Some(false)) => {
                        board.push(Item::cell(y, x, "green", ItemKind::Dot));
                    }
                    (0, Some(true)) => {
                        board.push(Item::cell(y, x, "green", ItemKind::Fill));
                    }
                    (n, None) => {
                        board.push(Item::cell(y, x, "black", ItemKind::Num(n)));
                    }
                    (n, Some(false)) => {
                        board.push(Item::cell(y, x, "black", ItemKind::Num(n)));
                        board.push(Item::cell(y, x, "green", ItemKind::Dot));
                    }
                    (n, Some(true)) => {
                        board.push(Item::cell(y, x, "green", ItemKind::Fill));
                        board.push(Item::cell(y, x, "white", ItemKind::Num(n)));
                    }
                }
            } else {
                if problem[y][x] != 0 {
                    board.push(Item::cell(y, x, "black", ItemKind::Num(problem[y][x])));
                }
            }
        }
    }

    Ok(board)
}
