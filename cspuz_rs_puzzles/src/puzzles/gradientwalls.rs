// Gradient Walls puzzle solver
// Rules:
//   - Fill each empty cell with a number 1..N (N = max(rows, cols))
//   - A wall (thick border) between two adjacent cells: values differ by MORE than 1 (|a-b| > 1)
//   - An open border between two adjacent cells: values differ by AT MOST 1 (|a-b| <= 1)
//   - Clue numbers are given and cannot be changed
//   - All cells must be filled

use cspuz_rs::graph::InnerGridEdges;
use cspuz_rs::serializer::{Combinator, Context, Rooms};
use cspuz_rs::solver::Solver;

pub struct GradientWallsProblem {
    pub height: usize,
    pub width: usize,
    pub max_val: i32,
    pub clues: Vec<Vec<Option<i32>>>,
    pub borders: InnerGridEdges<Vec<Vec<bool>>>,
}

pub fn solve_gradientwalls(problem: &GradientWallsProblem) -> Option<Vec<Vec<Option<i32>>>> {
    let h = problem.height;
    let w = problem.width;
    let max_val = problem.max_val;

    let mut solver = Solver::new();
    let cell_val = &solver.int_var_2d((h, w), 1, max_val);
    solver.add_answer_key_int(cell_val);

    // Fix clue cells
    for y in 0..h {
        for x in 0..w {
            if let Some(v) = problem.clues[y][x] {
                solver.add_expr(cell_val.at((y, x)).eq(v));
            }
        }
    }

    // Wall constraint: for each pair of adjacent cells with a border between them
    // Wall present  → |a - b| > 1, i.e. a - b > 1 OR b - a > 1
    //                  i.e. NOT(-1 <= a-b <= 1), i.e. NOT(a-b >= -1 AND a-b <= 1)
    //                  equivalently: a >= b+2 OR b >= a+2
    // No wall       → |a - b| <= 1, i.e. -1 <= a-b <= 1
    //                  equivalently: a >= b-1 AND a <= b+1

    // Vertical borders (between (y,x) and (y,x+1))
    for y in 0..h {
        for x in 0..(w - 1) {
            let a = cell_val.at((y, x));
            let b = cell_val.at((y, x + 1));
            if problem.borders.vertical[y][x] {
                // Wall: |a - b| > 1 → a >= b+2 OR b >= a+2
                solver.add_expr(a.ge(b.expr() + 2) | b.ge(a.expr() + 2));
            } else {
                // No wall: |a - b| <= 1 → a >= b-1 AND a <= b+1
                solver.add_expr(a.ge(b.expr() - 1));
                solver.add_expr(a.le(b.expr() + 1));
            }
        }
    }

    // Horizontal borders (between (y,x) and (y+1,x))
    for y in 0..(h - 1) {
        for x in 0..w {
            let a = cell_val.at((y, x));
            let b = cell_val.at((y + 1, x));
            if problem.borders.horizontal[y][x] {
                // Wall: |a - b| > 1
                solver.add_expr(a.ge(b.expr() + 2) | b.ge(a.expr() + 2));
            } else {
                // No wall: |a - b| <= 1
                solver.add_expr(a.ge(b.expr() - 1));
                solver.add_expr(a.le(b.expr() + 1));
            }
        }
    }

    solver.irrefutable_facts().map(|f| f.get(cell_val))
}

pub fn deserialize_problem(url: &str) -> Option<GradientWallsProblem> {
    let url_part = if let Some(idx) = url.find("gradientwalls/") {
        &url[idx + "gradientwalls/".len()..]
    } else {
        return None;
    };

    let parts: Vec<&str> = url_part.splitn(3, '/').collect();
    if parts.len() < 3 {
        return None;
    }

    let cols: usize = parts[0].parse().ok()?;
    let rows: usize = parts[1].parse().ok()?;
    let body = parts[2];

    parse_gradientwalls_body(body, rows, cols)
}

