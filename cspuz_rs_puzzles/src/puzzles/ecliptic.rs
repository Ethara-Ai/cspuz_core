// Ecliptic puzzle solver
// Rules:
//   1. Shade some cells on the grid
//   2. No 2x2 block of shaded cells
//   3. All unshaded cells must form a single connected group
//   4. Each number indicates how many of its orthogonal neighbors are shaded
//   5. Numbered cells are always unshaded
//   6. Every row has the same number of shaded cells, every column has the same
//      number of shaded cells, and these two counts are equal
//   7. At least one shaded cell must exist

use crate::util;
use cspuz_rs::graph;
use cspuz_rs::solver::{count_true, Solver};

pub type Problem = Vec<Vec<Option<i32>>>;

pub fn solve_ecliptic(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_shaded = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_shaded);

    // Rule 5: Numbered cells cannot be shaded
    for y in 0..h {
        for x in 0..w {
            if clues[y][x].is_some() {
                solver.add_expr(!is_shaded.at((y, x)));
            }
        }
    }

    // Rule 2: No 2x2 block of shaded cells
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            solver.add_expr(
                !(is_shaded.at((y, x))
                    & is_shaded.at((y, x + 1))
                    & is_shaded.at((y + 1, x))
                    & is_shaded.at((y + 1, x + 1))),
            );
        }
    }

    // Rule 3: All unshaded cells must be connected
    graph::active_vertices_connected_2d(&mut solver, !is_shaded);

    // Rule 4: Each number = count of shaded orthogonal neighbors
    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                let mut neighbors = vec![];
                if y > 0 {
                    neighbors.push(is_shaded.at((y - 1, x)));
                }
                if y + 1 < h {
                    neighbors.push(is_shaded.at((y + 1, x)));
                }
                if x > 0 {
                    neighbors.push(is_shaded.at((y, x - 1)));
                }
                if x + 1 < w {
                    neighbors.push(is_shaded.at((y, x + 1)));
                }
                solver.add_expr(count_true(&neighbors).eq(n));
            }
        }
    }

    // Rule 6: Every row has the same shaded count, every column has the same,
    // and row count == column count.
    // We use an int variable K for the common count.
    let max_k = h.min(w) as i32; // K can't exceed the smaller dimension
    let k = solver.int_var(1, max_k); // Rule 7: at least 1 shaded, so K >= 1

    // Each row has exactly K shaded cells
    for y in 0..h {
        let row_cells: Vec<_> = (0..w).map(|x| is_shaded.at((y, x))).collect();
        solver.add_expr(count_true(&row_cells).eq(k.expr()));
    }

    // Each column has exactly K shaded cells
    for x in 0..w {
        let col_cells: Vec<_> = (0..h).map(|y| is_shaded.at((y, x))).collect();
        solver.add_expr(count_true(&col_cells).eq(k.expr()));
    }

    solver.irrefutable_facts().map(|f| f.get(is_shaded))
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    let prefixes = ["ecliptic"];
    let mut remaining = None;
    for prefix in &prefixes {
        let pat = format!("{}/", prefix);
        if let Some(pos) = url.find(&pat) {
            remaining = Some(&url[pos + pat.len()..]);
            break;
        }
    }
    let remaining = remaining?;

    // Parse cols/rows
    let parts: Vec<&str> = remaining.splitn(3, '/').collect();
    if parts.len() < 3 {
        return None;
    }
    let cols: usize = parts[0].parse().ok()?;
    let rows: usize = parts[1].parse().ok()?;
    let body = parts[2];

    // Parse clues using decodeNumber16 format
    let clues = parse_ecliptic_body(body, rows, cols)?;
    Some(clues)
}

fn parse_ecliptic_body(body: &str, rows: usize, cols: usize) -> Option<Vec<Vec<Option<i32>>>> {
    let mut clues = vec![vec![None; cols]; rows];
    let total = rows * cols;
    let mut idx = 0;

    for ch in body.chars() {
        if idx >= total {
            break;
        }
        if ch.is_ascii_digit() {
            let val = (ch as i32) - ('0' as i32);
            let y = idx / cols;
            let x = idx % cols;
            clues[y][x] = Some(val);
            idx += 1;
        } else if ('a'..='f').contains(&ch) {
            let val = (ch as i32) - ('a' as i32) + 10;
            let y = idx / cols;
            let x = idx % cols;
            clues[y][x] = Some(val);
            idx += 1;
        } else if ('g'..='z').contains(&ch) {
            let skip = (ch as usize) - ('f' as usize);
            idx += skip;
        }
    }

    Some(clues)
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    let (h, w) = util::infer_shape(problem);
    let body = encode_ecliptic_body(problem, h, w);
    Some(format!("ecliptic/{}/{}/{}", w, h, body))
}

