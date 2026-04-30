use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::tidepool;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = tidepool::deserialize_problem(url).ok_or("invalid url")?;
    let ans = tidepool::solve_tidepool(&problem);

    let height = problem.len();
    let width = problem[0].len();
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&ans));
    for y in 0..height {
        for x in 0..width {
            if let Some(clue) = problem[y][x] {
                if clue >= 0 {
                    board.push(Item::cell(y, x, "black", ItemKind::Num(clue)));
                }
            } else if let Some(ans) = &ans {
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

#[cfg(test)]
mod tests {
    use super::solve;
    use crate::uniqueness::Uniqueness;

    #[test]
    fn test_tidepool_easy() {
        let url = "https://puzz.link/p?tidepool/6/6/00000001h100g33g0g1h1g01111000h00";
        let board = solve(url).unwrap();
        assert_eq!(board.uniqueness, Uniqueness::Unique);
    }
}
