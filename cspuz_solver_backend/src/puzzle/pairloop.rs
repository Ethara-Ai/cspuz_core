use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::{check_uniqueness, Uniqueness};
use cspuz_rs_puzzles::puzzles::pairloop;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = pairloop::deserialize_problem(url).ok_or("invalid url")?;
    let is_line = pairloop::solve_pairloop(&problem);

    let height = problem.len();
    let width = problem[0].len();
    let mut board = Board::new(
        BoardKind::DotGrid,
        height,
        width,
        check_uniqueness(&is_line),
    );

    for y in 0..height {
        for x in 0..width {
            if let Some(clue) = problem[y][x] {
                if clue >= 0 && clue <= 4 {
                    board.push(Item::cell(y, x, "black", ItemKind::Num(clue)));
                } else if clue >= 5 && clue <= 8 {
                    let arrow = match clue {
                        5 => "↑",
                        6 => "→",
                        7 => "↓",
                        8 => "←",
                        _ => unreachable!(),
                    };
                    board.push(Item::cell(y, x, "black", ItemKind::Text(arrow)));
                }
            }
        }
    }
    if let Some(is_line) = &is_line {
        for y in 0..height {
            for x in 0..=width {
                if let Some(b) = is_line.vertical[y][x] {
                    board.push(Item {
                        y: y * 2 + 1,
                        x: x * 2,
                        color: "green",
                        kind: if b { ItemKind::Wall } else { ItemKind::Cross },
                    })
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
                    })
                }
            }
        }
    }

    Ok(board)
}

pub fn enumerate(url: &str, num_max_answers: usize) -> Result<(Board, Vec<Board>), &'static str> {
    let problem = pairloop::deserialize_problem(url).ok_or("invalid url")?;
    let answer_common = pairloop::solve_pairloop(&problem).ok_or("no answer")?;
    let answers = pairloop::enumerate_answers_pairloop(&problem, num_max_answers);

    let height = problem.len();
    let width = problem[0].len();
    let mut board_common = Board::new(BoardKind::DotGrid, height, width, Uniqueness::NotApplicable);

    for y in 0..height {
        for x in 0..width {
            if let Some(clue) = problem[y][x] {
                if clue >= 0 && clue <= 4 {
                    board_common.push(Item::cell(y, x, "black", ItemKind::Num(clue)));
                } else if clue >= 5 && clue <= 8 {
                    let arrow = match clue {
                        5 => "↑",
                        6 => "→",
                        7 => "↓",
                        8 => "←",
                        _ => unreachable!(),
                    };
                    board_common.push(Item::cell(y, x, "black", ItemKind::Text(arrow)));
                }
            }
        }
    }
    for y in 0..height {
        for x in 0..=width {
            if let Some(b) = answer_common.vertical[y][x] {
                board_common.push(Item {
                    y: y * 2 + 1,
                    x: x * 2,
                    color: "black",
                    kind: if b { ItemKind::Wall } else { ItemKind::Cross },
                })
            }
        }
    }
    for y in 0..=height {
        for x in 0..width {
            if let Some(b) = answer_common.horizontal[y][x] {
                board_common.push(Item {
                    y: y * 2,
                    x: x * 2 + 1,
                    color: "black",
                    kind: if b { ItemKind::Wall } else { ItemKind::Cross },
                })
            }
        }
    }

    let mut board_answers = vec![];
    for ans in answers {
        let mut board_answer =
            Board::new(BoardKind::Empty, height, width, Uniqueness::NotApplicable);
        for y in 0..height {
            for x in 0..=width {
                if answer_common.vertical[y][x].is_some() {
                    continue;
                }
                let b = ans.vertical[y][x];
                board_answer.push(Item {
                    y: y * 2 + 1,
                    x: x * 2,
                    color: "green",
                    kind: if b { ItemKind::Wall } else { ItemKind::Cross },
                });
            }
        }
        for y in 0..=height {
            for x in 0..width {
                if answer_common.horizontal[y][x].is_some() {
                    continue;
                }
                let b = ans.horizontal[y][x];
                board_answer.push(Item {
                    y: y * 2,
                    x: x * 2 + 1,
                    color: "green",
                    kind: if b { ItemKind::Wall } else { ItemKind::Cross },
                });
            }
        }

        board_answers.push(board_answer);
    }

    Ok((board_common, board_answers))
}