fn encode_ecliptic_body(clues: &[Vec<Option<i32>>], rows: usize, cols: usize) -> String {
    let mut result = String::new();
    let mut gap = 0;

    for y in 0..rows {
        for x in 0..cols {
            if let Some(val) = clues[y][x] {
                if gap > 0 {
                    while gap > 20 {
                        result.push('z');
                        gap -= 20;
                    }
                    if gap > 0 {
                        result.push((b'f' + gap as u8) as char);
                    }
                    gap = 0;
                }
                if val < 10 {
                    result.push((b'0' + val as u8) as char);
                } else {
                    result.push((b'a' + (val - 10) as u8) as char);
                }
            } else {
                gap += 1;
            }
        }
    }
    // Trailing gap can be omitted
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_easy_problem() -> Problem {
        // 6x6, K=2
        let mut clues = vec![vec![None; 6]; 6];
        clues[0][2] = Some(2);
        clues[0][4] = Some(0);
        clues[1][5] = Some(0);
        clues[2][0] = Some(1);
        clues[3][0] = Some(1);
        clues[3][5] = Some(1);
        clues[4][0] = Some(0);
        clues[5][2] = Some(1);
        clues
    }

    fn make_medium_problem() -> Problem {
        // 8x8, K=3, 20 clues (proven unique)
        let mut clues = vec![vec![None; 8]; 8];
        clues[0][3] = Some(2);
        clues[0][4] = Some(0);
        clues[0][5] = Some(0);
        clues[0][6] = Some(0);
        clues[0][7] = Some(0);
        clues[1][1] = Some(3);
        clues[1][4] = Some(1);
        clues[1][5] = Some(1);
        clues[1][6] = Some(0);
        clues[1][7] = Some(0);
        clues[2][4] = Some(2);
        clues[3][0] = Some(1);
        clues[3][1] = Some(1);
        clues[5][0] = Some(1);
        clues[5][5] = Some(1);
        clues[6][1] = Some(1);
        clues[6][6] = Some(3);
        clues[7][0] = Some(0);
        clues[7][1] = Some(0);
        clues[7][3] = Some(1);
        clues
    }

    fn make_hard_problem() -> Problem {
        // 10x10, K=3, 17 clues (from puzzle_ecliptic.py, verified unique)
        let mut clues = vec![vec![None; 10]; 10];
        clues[0][2] = Some(2);
        clues[0][5] = Some(1);
        clues[0][9] = Some(1);
        clues[1][0] = Some(1);
        clues[1][5] = Some(3);
        clues[1][7] = Some(2);
        clues[2][3] = Some(0);
        clues[2][8] = Some(2);
        clues[3][1] = Some(1);
        clues[4][0] = Some(1);
        clues[5][2] = Some(1);
        clues[5][7] = Some(3);
        clues[6][1] = Some(1);
        clues[6][3] = Some(2);
        clues[6][5] = Some(2);
        clues[6][8] = Some(3);
        clues[7][2] = Some(1);
        clues
    }

    #[test]
    fn test_solve_easy() {
        let clues = make_easy_problem();
        let ans = solve_ecliptic(&clues);
        assert!(ans.is_some(), "Easy puzzle should be solvable");
        let ans = ans.unwrap();
        for y in 0..6 {
            for x in 0..6 {
                assert!(
                    ans[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
        assert_eq!(ans[0][0], Some(true));
        assert_eq!(ans[0][1], Some(true));
        assert_eq!(ans[1][0], Some(true));
        assert_eq!(ans[1][2], Some(true));
    }

    #[test]
    fn test_solve_medium() {
        let clues = make_medium_problem();
        let ans = solve_ecliptic(&clues);
        assert!(ans.is_some(), "Medium puzzle should be solvable");
        let ans = ans.unwrap();
        for y in 0..8 {
            for x in 0..8 {
                assert!(
                    ans[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
        assert_eq!(ans[0][0], Some(true));
        assert_eq!(ans[0][1], Some(true));
        assert_eq!(ans[0][2], Some(true));
        assert_eq!(ans[1][0], Some(true));
        assert_eq!(ans[1][2], Some(true));
        assert_eq!(ans[1][3], Some(true));
    }

    #[test]
    fn test_solve_hard() {
        let clues = make_hard_problem();
        let ans = solve_ecliptic(&clues);
        assert!(ans.is_some(), "Hard puzzle should be solvable");
        let ans = ans.unwrap();
        for y in 0..10 {
            for x in 0..10 {
                assert!(
                    ans[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
        assert_eq!(ans[0][1], Some(true));
        assert_eq!(ans[0][3], Some(true));
        assert_eq!(ans[0][4], Some(true));
    }

    #[test]
    fn test_deserialize_easy_url() {
        let url = "http://localhost:8000/p.html?ecliptic/6/6/h2g0l01k1j10m1i";
        let problem = deserialize_problem(url);
        assert!(problem.is_some());
        let clues = problem.unwrap();
        assert_eq!(clues[0][2], Some(2));
        assert_eq!(clues[0][4], Some(0));
        assert_eq!(clues[1][5], Some(0));
        assert_eq!(clues[2][0], Some(1));
        assert_eq!(clues[3][0], Some(1));
        assert_eq!(clues[3][5], Some(1));
        assert_eq!(clues[4][0], Some(0));
        assert_eq!(clues[5][2], Some(1));
    }

    #[test]
    fn test_solve_easy_from_url() {
        let url = "http://localhost:8000/p.html?ecliptic/6/6/h2g0l01k1j10m1i";
        let clues = deserialize_problem(url).unwrap();
        let ans = solve_ecliptic(&clues);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        for y in 0..6 {
            for x in 0..6 {
                assert!(
                    ans[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
    }

    #[test]
    fn test_solve_medium_from_url() {
        let url = "http://localhost:8000/p.html?ecliptic/8/8/i20000g3h1100j2i11t1j1i1j3g00g1";
        let clues = deserialize_problem(url).unwrap();
        let ans = solve_ecliptic(&clues);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        for y in 0..8 {
            for x in 0..8 {
                assert!(
                    ans[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
    }

    #[test]
    fn test_solve_hard_from_url() {
        let url = "http://localhost:8000/p.html?ecliptic/10/10/h2h1i11j3g2k0j2h1n1q1j3i1g2g2h3i1";
        let clues = deserialize_problem(url).unwrap();
        let ans = solve_ecliptic(&clues);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        for y in 0..10 {
            for x in 0..10 {
                assert!(
                    ans[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
    }
}
