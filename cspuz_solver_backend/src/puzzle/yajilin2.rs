use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs::items::Arrow;
use cspuz_rs_puzzles::puzzles::yajilin2;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let (outside, problem) = yajilin2::deserialize_problem(url).ok_or("invalid url")?;
    let ans = yajilin2::solve_yajilin2(outside, &problem);

    let height = problem.len();
    let width = problem[0].len();
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&ans));

    if let Some(ref ans) = ans {
        let mut skip_line = vec![];
        for y in 0..height {
            let mut row = vec![];
            for x in 0..width {
                row.push(problem[y][x].is_some() || ans.1[y][x] == Some(true));
            }
            skip_line.push(row);
        }
        for y in 0..height {
            for x in 0..width {
                if let Some(clue) = problem[y][x] {
                    let arrow = match clue.0 {
                        Arrow::Unspecified => None,
                        Arrow::Up => Some(ItemKind::SideArrowUp),
                        Arrow::Down => Some(ItemKind::SideArrowDown),
                        Arrow::Left => Some(ItemKind::SideArrowLeft),
                        Arrow::Right => Some(ItemKind::SideArrowRight),
                    };
                    let n = clue.1;
                    if let Some(arrow) = arrow {
                        board.push(Item::cell(y, x, "black", arrow));
                    }
                    board.push(Item::cell(
                        y,
                        x,
                        "black",
                        if n >= 0 {
                            ItemKind::Num(n)
                        } else {
                            ItemKind::Text("?")
                        },
                    ));
                } else if let Some(b) = ans.1[y][x] {
                    board.push(Item::cell(
                        y,
                        x,
                        "green",
                        if b { ItemKind::Block } else { ItemKind::Dot },
                    ));
                }
            }
        }

        board.add_lines_irrefutable_facts(&ans.0, "green", Some(&skip_line));
    } else {
        for y in 0..height {
            for x in 0..width {
                if let Some(clue) = problem[y][x] {
                    let arrow = match clue.0 {
                        Arrow::Unspecified => None,
                        Arrow::Up => Some(ItemKind::SideArrowUp),
                        Arrow::Down => Some(ItemKind::SideArrowDown),
                        Arrow::Left => Some(ItemKind::SideArrowLeft),
                        Arrow::Right => Some(ItemKind::SideArrowRight),
                    };
                    let n = clue.1;
                    if let Some(arrow) = arrow {
                        board.push(Item::cell(y, x, "black", arrow));
                    }
                    board.push(Item::cell(
                        y,
                        x,
                        "black",
                        if n >= 0 {
                            ItemKind::Num(n)
                        } else {
                            ItemKind::Text("?")
                        },
                    ));
                }
            }
        }
    }

    Ok(board)
}
