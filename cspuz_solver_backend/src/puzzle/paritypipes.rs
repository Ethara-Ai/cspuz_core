use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs_puzzles::puzzles::paritypipes;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = paritypipes::deserialize_problem(url).ok_or("invalid url")?;
    let ans = paritypipes::solve_paritypipes(&problem);

    let height = problem.len() - 1;
    let width = problem[0].len() - 1;
    let mut board = Board::new(BoardKind::DotGrid, height, width, check_uniqueness(&ans));

    for y in 0..=height {
        for x in 0..=width {
            if problem[y][x] {
                board.push(Item {
                    y: y * 2,
                    x: x * 2,
                    color: "black",
                    kind: ItemKind::SmallFilledCircle,
                });
            } else {
                board.push(Item {
                    y: y * 2,
                    x: x * 2,
                    color: "black",
                    kind: ItemKind::SmallCircle,
                });
            }
        }
    }

    if let Some(ref is_line) = ans {
        for y in 0..height {
            for x in 0..=width {
                if let Some(b) = is_line.vertical[y][x] {
                    board.push(Item {
                        y: y * 2 + 1,
                        x: x * 2,
                        color: "green",
                        kind: if b { ItemKind::Wall } else { ItemKind::Cross },
                    });
                }
            }
        }
        for y in 0..=height {
            for x in 0..width {
                if let Some(b) = is_line.horizontal[y][x] {
                    board.push(Item {
                        y: y * 2,
                        x: x * 2 + 1,
                        color: "green",
                        kind: if b { ItemKind::Wall } else { ItemKind::Cross },
                    });
                }
            }
        }
    }

    Ok(board)
}

#[cfg(test)]
mod tests {
    use super::solve;
    use crate::board::*;
    use crate::uniqueness::Uniqueness;

    #[test]
    fn test_paritypipes_easy() {
        let url = "https://pzprxs.vercel.app/p?paritypipes/6/6/07fauss700";
        let board = solve(url).unwrap();
        assert_eq!(board.uniqueness, Uniqueness::Unique);
    }
}
