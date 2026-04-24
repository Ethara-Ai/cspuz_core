use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::Uniqueness;
use cspuz_rs_puzzles::puzzles::sudoku2;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = sudoku2::deserialize_problem(url).ok_or("invalid url")?;
    let ans = sudoku2::solve_sudoku2(&problem);

    let height = problem.len();
    let width = problem[0].len();

    let is_unique = if let Some(ans) = &ans {
        let mut uniqueness = Uniqueness::Unique;
        for y in 0..height {
            for x in 0..width {
                if ans[y][x].is_none() {
                    uniqueness = Uniqueness::NonUnique;
                }
            }
        }
        uniqueness
    } else {
        Uniqueness::NoAnswer
    };
    let mut board = Board::new(BoardKind::Grid, height, width, is_unique);

    let (bh, bw) = match height {
        4 => (2, 2),
        6 => (2, 3),
        9 => (3, 3),
        16 => (4, 4),
        25 => (5, 5),
        _ => return Err("invalid size"),
    };

    if let Some(ans) = &ans {
        for y in 0..height {
            for x in 0..width {
                if let Some(n) = problem[y][x] {
                    board.push(Item::cell(y, x, "black", ItemKind::Num(n)));
                } else if let Some(v) = ans[y][x] {
                    board.push(Item::cell(y, x, "green", ItemKind::Num(v)));
                }
            }
        }
    } else {
        for y in 0..height {
            for x in 0..width {
                if let Some(n) = problem[y][x] {
                    board.push(Item::cell(y, x, "black", ItemKind::Num(n)));
                }
            }
        }
    }
    for x in 0..bh {
        for y in 0..height {
            board.push(Item {
                y: 2 * y + 1,
                x: 2 * x * bw,
                color: "black",
                kind: ItemKind::BoldWall,
            });
        }
    }
    for y in 0..bw {
        for x in 0..width {
            board.push(Item {
                y: 2 * y * bh,
                x: 2 * x + 1,
                color: "black",
                kind: ItemKind::BoldWall,
            });
        }
    }

    Ok(board)
}
