use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::resonance;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = resonance::deserialize_problem(url).ok_or("invalid url")?;
    let ans = resonance::solve_resonance(&problem);

    let height = problem.height;
    let width = problem.width;
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&ans));

    for y in 0..height {
        for x in 0..width {
            if let Some(clue_val) = problem.clues[y][x] {
                board.push(Item::cell(y, x, "black", ItemKind::Num(clue_val)));
            } else if let Some(ref grid) = ans {
                if let Some(val) = grid[y][x] {
                    if val > 0 {
                        board.push(Item::cell(y, x, "green", ItemKind::Num(val)));
                    } else {
                        board.push(Item::cell(y, x, "green", ItemKind::Dot));
                    }
                }
            }
        }
    }

    Ok(board)
}