/// URL body format: clue_grid_data + border_data
/// - Clue grid (decodeNumber16): single hex digits (0-f), letters >= 'g' skip (ch - 'f') cells
/// - Border data (decodeBorder): pzprjs Rooms encoding (MultiDigit(2,5) = base-32, 5 border bits per char)
///   First vertical borders (h × (w-1)), then horizontal borders ((h-1) × w)
fn parse_gradientwalls_body(body: &str, rows: usize, cols: usize) -> Option<GradientWallsProblem> {
    let total_cells = rows * cols;
    let bytes = body.as_bytes();
    let mut pos = 0;
    let mut clues = vec![vec![None; cols]; rows];
    let mut cell_idx = 0;
    let max_val = std::cmp::max(rows, cols) as i32;

    // Parse clue data (decodeNumber16 format)
    while cell_idx < total_cells && pos < bytes.len() {
        let ch = bytes[pos];
        pos += 1;

        if ch.is_ascii_digit() {
            let val = (ch - b'0') as i32;
            let y = cell_idx / cols;
            let x = cell_idx % cols;
            if val > 0 {
                clues[y][x] = Some(val);
            }
            cell_idx += 1;
        } else if ch >= b'g' && ch <= b'z' {
            let skip = (ch - b'f') as usize;
            cell_idx += skip;
        } else if ch >= b'a' && ch <= b'f' {
            let val = (ch - b'a') as i32 + 10;
            let y = cell_idx / cols;
            let x = cell_idx % cols;
            clues[y][x] = Some(val);
            cell_idx += 1;
        } else {
            return None;
        }
    }

    // Parse border data using pzprjs Rooms encoding (decodeBorder format)
    let border_bytes = &bytes[pos..];
    let ctx = Context::sized(rows, cols);
    let (_n_read, borders_vec) = Rooms.deserialize(&ctx, border_bytes)?;
    let borders = borders_vec.into_iter().next()?;

    Some(GradientWallsProblem {
        height: rows,
        width: cols,
        max_val,
        clues,
        borders,
    })
}

