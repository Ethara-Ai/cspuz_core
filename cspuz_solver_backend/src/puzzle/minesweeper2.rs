use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::minesweeper2;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = minesweeper2::deserialize_problem(url).ok_or("invalid url")?;
    let ans = minesweeper2::solve_minesweeper2(&problem);

    let height = problem.len();
    let width = problem[0].len();
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&ans));
    for y in 0..height {
        for x in 0..width {
            if let Some(clue) = problem[y][x] {
                if clue >= 0 {
                    board.push(Item::cell(y, x, "black", ItemKind::Num(clue)));
                } else {
                    board.push(Item::cell(y, x, "black", ItemKind::Text("?")));
                }
            } else if let Some(ans) = &ans {
                if let Some(a) = ans[y][x] {
                    board.push(Item::cell(
                        y,
                        x,
                        "green",
                        if a {
                            ItemKind::FilledCircle
                        } else {
                            ItemKind::Dot
                        },
                    ));
                }
            }
        }
    }

    Ok(board)
}
