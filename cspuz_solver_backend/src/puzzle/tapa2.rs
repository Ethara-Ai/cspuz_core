use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::tapa2;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = tapa2::deserialize_problem(url).ok_or("invalid url")?;
    let ans = tapa2::solve_tapa2(&problem);

    let height = problem.len();
    let width = problem[0].len();
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&ans));
    for y in 0..height {
        for x in 0..width {
            if let Some(clue) = problem[y][x] {
                board.push(Item::cell(y, x, "black", ItemKind::TapaClue(clue)));
            } else if let Some(ref ans) = ans {
                if let Some(a) = ans[y][x] {
                    board.push(Item::cell(
                        y,
                        x,
                        "green",
                        if a { ItemKind::Block } else { ItemKind::Dot },
                    ));
                }
            }
        }
    }

    Ok(board)
}