pub fn serialize_problem(problem: &GradientWallsProblem) -> Option<String> {
    let h = problem.height;
    let w = problem.width;

    // Encode clue data
    let mut body = String::new();
    let mut gap = 0;

    for y in 0..h {
        for x in 0..w {
            match problem.clues[y][x] {
                Some(v) => {
                    while gap > 0 {
                        let skip = std::cmp::min(gap, 20);
                        body.push((b'f' + skip as u8) as char);
                        gap -= skip;
                    }
                    if v < 10 {
                        body.push(char::from_digit(v as u32, 10).unwrap());
                    } else {
                        body.push((b'a' + (v - 10) as u8) as char);
                    }
                }
                None => {
                    gap += 1;
                }
            }
        }
    }
    while gap > 0 {
        let skip = std::cmp::min(gap, 20);
        body.push((b'f' + skip as u8) as char);
        gap -= skip;
    }

    // Encode border data
    let ctx = Context::sized(h, w);
    let (_, border_bytes) = Rooms.serialize(&ctx, &[problem.borders.clone()])?;
    body.push_str(std::str::from_utf8(&border_bytes).ok()?);

    Some(format!(
        "https://puzz.link/p?gradientwalls/{}/{}/{}",
        w, h, body
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_easy_problem() -> GradientWallsProblem {
        // From puzzle_gradientwalls.py easy puzzle: 6x6, max_val=4
        let h = 6;
        let w = 6;
        let mut clues = vec![vec![None; w]; h];
        // Row 0: (0,0)=1, (0,2)=1, (0,3)=4, (0,4)=3, (0,5)=4
        clues[0][0] = Some(1);
        clues[0][2] = Some(1);
        clues[0][3] = Some(4);
        clues[0][4] = Some(3);
        clues[0][5] = Some(4);
        // Row 1: (1,1)=3, (1,4)=4
        clues[1][1] = Some(3);
        clues[1][4] = Some(4);
        // Row 2: (2,0)=1, (2,4)=1
        clues[2][0] = Some(1);
        clues[2][4] = Some(1);
        // Row 3: (3,0)=4, (3,2)=4, (3,3)=1, (3,5)=1
        clues[3][0] = Some(4);
        clues[3][2] = Some(4);
        clues[3][3] = Some(1);
        clues[3][5] = Some(1);
        // Row 4: (4,5)=4
        clues[4][5] = Some(4);
        // Row 5: (5,0)=4, (5,1)=1, (5,2)=4, (5,4)=4, (5,5)=3
        clues[5][0] = Some(4);
        clues[5][1] = Some(1);
        clues[5][2] = Some(4);
        clues[5][4] = Some(4);
        clues[5][5] = Some(3);

        // Borders: derived from the solution to verify
        // Solution:
        // [1, 2, 1, 4, 3, 4]
        // [2, 3, 2, 3, 4, 3]
        // [1, 2, 3, 2, 1, 2]
        // [4, 3, 4, 1, 2, 1]
        // [3, 2, 3, 2, 3, 4]
        // [4, 1, 4, 3, 4, 3]
        // Wall if |diff| > 1, no wall if |diff| <= 1
        let solution: Vec<Vec<i32>> = vec![
            vec![1, 2, 1, 4, 3, 4],
            vec![2, 3, 2, 3, 4, 3],
            vec![1, 2, 3, 2, 1, 2],
            vec![4, 3, 4, 1, 2, 1],
            vec![3, 2, 3, 2, 3, 4],
            vec![4, 1, 4, 3, 4, 3],
        ];

        let mut vertical = vec![vec![false; w - 1]; h];
        let mut horizontal = vec![vec![false; w]; h - 1];
        for y in 0..h {
            for x in 0..(w - 1) {
                vertical[y][x] = (solution[y][x] - solution[y][x + 1]).abs() > 1;
            }
        }
        for y in 0..(h - 1) {
            for x in 0..w {
                horizontal[y][x] = (solution[y][x] - solution[y + 1][x]).abs() > 1;
            }
        }

        GradientWallsProblem {
            height: h,
            width: w,
            max_val: 4,
            clues,
            borders: InnerGridEdges {
                vertical,
                horizontal,
            },
        }
    }

    fn make_medium_problem() -> GradientWallsProblem {
        let h = 8;
        let w = 8;
        let mut clues = vec![vec![None; w]; h];
        clues[0][0] = Some(1);
        clues[0][4] = Some(5);
        clues[1][1] = Some(1);
        clues[1][3] = Some(3);
        clues[1][5] = Some(5);
        clues[1][7] = Some(1);
        clues[2][2] = Some(5);
        clues[2][6] = Some(1);
        clues[3][3] = Some(5);
        clues[4][0] = Some(5);
        clues[4][4] = Some(1);
        clues[5][1] = Some(5);
        clues[5][5] = Some(1);
        clues[5][7] = Some(5);
        clues[6][2] = Some(1);
        clues[6][6] = Some(5);
        clues[7][0] = Some(2);
        clues[7][3] = Some(1);
        clues[7][4] = Some(4);
        clues[7][7] = Some(3);

        let solution: Vec<Vec<i32>> = vec![
            vec![1, 2, 3, 4, 5, 4, 3, 2],
            vec![2, 1, 4, 3, 4, 5, 2, 1],
            vec![3, 2, 5, 4, 3, 4, 1, 2],
            vec![4, 3, 4, 5, 2, 3, 2, 3],
            vec![5, 4, 3, 4, 1, 2, 3, 4],
            vec![4, 5, 2, 3, 2, 1, 4, 5],
            vec![3, 4, 1, 2, 3, 2, 5, 4],
            vec![2, 3, 2, 1, 4, 3, 4, 3],
        ];

        let mut vertical = vec![vec![false; w - 1]; h];
        let mut horizontal = vec![vec![false; w]; h - 1];
        for y in 0..h {
            for x in 0..(w - 1) {
                vertical[y][x] = (solution[y][x] - solution[y][x + 1]).abs() > 1;
            }
        }
        for y in 0..(h - 1) {
            for x in 0..w {
                horizontal[y][x] = (solution[y][x] - solution[y + 1][x]).abs() > 1;
            }
        }

        GradientWallsProblem {
            height: h,
            width: w,
            max_val: 5,
            clues,
            borders: InnerGridEdges {
                vertical,
                horizontal,
            },
        }
    }

    fn make_hard_problem() -> GradientWallsProblem {
        let h = 10;
        let w = 10;
        let mut clues = vec![vec![None; w]; h];
        clues[0][2] = Some(1);
        clues[0][3] = Some(4);
        clues[0][5] = Some(6);
        clues[0][8] = Some(1);
        clues[0][9] = Some(2);
        clues[1][4] = Some(4);
        clues[1][9] = Some(1);
        clues[2][3] = Some(6);
        clues[3][0] = Some(6);
        clues[3][4] = Some(6);
        clues[3][6] = Some(6);
        clues[4][1] = Some(6);
        clues[4][7] = Some(6);
        clues[5][2] = Some(6);
        clues[5][5] = Some(1);
        clues[5][8] = Some(6);
        clues[6][9] = Some(6);
        clues[7][3] = Some(1);
        clues[8][0] = Some(1);
        clues[8][4] = Some(1);
        clues[8][6] = Some(1);
        clues[9][0] = Some(2);
        clues[9][1] = Some(1);
        clues[9][3] = Some(3);
        clues[9][5] = Some(5);
        clues[9][6] = Some(2);
        clues[9][7] = Some(1);

        let solution: Vec<Vec<i32>> = vec![
            vec![3, 2, 1, 4, 5, 6, 3, 2, 1, 2],
            vec![4, 3, 2, 5, 4, 5, 4, 3, 2, 1],
            vec![5, 4, 3, 6, 5, 4, 5, 4, 3, 2],
            vec![6, 5, 4, 5, 6, 3, 6, 5, 4, 3],
            vec![5, 6, 5, 4, 5, 2, 5, 6, 5, 4],
            vec![4, 5, 6, 3, 4, 1, 4, 5, 6, 5],
            vec![3, 4, 5, 2, 3, 2, 3, 4, 5, 6],
            vec![2, 3, 4, 1, 2, 3, 2, 3, 4, 5],
            vec![1, 2, 3, 2, 1, 4, 1, 2, 3, 4],
            vec![2, 1, 2, 3, 2, 5, 2, 1, 2, 3],
        ];

        let mut vertical = vec![vec![false; w - 1]; h];
        let mut horizontal = vec![vec![false; w]; h - 1];
        for y in 0..h {
            for x in 0..(w - 1) {
                vertical[y][x] = (solution[y][x] - solution[y][x + 1]).abs() > 1;
            }
        }
        for y in 0..(h - 1) {
            for x in 0..w {
                horizontal[y][x] = (solution[y][x] - solution[y + 1][x]).abs() > 1;
            }
        }

        GradientWallsProblem {
            height: h,
            width: w,
            max_val: 6,
            clues,
            borders: InnerGridEdges {
                vertical,
                horizontal,
            },
        }
    }

    #[test]
    fn test_solve_easy() {
        let problem = make_easy_problem();
        let ans = solve_gradientwalls(&problem);
        assert!(ans.is_some(), "Easy puzzle should be solvable");
        let grid = ans.unwrap();
        let expected = vec![
            vec![1, 2, 1, 4, 3, 4],
            vec![2, 3, 2, 3, 4, 3],
            vec![1, 2, 3, 2, 1, 2],
            vec![4, 3, 4, 1, 2, 1],
            vec![3, 2, 3, 2, 3, 4],
            vec![4, 1, 4, 3, 4, 3],
        ];
        for y in 0..6 {
            for x in 0..6 {
                assert_eq!(
                    grid[y][x],
                    Some(expected[y][x]),
                    "Cell ({},{}) should be {} but got {:?}",
                    y,
                    x,
                    expected[y][x],
                    grid[y][x]
                );
            }
        }
    }

    #[test]
    fn test_solve_medium() {
        let problem = make_medium_problem();
        let ans = solve_gradientwalls(&problem);
        assert!(ans.is_some(), "Medium puzzle should be solvable");
        let grid = ans.unwrap();
        let expected = vec![
            vec![1, 2, 3, 4, 5, 4, 3, 2],
            vec![2, 1, 4, 3, 4, 5, 2, 1],
            vec![3, 2, 5, 4, 3, 4, 1, 2],
            vec![4, 3, 4, 5, 2, 3, 2, 3],
            vec![5, 4, 3, 4, 1, 2, 3, 4],
            vec![4, 5, 2, 3, 2, 1, 4, 5],
            vec![3, 4, 1, 2, 3, 2, 5, 4],
            vec![2, 3, 2, 1, 4, 3, 4, 3],
        ];
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(
                    grid[y][x],
                    Some(expected[y][x]),
                    "Cell ({},{}) should be {} but got {:?}",
                    y,
                    x,
                    expected[y][x],
                    grid[y][x]
                );
            }
        }
    }

    #[test]
    fn test_solve_hard() {
        let problem = make_hard_problem();
        let ans = solve_gradientwalls(&problem);
        assert!(ans.is_some(), "Hard puzzle should be solvable");
        let grid = ans.unwrap();
        let expected = vec![
            vec![3, 2, 1, 4, 5, 6, 3, 2, 1, 2],
            vec![4, 3, 2, 5, 4, 5, 4, 3, 2, 1],
            vec![5, 4, 3, 6, 5, 4, 5, 4, 3, 2],
            vec![6, 5, 4, 5, 6, 3, 6, 5, 4, 3],
            vec![5, 6, 5, 4, 5, 2, 5, 6, 5, 4],
            vec![4, 5, 6, 3, 4, 1, 4, 5, 6, 5],
            vec![3, 4, 5, 2, 3, 2, 3, 4, 5, 6],
            vec![2, 3, 4, 1, 2, 3, 2, 3, 4, 5],
            vec![1, 2, 3, 2, 1, 4, 1, 2, 3, 4],
            vec![2, 1, 2, 3, 2, 5, 2, 1, 2, 3],
        ];
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    grid[y][x],
                    Some(expected[y][x]),
                    "Cell ({},{}) should be {} but got {:?}",
                    y,
                    x,
                    expected[y][x],
                    grid[y][x]
                );
            }
        }
    }

    #[test]
    fn test_deserialize_easy_url() {
        let url = "http://localhost:8000/p.html?gradientwalls/6/6/1g1434g3h4g1i1g4g41g1k4414g4340040o00k020";
        let problem = deserialize_problem(url);
        assert!(problem.is_some(), "Should parse easy URL");
        let p = problem.unwrap();
        assert_eq!(p.height, 6);
        assert_eq!(p.width, 6);
        assert_eq!(p.clues[0][0], Some(1));
        assert_eq!(p.clues[0][1], None);
        assert_eq!(p.clues[0][2], Some(1));
        assert_eq!(p.clues[0][3], Some(4));
    }

    #[test]
    fn test_deserialize_and_solve_easy() {
        let url = "http://localhost:8000/p.html?gradientwalls/6/6/1g1434g3h4g1i1g4g41g1k4414g4340040o00k020";
        let problem = deserialize_problem(url).unwrap();
        let ans = solve_gradientwalls(&problem);
        assert!(ans.is_some(), "Easy URL puzzle should be solvable");
        let grid = ans.unwrap();
        // Check all cells are determined (unique solution)
        for y in 0..6 {
            for x in 0..6 {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
    }

    #[test]
    fn test_deserialize_and_solve_medium() {
        let url = "http://localhost:8000/p.html?gradientwalls/8/8/1i5j1g3g5g1h5i1j5j5i1j5i1g5h1i5g2h14h3024h1088i440000000000000";
        let problem = deserialize_problem(url).unwrap();
        let ans = solve_gradientwalls(&problem);
        assert!(ans.is_some(), "Medium URL puzzle should be solvable");
        let grid = ans.unwrap();
        for y in 0..8 {
            for x in 0..8 {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
    }

    #[test]
    fn test_deserialize_and_solve_hard() {
        let url = "http://localhost:8000/p.html?gradientwalls/10/10/h14g6h12j4j1i6l6i6g6j6k6j6h1h6p6i1l1i1g1i21g3g521h4g80g0c0o5g80g0c0o000000000000000000";
        let problem = deserialize_problem(url).unwrap();
        let ans = solve_gradientwalls(&problem);
        assert!(ans.is_some(), "Hard URL puzzle should be solvable");
        let grid = ans.unwrap();
        for y in 0..10 {
            for x in 0..10 {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
    }
}
