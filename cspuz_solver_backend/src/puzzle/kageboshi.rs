use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::kageboshi;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = kageboshi::deserialize_problem(url).ok_or("invalid url")?;
    let ans = kageboshi::solve_kageboshi(&problem);

    let height = problem.len();
    let width = problem[0].len();
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&ans));

    for y in 0..height {
        for x in 0..width {
            if let Some(clue) = problem[y][x] {
                board.push(Item::cell(y, x, "black", ItemKind::Num(clue)));
            } else if let Some(ref grid) = ans {
                if let Some(shaded) = grid[y][x] {
                    if shaded {
                        board.push(Item::cell(y, x, "green", ItemKind::Block));
                    } else {
                        board.push(Item::cell(y, x, "green", ItemKind::Dot));
                    }
                }
            }
        }
    }

    Ok(board)
}
