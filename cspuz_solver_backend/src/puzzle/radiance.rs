use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::{check_uniqueness, Uniqueness};
use cspuz_rs_puzzles::puzzles::radiance::{self, CellKind, MIRROR_SLASH};

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let problem = radiance::deserialize_problem(url).ok_or("invalid url")?;
    let ans = radiance::solve_radiance(&problem);

    let height = problem.height;
    let width = problem.width;

    let uniqueness = match &ans {
        None => Uniqueness::NoAnswer,
        Some(grid) => {
            let slot_values: Vec<Option<i32>> = problem
                .mirror_slots()
                .iter()
                .map(|&(y, x)| grid[y][x])
                .collect();
            check_uniqueness(&Some(slot_values))
        }
    };

    let mut board = Board::new(BoardKind::Grid, height, width, uniqueness);

    for y in 0..height {
        for x in 0..width {
            match &problem.grid[y][x] {
                CellKind::Emitter(dir) => {
                    board.push(Item::cell(y, x, "black", ItemKind::Fill));
                    let arrow = match *dir {
                        1 => ItemKind::ArrowUp,
                        2 => ItemKind::ArrowDown,
                        3 => ItemKind::ArrowLeft,
                        4 => ItemKind::ArrowRight,
                        _ => continue,
                    };
                    board.push(Item::cell(y, x, "white", arrow));
                }
                CellKind::Target => {
                    board.push(Item::cell(y, x, "red", ItemKind::FilledCircle));
                }
                CellKind::MirrorSlot => {
                    board.push(Item::cell(y, x, "#ccc", ItemKind::SmallFilledCircle));
                }
                CellKind::Empty => {}
            }
        }
    }

    if let Some(ref ans) = ans {
        for y in 0..height {
            for x in 0..width {
                if problem.grid[y][x] == CellKind::MirrorSlot {
                    if let Some(mirror) = ans[y][x] {
                        board.push(Item::cell(
                            y,
                            x,
                            "green",
                            if mirror == MIRROR_SLASH {
                                ItemKind::Slash
                            } else {
                                ItemKind::Backslash
                            },
                        ));
                    }
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
    fn test_radiance_easy() {
        let url = "https://pzprxs.vercel.app/p?radiance/6/6/14d3l2d3l";
        let board = solve(url).unwrap();
        assert_eq!(board.uniqueness, Uniqueness::Unique);
    }

    #[test]
    fn test_radiance_medium() {
        let url = "https://pzprxs.vercel.app/p?radiance/8/8/14f3p3f3x2g";
        let board = solve(url).unwrap();
        assert_eq!(board.uniqueness, Uniqueness::Unique);
    }

    #[test]
    fn test_radiance_hard() {
        let url = "https://pzprxs.vercel.app/p?radiance/10/10/14h3zd3h3t3h3s2";
        let board = solve(url).unwrap();
        assert_eq!(board.uniqueness, Uniqueness::Unique);
    }
}
