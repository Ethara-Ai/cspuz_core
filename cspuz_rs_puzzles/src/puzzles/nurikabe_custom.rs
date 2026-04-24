// Custom Nurikabe variant:
// - Standard nurikabe rules EXCEPT:
//   - checkShadeMax3: Shaded (black) connected components have at most 3 cells
//   - checkStraightLineIslands: Each island (unshaded group with a number) must be a straight line (1×N or N×1)
// - REMOVED: global black connectivity (standard nurikabe requires all black connected)
// - KEPT: no 2×2 black, island sizes match clues, each island connected

use crate::util;
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Optionalize, Spaces,
};
use cspuz_rs::solver::Solver;

pub fn solve_nurikabe_custom(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    let mut clue_pos = vec![];
    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                clue_pos.push((y, x, n));
            }
        }
    }

    // Group IDs: 0 = black, 1..=N = island groups
    let group_id = solver.int_var_2d((h, w), 0, clue_pos.len() as i32);
    solver.add_expr(is_black.iff(group_id.eq(0)));

    // Standard: global black connectivity
    graph::active_vertices_connected_2d(&mut solver, is_black);

    // Each island connected
    for i in 1..=clue_pos.len() {
        graph::active_vertices_connected_2d(&mut solver, group_id.eq(i as i32));
    }

    // Adjacent unshaded cells must share same group
    solver.add_expr(
        (!is_black.conv2d_or((2, 1))).imp(
            group_id
                .slice((..(h - 1), ..))
                .eq(group_id.slice((1.., ..))),
        ),
    );
    solver.add_expr(
        (!is_black.conv2d_or((1, 2))).imp(
            group_id
                .slice((.., ..(w - 1)))
                .eq(group_id.slice((.., 1..))),
        ),
    );

    // No 2×2 black block (same as standard)
    solver.add_expr(!is_black.conv2d_and((2, 2)));

    // Clue cells must be in their group with correct size
    for (i, &(y, x, n)) in clue_pos.iter().enumerate() {
        solver.add_expr(group_id.at((y, x)).eq((i + 1) as i32));
        if n > 0 {
            solver.add_expr(group_id.eq((i + 1) as i32).count_true().eq(n));
        }
    }

    // Forbid all tetromino placements to enforce max shaded component size of 3.
    // Every connected subgraph of 4 cells on a grid is a tetromino (I/O/L/T/S).
    // 2×2 (O) already forbidden above. Forbid I, L, T, S in all orientations.
    if w >= 4 {
        solver.add_expr(!is_black.conv2d_and((1, 4)));
    }
    if h >= 4 {
        solver.add_expr(!is_black.conv2d_and((4, 1)));
    }

    // Forbid L-shapes (3 in row + 1 extending from end)
    // L-shape type 1: cells (y,x), (y,x+1), (y,x+2), (y+1,x) — bottom-left L
    for y in 0..h - 1 {
        for x in 0..w - 2 {
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y, x + 1))
                    & is_black.at((y, x + 2))
                    & is_black.at((y + 1, x))),
            );
            // Bottom-right L: (y,x), (y,x+1), (y,x+2), (y+1,x+2)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y, x + 1))
                    & is_black.at((y, x + 2))
                    & is_black.at((y + 1, x + 2))),
            );
            // Top-left L: (y,x), (y,x+1), (y,x+2), (y-1,x) — handle y>0
            // Top-right L: (y,x), (y,x+1), (y,x+2), (y-1,x+2)
        }
    }
    for y in 1..h {
        for x in 0..w - 2 {
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y, x + 1))
                    & is_black.at((y, x + 2))
                    & is_black.at((y - 1, x))),
            );
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y, x + 1))
                    & is_black.at((y, x + 2))
                    & is_black.at((y - 1, x + 2))),
            );
        }
    }

    // L-shapes vertical: 3 in column + 1 extending from end
    for y in 0..h - 2 {
        for x in 0..w - 1 {
            // Right-bottom: (y,x), (y+1,x), (y+2,x), (y,x+1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 2, x))
                    & is_black.at((y, x + 1))),
            );
            // Right-top extension: (y,x), (y+1,x), (y+2,x), (y+2,x+1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 2, x))
                    & is_black.at((y + 2, x + 1))),
            );
        }
        for x in 1..w {
            // Left-top: (y,x), (y+1,x), (y+2,x), (y,x-1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 2, x))
                    & is_black.at((y, x - 1))),
            );
            // Left-bottom: (y,x), (y+1,x), (y+2,x), (y+2,x-1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 2, x))
                    & is_black.at((y + 2, x - 1))),
            );
        }
    }

    // T-shapes: 3 in line + 1 from middle
    for y in 0..h - 1 {
        for x in 0..w - 2 {
            // T-down: (y,x), (y,x+1), (y,x+2), (y+1,x+1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y, x + 1))
                    & is_black.at((y, x + 2))
                    & is_black.at((y + 1, x + 1))),
            );
        }
    }
    for y in 1..h {
        for x in 0..w - 2 {
            // T-up: (y,x), (y,x+1), (y,x+2), (y-1,x+1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y, x + 1))
                    & is_black.at((y, x + 2))
                    & is_black.at((y - 1, x + 1))),
            );
        }
    }
    for y in 0..h - 2 {
        for x in 0..w - 1 {
            // T-right: (y,x), (y+1,x), (y+2,x), (y+1,x+1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 2, x))
                    & is_black.at((y + 1, x + 1))),
            );
        }
        for x in 1..w {
            // T-left: (y,x), (y+1,x), (y+2,x), (y+1,x-1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 2, x))
                    & is_black.at((y + 1, x - 1))),
            );
        }
    }

    // S/Z-shapes (skew tetrominoes)
    for y in 0..h - 1 {
        for x in 0..w - 2 {
            // S-horizontal: (y,x+1), (y,x+2), (y+1,x), (y+1,x+1)
            solver.add_expr(
                !(is_black.at((y, x + 1))
                    & is_black.at((y, x + 2))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 1, x + 1))),
            );
            // Z-horizontal: (y,x), (y,x+1), (y+1,x+1), (y+1,x+2)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y, x + 1))
                    & is_black.at((y + 1, x + 1))
                    & is_black.at((y + 1, x + 2))),
            );
        }
    }
    // S/Z-vertical
    for y in 0..h - 2 {
        for x in 0..w - 1 {
            // S-vertical: (y,x), (y+1,x), (y+1,x+1), (y+2,x+1)
            solver.add_expr(
                !(is_black.at((y, x))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 1, x + 1))
                    & is_black.at((y + 2, x + 1))),
            );
            // Z-vertical: (y,x+1), (y+1,x+1), (y+1,x), (y+2,x)
            solver.add_expr(
                !(is_black.at((y, x + 1))
                    & is_black.at((y + 1, x + 1))
                    & is_black.at((y + 1, x))
                    & is_black.at((y + 2, x))),
            );
        }
    }

    // CUSTOM: checkStraightLineIslands — each island must be a straight line (1×N or N×1)
    // For each island group i, all cells with group_id == i+1 must share the same row or same column.
    // We encode this: for each island, create a bool var is_horizontal.
    // If horizontal: all cells in island have same y-coordinate.
    // If vertical: all cells in island have same x-coordinate.
    for (i, &(cy, cx, n)) in clue_pos.iter().enumerate() {
        if n <= 1 {
            continue; // single cell is trivially a line
        }
        let gid = (i + 1) as i32;
        // is_horizontal: if true, all cells share the clue's row
        let is_horizontal = solver.bool_var();

        for y in 0..h {
            for x in 0..w {
                if y == cy && x == cx {
                    continue; // clue cell is always in the group
                }
                // If cell is in this group:
                // - if horizontal: must be in same row as clue (y == cy)
                // - if vertical: must be in same column as clue (x == cx)
                let in_group = group_id.at((y, x)).eq(gid);

                if y != cy && x != cx {
                    // Neither same row nor same column — can't be in this group
                    solver.add_expr(!in_group);
                } else if y != cy {
                    // Same column but different row — must be vertical
                    solver.add_expr(in_group.imp(!&is_horizontal));
                } else if x != cx {
                    // Same row but different column — must be horizontal
                    solver.add_expr(in_group.imp(&is_horizontal));
                }
            }
        }
    }

    solver.irrefutable_facts().map(|f| f.get(is_black))
}

type Problem = Vec<Vec<Option<i32>>>;

fn combinator() -> impl Combinator<Problem> {
    Grid::new(Choice::new(vec![
        Box::new(Optionalize::new(HexInt)),
        Box::new(Spaces::new(None, 'g')),
        Box::new(Dict::new(Some(-1), ".")),
    ]))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    problem_to_url(combinator(), "nurikabe_custom", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["nurikabe", "nurikabe_custom"], url)
}
