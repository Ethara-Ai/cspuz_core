// Kageboshi puzzle solver
// Rules:
//   1. Shade some cells on the grid
//   2. No two shaded cells may be orthogonally adjacent
//   3. All unshaded cells must form a single connected group
//   4. Each number indicates the total count of shaded cells in its row + column
//      (excluding the cell itself)
//   5. Numbered cells are always unshaded
//   6. At least one shaded cell must exist

use crate::util;
use cspuz_rs::graph;
use cspuz_rs::solver::Solver;

pub type Problem = Vec<Vec<Option<i32>>>;

pub fn solve_kageboshi(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
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

    // Rule 2: No two shaded cells orthogonally adjacent
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w {
                solver.add_expr(!(is_shaded.at((y, x)) & is_shaded.at((y, x + 1))));
            }
            if y + 1 < h {
                solver.add_expr(!(is_shaded.at((y, x)) & is_shaded.at((y + 1, x))));
            }
        }
    }

    // Rule 3: All unshaded cells must be connected
    graph::active_vertices_connected_2d(&mut solver, !is_shaded);

    // Rule 4: Each number = count of shaded cells in same row + column (excluding self)
    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                // Collect all cells in same row (excluding self)
                let mut cross_cells = Vec::new();
                for xx in 0..w {
                    if xx != x {
                        cross_cells.push(is_shaded.at((y, xx)).ite(1, 0));
                    }
                }
                // Collect all cells in same column (excluding self)
                for yy in 0..h {
                    if yy != y {
                        cross_cells.push(is_shaded.at((yy, x)).ite(1, 0));
                    }
                }
                solver.add_expr(cspuz_rs::solver::sum(&cross_cells).eq(n));
            }
        }
    }

    // Rule 6: At least one shaded cell
    solver.add_expr(is_shaded.count_true().ge(1));

    solver.irrefutable_facts().map(|f| f.get(is_shaded))
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    // URL format: kageboshi/{cols}/{rows}/{body}
    let url_part = if let Some(idx) = url.find("kageboshi/") {
        &url[idx + "kageboshi/".len()..]
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

    parse_kageboshi_body(body, rows, cols)
}

/// Parse body using decodeNumber16 format:
/// digits 0-9 = clue values, a-f = values 10-15, g-z = skip (ch - 'f') cells
fn parse_kageboshi_body(body: &str, rows: usize, cols: usize) -> Option<Problem> {
    let total_cells = rows * cols;
    let bytes = body.as_bytes();
    let mut pos = 0;
    let mut clues = vec![vec![None; cols]; rows];
    let mut cell_idx = 0;

    while cell_idx < total_cells && pos < bytes.len() {
        let ch = bytes[pos];
        pos += 1;

        if ch.is_ascii_digit() {
            let val = (ch - b'0') as i32;
            let y = cell_idx / cols;
            let x = cell_idx % cols;
            clues[y][x] = Some(val);
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

    Some(clues)
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    let h = problem.len();
    if h == 0 {
        return None;
    }
    let w = problem[0].len();

    let mut body = String::new();
    let mut gap = 0;

    for y in 0..h {
        for x in 0..w {
            match problem[y][x] {
                Some(v) => {
                    if gap > 0 {
                        body.push((b'f' + gap as u8) as char);
                        gap = 0;
                    }
                    if v < 10 {
                        body.push(char::from_digit(v as u32, 10).unwrap());
                    } else {
                        body.push((b'a' + (v - 10) as u8) as char);
                    }
                }
                None => {
                    gap += 1;
                    if gap >= 20 {
                        body.push((b'f' + gap as u8) as char);
                        gap = 0;
                    }
                }
            }
        }
    }
    if gap > 0 {
        body.push((b'f' + gap as u8) as char);
    }

    Some(format!(
        "https://puzz.link/p?kageboshi/{}/{}/{}",
        w, h, body
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_easy_problem() -> Problem {
        // From puzzle_kageboshi.py easy: 6x6 grid
        // clues: (0,1,5) (2,3,4) (2,5,4) (3,1,5) (5,0,5) (5,4,4)
        let h = 6;
        let w = 6;
        let mut clues = vec![vec![None; w]; h];
        clues[0][1] = Some(5);
        clues[2][3] = Some(4);
        clues[2][5] = Some(4);
        clues[3][1] = Some(5);
        clues[5][0] = Some(5);
        clues[5][4] = Some(4);
        clues
    }

    fn make_medium_problem() -> Problem {
        // From puzzle_kageboshi.py medium: 8x8 grid
        let h = 8;
        let w = 8;
        let mut clues = vec![vec![None; w]; h];
        let clue_list = [
            (0, 0, 6),
            (0, 2, 6),
            (0, 4, 5),
            (0, 6, 5),
            (1, 1, 3),
            (1, 3, 3),
            (1, 5, 4),
            (1, 6, 3),
            (2, 1, 5),
            (2, 3, 5),
            (2, 4, 5),
            (2, 6, 5),
            (3, 0, 6),
            (3, 2, 6),
            (3, 3, 5),
            (3, 5, 6),
            (3, 6, 5),
            (4, 0, 5),
            (4, 1, 4),
            (4, 3, 4),
            (4, 4, 4),
            (4, 6, 4),
            (4, 7, 4),
            (5, 1, 4),
            (5, 3, 4),
            (5, 4, 4),
            (5, 5, 5),
            (5, 7, 4),
            (6, 0, 5),
            (6, 1, 4),
            (6, 3, 4),
            (6, 5, 5),
            (6, 7, 4),
            (7, 1, 5),
            (7, 4, 5),
            (7, 5, 6),
            (7, 7, 5),
        ];
        for (r, c, v) in clue_list {
            clues[r][c] = Some(v);
        }
        clues
    }

    fn make_hard_problem() -> Problem {
        // From puzzle_kageboshi.py hard: 10x10 grid
        let h = 10;
        let w = 10;
        let mut clues = vec![vec![None; w]; h];
        let clue_list = [
            (0, 0, 5),
            (0, 2, 5),
            (0, 3, 4),
            (0, 5, 5),
            (0, 6, 4),
            (0, 8, 5),
            (0, 9, 5),
            (1, 0, 3),
            (1, 1, 3),
            (1, 2, 3),
            (1, 3, 2),
            (1, 4, 4),
            (1, 5, 3),
            (1, 6, 2),
            (1, 7, 4),
            (1, 8, 3),
            (2, 0, 4),
            (2, 1, 4),
            (2, 3, 3),
            (2, 4, 5),
            (2, 5, 4),
            (2, 7, 5),
            (2, 8, 4),
            (2, 9, 4),
            (3, 1, 5),
            (3, 2, 5),
            (3, 3, 4),
            (3, 5, 5),
            (3, 6, 4),
            (3, 7, 6),
            (3, 9, 5),
            (4, 0, 2),
            (4, 1, 2),
            (4, 2, 2),
            (4, 3, 1),
            (4, 4, 3),
            (4, 5, 2),
            (4, 6, 1),
            (4, 7, 3),
            (4, 8, 2),
            (4, 9, 2),
            (5, 0, 5),
            (5, 2, 5),
            (5, 3, 4),
            (5, 4, 6),
            (5, 6, 4),
            (5, 7, 6),
            (5, 8, 5),
            (6, 0, 4),
            (6, 1, 4),
            (6, 2, 4),
            (6, 4, 5),
            (6, 5, 4),
            (6, 6, 3),
            (6, 8, 4),
            (6, 9, 4),
            (7, 1, 3),
            (7, 2, 3),
            (7, 3, 2),
            (7, 4, 4),
            (7, 5, 3),
            (7, 6, 2),
            (7, 7, 4),
            (7, 8, 3),
            (7, 9, 3),
            (8, 0, 5),
            (8, 1, 5),
            (8, 3, 4),
            (8, 4, 6),
            (8, 6, 4),
            (8, 7, 6),
            (8, 9, 5),
            (9, 0, 4),
            (9, 1, 4),
            (9, 2, 4),
            (9, 3, 3),
            (9, 5, 4),
            (9, 6, 3),
            (9, 8, 4),
            (9, 9, 4),
        ];
        for (r, c, v) in clue_list {
            clues[r][c] = Some(v);
        }
        clues
    }

    #[test]
    fn test_kageboshi_easy() {
        let problem = make_easy_problem();
        let result = solve_kageboshi(&problem);
        assert!(result.is_some(), "Easy puzzle should be solvable");
        let grid = result.unwrap();
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
        // Verify expected shaded cells from puzzle_kageboshi.py:
        let expected_shaded = [
            (0, 0),
            (0, 3),
            (0, 5),
            (1, 1),
            (2, 4),
            (3, 0),
            (3, 3),
            (3, 5),
            (5, 1),
            (5, 3),
            (5, 5),
        ];
        for &(y, x) in &expected_shaded {
            assert_eq!(
                grid[y][x],
                Some(true),
                "Cell ({},{}) should be shaded",
                y,
                x
            );
        }
    }

    #[test]
    fn test_kageboshi_medium() {
        let problem = make_medium_problem();
        let result = solve_kageboshi(&problem);
        assert!(result.is_some(), "Medium puzzle should be solvable");
        let grid = result.unwrap();
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
        // Verify expected shaded cells:
        let expected_shaded = [
            (0, 1),
            (0, 3),
            (0, 5),
            (1, 7),
            (2, 0),
            (2, 2),
            (2, 5),
            (3, 1),
            (3, 4),
            (3, 7),
            (4, 2),
            (4, 5),
            (5, 0),
            (5, 6),
            (6, 2),
            (6, 4),
            (7, 0),
            (7, 3),
            (7, 6),
        ];
        for &(y, x) in &expected_shaded {
            assert_eq!(
                grid[y][x],
                Some(true),
                "Cell ({},{}) should be shaded",
                y,
                x
            );
        }
    }

    #[test]
    fn test_kageboshi_hard() {
        let problem = make_hard_problem();
        let result = solve_kageboshi(&problem);
        assert!(result.is_some(), "Hard puzzle should be solvable");
        let grid = result.unwrap();
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
        // Verify expected shaded cells:
        let expected_shaded = [
            (0, 1),
            (0, 4),
            (0, 7),
            (1, 9),
            (2, 2),
            (2, 6),
            (3, 0),
            (3, 4),
            (3, 8),
            (5, 1),
            (5, 5),
            (5, 9),
            (6, 3),
            (6, 7),
            (7, 0),
            (8, 2),
            (8, 5),
            (8, 8),
            (9, 4),
            (9, 7),
        ];
        for &(y, x) in &expected_shaded {
            assert_eq!(
                grid[y][x],
                Some(true),
                "Cell ({},{}) should be shaded",
                y,
                x
            );
        }
    }

    #[test]
    fn test_deserialize_easy_url() {
        let url = "http://localhost:8000/p.html?kageboshi/6/6/g5s4g4g5p5i4g";
        let problem = deserialize_problem(url);
        assert!(problem.is_some(), "Should parse easy URL");
        let p = problem.unwrap();
        assert_eq!(p.len(), 6);
        assert_eq!(p[0].len(), 6);
        assert_eq!(p[0][1], Some(5));
        assert_eq!(p[2][3], Some(4));
        assert_eq!(p[2][5], Some(4));
        assert_eq!(p[3][1], Some(5));
        assert_eq!(p[5][0], Some(5));
        assert_eq!(p[5][4], Some(4));
    }

    #[test]
    fn test_deserialize_and_solve_easy() {
        let url = "http://localhost:8000/p.html?kageboshi/6/6/g5s4g4g5p5i4g";
        let problem = deserialize_problem(url).unwrap();
        let result = solve_kageboshi(&problem);
        assert!(result.is_some(), "Easy URL puzzle should be solvable");
        let grid = result.unwrap();
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
        let url = "http://localhost:8000/p.html?kageboshi/8/8/6g6g5g5h3g3g43h5g55g5g6g65g65g54g44g44g4g445g454g4g5g4g5h56g5";
        let problem = deserialize_problem(url).unwrap();
        let result = solve_kageboshi(&problem);
        assert!(result.is_some(), "Medium URL puzzle should be solvable");
        let grid = result.unwrap();
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
        let url = "http://localhost:8000/p.html?kageboshi/10/10/5g54g54g55333243243g44g354g544g554g546g522213213225g546g465g444g543g44g33243243355g46g46g54443g43g44";
        let problem = deserialize_problem(url).unwrap();
        let result = solve_kageboshi(&problem);
        assert!(result.is_some(), "Hard URL puzzle should be solvable");
        let grid = result.unwrap();
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
