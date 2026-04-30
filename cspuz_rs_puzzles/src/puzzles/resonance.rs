// Resonance puzzle solver
// Rules:
//   R1: Signal strength at Manhattan distance d from emitter of value V = max(V - d, 0)
//   R2: Multiple signals sum additively
//   R3: Each clue cell receives total signal exactly equal to its number
//   R4: At most one emitter per region (irregular regions)
//   R5: No two emitters orthogonally adjacent
//   Emitter values: 1, 2, or 3 (0 = no emitter)

use cspuz_rs::graph::{self, InnerGridEdges};
use cspuz_rs::serializer::{Combinator, Context, Rooms};
use cspuz_rs::solver::Solver;

pub struct ResonanceProblem {
    pub height: usize,
    pub width: usize,
    pub clues: Vec<Vec<Option<i32>>>,
    pub regions: Vec<Vec<i32>>,
}

pub fn solve_resonance(problem: &ResonanceProblem) -> Option<Vec<Vec<Option<i32>>>> {
    let h = problem.height;
    let w = problem.width;
    let clues = &problem.clues;
    let regions = &problem.regions;

    let mut solver = Solver::new();
    let emitter_val = &solver.int_var_2d((h, w), 0, 3);
    solver.add_answer_key_int(emitter_val);

    for y in 0..h {
        for x in 0..w {
            if clues[y][x].is_some() {
                solver.add_expr(emitter_val.at((y, x)).eq(0));
            }
        }
    }

    // R5: no two emitters orthogonally adjacent
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w {
                solver.add_expr(!(emitter_val.at((y, x)).ge(1) & emitter_val.at((y, x + 1)).ge(1)));
            }
            if y + 1 < h {
                solver.add_expr(!(emitter_val.at((y, x)).ge(1) & emitter_val.at((y + 1, x)).ge(1)));
            }
        }
    }

    // R4: at most one emitter per region
    let num_regions = regions
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .max()
        .unwrap_or(0)
        + 1;
    let mut region_cells: Vec<Vec<(usize, usize)>> = vec![vec![]; num_regions as usize];
    for y in 0..h {
        for x in 0..w {
            region_cells[regions[y][x] as usize].push((y, x));
        }
    }

    for cells in &region_cells {
        if cells.len() <= 1 {
            continue;
        }
        let count_exprs: Vec<_> = cells
            .iter()
            .map(|&(y, x)| emitter_val.at((y, x)).ge(1).ite(1, 0))
            .collect();
        solver.add_expr(cspuz_rs::solver::sum(&count_exprs).le(1));
    }

    // R3: for each clue, sum of max(emitter_val[ey][ex] - d, 0) over all cells == clue_val
    // Max emitter value is 3, so only cells within Manhattan distance <= 2 can contribute:
    //   d=0: self (but clue cells can't have emitters, so always 0)
    //   d=1: contribution = (V >= 2).ite(V - 1, 0) + (V == 1).ite(0, 0) = (V >= 2).ite(V - 1, 0)
    //     Actually V=1 at d=1 gives max(1-1,0)=0, V=2 gives 1, V=3 gives 2
    //   d=2: contribution = (V >= 3).ite(V - 2, 0)
    //     V=1 gives 0, V=2 gives 0, V=3 gives 1
    //   d=3: max(V-3,0) = 0 for all V <= 3, so no contribution
    for y in 0..h {
        for x in 0..w {
            let clue_val = match clues[y][x] {
                Some(v) => v,
                None => continue,
            };

            let mut contributions: Vec<_> = Vec::new();

            for ey in 0..h {
                for ex in 0..w {
                    let d = ((ey as i32 - y as i32).abs() + (ex as i32 - x as i32).abs()) as i32;
                    if d == 0 || d > 2 {
                        continue;
                    }

                    let ev = emitter_val.at((ey, ex));
                    let contrib = if d == 1 {
                        ev.ge(2).ite(ev.expr() - 1, 0)
                    } else {
                        // d == 2
                        ev.ge(3).ite(ev.expr() - 2, 0)
                    };
                    contributions.push(contrib);
                }
            }

            if contributions.is_empty() {
                if clue_val != 0 {
                    return None;
                }
            } else {
                solver.add_expr(cspuz_rs::solver::sum(&contributions).eq(clue_val));
            }
        }
    }

    solver.irrefutable_facts().map(|f| f.get(emitter_val))
}

pub fn deserialize_problem(url: &str) -> Option<ResonanceProblem> {
    let url_part = if let Some(idx) = url.find("resonance/") {
        &url[idx + "resonance/".len()..]
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

    parse_resonance_body(body, rows, cols)
}

/// URL body format: clue_grid_data + border_data
/// - Clue grid: digits = clue values (0-9), letters a-f = values 10-15,
///   letters >= 'g' = skip (ch - 'f') empty cells
/// - Border data: pzprjs Rooms encoding (MultiDigit(2,5) = base-32, 5 border bits per char)
///   First vertical borders (h × (w-1)), then horizontal borders ((h-1) × w)
fn parse_resonance_body(body: &str, rows: usize, cols: usize) -> Option<ResonanceProblem> {
    let total_cells = rows * cols;
    let bytes = body.as_bytes();
    let mut pos = 0;
    let mut clues = vec![vec![None; cols]; rows];
    let mut cell_idx = 0;

    // Parse clue data
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

    // Parse border data using pzprjs Rooms encoding
    let border_bytes = &bytes[pos..];
    let ctx = Context::sized(rows, cols);
    let borders_result = Rooms.deserialize(&ctx, border_bytes);

    let regions = if let Some((_n_read, borders_vec)) = borders_result {
        let borders = &borders_vec[0];
        // Convert borders to room assignments
        let rooms = graph::borders_to_rooms(borders);
        let mut region_grid = vec![vec![0i32; cols]; rows];
        for (room_id, cells) in rooms.iter().enumerate() {
            for &(y, x) in cells {
                region_grid[y][x] = room_id as i32;
            }
        }
        region_grid
    } else {
        // Fallback: uniform 2x2 regions if no border data
        let region_cols = cols / 2;
        let mut regions = vec![vec![0i32; cols]; rows];
        for y in 0..rows {
            for x in 0..cols {
                regions[y][x] = ((y / 2) * region_cols + (x / 2)) as i32;
            }
        }
        regions
    };

    Some(ResonanceProblem {
        height: rows,
        width: cols,
        clues,
        regions,
    })
}

pub fn serialize_problem(problem: &ResonanceProblem) -> Option<String> {
    let h = problem.height;
    let w = problem.width;

    // Encode clue data
    let mut body = String::new();
    let mut gap = 0;

    for y in 0..h {
        for x in 0..w {
            match problem.clues[y][x] {
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

    // Encode border data from regions
    let borders = regions_to_borders(h, w, &problem.regions);
    let ctx = Context::sized(h, w);
    let (_, border_bytes) = Rooms.serialize(&ctx, &[borders])?;
    body.push_str(std::str::from_utf8(&border_bytes).ok()?);

    Some(format!(
        "https://puzz.link/p?resonance/{}/{}/{}",
        w, h, body
    ))
}

/// Convert a region grid to InnerGridEdges
fn regions_to_borders(h: usize, w: usize, regions: &[Vec<i32>]) -> InnerGridEdges<Vec<Vec<bool>>> {
    let mut vertical = vec![vec![false; w - 1]; h];
    let mut horizontal = vec![vec![false; w]; h - 1];

    for y in 0..h {
        for x in 0..w {
            if x + 1 < w && regions[y][x] != regions[y][x + 1] {
                vertical[y][x] = true;
            }
            if y + 1 < h && regions[y][x] != regions[y + 1][x] {
                horizontal[y][x] = true;
            }
        }
    }

    InnerGridEdges {
        vertical,
        horizontal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_easy_problem() -> ResonanceProblem {
        // From puzzle_resonance.py easy puzzle
        let h = 6;
        let w = 6;
        let regions = vec![
            vec![0, 0, 1, 1, 2, 2],
            vec![0, 3, 3, 1, 4, 2],
            vec![5, 5, 3, 6, 4, 4],
            vec![7, 5, 8, 6, 6, 9],
            vec![7, 7, 8, 8, 10, 9],
            vec![11, 11, 11, 10, 10, 9],
        ];
        let mut clues = vec![vec![None; w]; h];
        // Clues from puzzle_resonance.py:
        // (0,1,2) (0,4,0) (0,5,0) (1,1,1) (1,2,2) (1,4,2) (1,5,0)
        // (2,0,1) (2,1,1) (3,0,1) (3,5,1) (4,1,1) (5,1,1) (5,3,0) (5,4,0)
        clues[0][1] = Some(2);
        clues[0][4] = Some(0);
        clues[0][5] = Some(0);
        clues[1][1] = Some(1);
        clues[1][2] = Some(2);
        clues[1][4] = Some(2);
        clues[1][5] = Some(0);
        clues[2][0] = Some(1);
        clues[2][1] = Some(1);
        clues[3][0] = Some(1);
        clues[3][5] = Some(1);
        clues[4][1] = Some(1);
        clues[5][1] = Some(1);
        clues[5][3] = Some(0);
        clues[5][4] = Some(0);

        ResonanceProblem {
            height: h,
            width: w,
            clues,
            regions,
        }
    }

    fn make_medium_problem() -> ResonanceProblem {
        // From puzzle_resonance.py medium puzzle
        let h = 8;
        let w = 8;
        let regions = vec![
            vec![0, 0, 0, 1, 1, 1, 2, 2],
            vec![0, 3, 3, 3, 1, 2, 2, 2],
            vec![4, 4, 3, 3, 5, 5, 5, 6],
            vec![4, 4, 7, 7, 5, 8, 6, 6],
            vec![4, 7, 7, 7, 8, 8, 8, 6],
            vec![9, 9, 7, 10, 10, 8, 11, 6],
            vec![9, 9, 10, 10, 10, 11, 11, 6],
            vec![9, 9, 9, 10, 11, 11, 11, 6],
        ];
        let mut clues = vec![vec![None; w]; h];
        // Clues from puzzle_resonance.py:
        clues[0][6] = Some(0);
        clues[0][7] = Some(0);
        clues[1][1] = Some(2);
        clues[1][2] = Some(3);
        clues[1][6] = Some(2);
        clues[1][7] = Some(0);
        clues[2][0] = Some(0);
        clues[2][3] = Some(2);
        clues[3][0] = Some(2);
        clues[3][4] = Some(2);
        clues[3][7] = Some(0);
        clues[4][7] = Some(0);
        clues[5][3] = Some(2);
        clues[5][7] = Some(1);
        clues[6][2] = Some(2);
        clues[6][4] = Some(2);
        clues[6][6] = Some(1);
        clues[6][7] = Some(0);
        clues[7][0] = Some(0);
        clues[7][5] = Some(2);
        clues[7][6] = Some(0);
        clues[7][7] = Some(0);

        ResonanceProblem {
            height: h,
            width: w,
            clues,
            regions,
        }
    }

    fn make_hard_problem() -> ResonanceProblem {
        // From puzzle_resonance.py hard puzzle
        let h = 10;
        let w = 10;
        let regions = vec![
            vec![0, 0, 0, 1, 1, 2, 2, 2, 3, 3],
            vec![0, 0, 0, 1, 1, 2, 2, 2, 3, 3],
            vec![0, 0, 1, 1, 1, 2, 2, 3, 3, 3],
            vec![4, 4, 4, 1, 1, 5, 5, 5, 5, 3],
            vec![4, 4, 4, 6, 6, 5, 5, 5, 5, 7],
            vec![4, 4, 6, 6, 6, 6, 6, 7, 7, 7],
            vec![8, 8, 8, 8, 8, 6, 7, 7, 7, 7],
            vec![8, 8, 8, 9, 9, 9, 9, 9, 9, 11],
            vec![10, 10, 10, 10, 9, 9, 11, 11, 11, 11],
            vec![10, 10, 10, 10, 12, 12, 12, 12, 11, 11],
        ];
        let mut clues = vec![vec![None; w]; h];
        // Clues from puzzle_resonance.py:
        clues[0][0] = Some(2);
        clues[0][3] = Some(0);
        clues[1][3] = Some(1);
        clues[1][5] = Some(1);
        clues[1][6] = Some(2);
        clues[1][9] = Some(1);
        clues[2][8] = Some(2);
        clues[3][2] = Some(0);
        clues[3][3] = Some(1);
        clues[3][5] = Some(1);
        clues[3][9] = Some(2);
        clues[4][1] = Some(1);
        clues[4][4] = Some(1);
        clues[5][5] = Some(1);
        clues[6][0] = Some(1);
        clues[6][1] = Some(0);
        clues[7][8] = Some(3);
        clues[8][1] = Some(0);
        clues[8][2] = Some(0);
        clues[8][3] = Some(1);
        clues[8][7] = Some(2);
        clues[8][9] = Some(1);
        clues[9][0] = Some(0);
        clues[9][1] = Some(0);
        clues[9][2] = Some(0);
        clues[9][3] = Some(0);
        clues[9][4] = Some(1);
        clues[9][6] = Some(1);
        clues[9][7] = Some(0);

        ResonanceProblem {
            height: h,
            width: w,
            clues,
            regions,
        }
    }

    #[test]
    fn test_resonance_easy() {
        let problem = make_easy_problem();
        let result = solve_resonance(&problem);
        assert!(result.is_some(), "Easy puzzle should be solvable");
        let grid = result.unwrap();
        for y in 0..problem.height {
            for x in 0..problem.width {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
        // Expected solution from puzzle_resonance.py:
        // (0,0,3) (1,3,2) (2,4,2) (3,2,3) (4,5,2) (5,0,2)
        assert_eq!(grid[0][0], Some(3));
        assert_eq!(grid[1][3], Some(2));
        assert_eq!(grid[2][4], Some(2));
        assert_eq!(grid[3][2], Some(3));
        assert_eq!(grid[4][5], Some(2));
        assert_eq!(grid[5][0], Some(2));
        // Non-emitter cells should be 0
        assert_eq!(grid[0][2], Some(0));
        assert_eq!(grid[2][2], Some(0));
    }

    #[test]
    fn test_resonance_medium() {
        let problem = make_medium_problem();
        let result = solve_resonance(&problem);
        assert!(result.is_some(), "Medium puzzle should be solvable");
        let grid = result.unwrap();
        for y in 0..problem.height {
            for x in 0..problem.width {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
        // Expected solution from puzzle_resonance.py:
        // (0,1,3) (1,4,3) (2,6,2) (3,2,3) (4,0,2) (5,5,3) (6,1,2) (7,3,3)
        assert_eq!(grid[0][1], Some(3));
        assert_eq!(grid[1][4], Some(3));
        assert_eq!(grid[2][6], Some(2));
        assert_eq!(grid[3][2], Some(3));
        assert_eq!(grid[4][0], Some(2));
        assert_eq!(grid[5][5], Some(3));
        assert_eq!(grid[6][1], Some(2));
        assert_eq!(grid[7][3], Some(3));
    }

    #[test]
    fn test_resonance_hard() {
        let problem = make_hard_problem();
        let result = solve_resonance(&problem);
        assert!(result.is_some(), "Hard puzzle should be solvable");
        let grid = result.unwrap();
        for y in 0..problem.height {
            for x in 0..problem.width {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined",
                    y,
                    x
                );
            }
        }
        // Expected solution from puzzle_resonance.py:
        // (1,0,3) (2,3,2) (0,6,3) (2,9,2) (4,0,2) (3,7,3) (5,4,2) (6,8,3)
        // (7,0,2) (8,5,3) (8,8,2)
        assert_eq!(grid[1][0], Some(3));
        assert_eq!(grid[2][3], Some(2));
        assert_eq!(grid[0][6], Some(3));
        assert_eq!(grid[2][9], Some(2));
        assert_eq!(grid[4][0], Some(2));
        assert_eq!(grid[3][7], Some(3));
        assert_eq!(grid[5][4], Some(2));
        assert_eq!(grid[6][8], Some(3));
        assert_eq!(grid[7][0], Some(2));
        assert_eq!(grid[8][5], Some(3));
        assert_eq!(grid[8][8], Some(2));
    }

    #[test]
    fn test_deserialize_easy_url() {
        let url =
            "http://localhost:8000/p.html?resonance/6/6/g2h00g12g2011j1j1g1k1g00ganetb5ddddds";
        let problem = deserialize_problem(url);
        assert!(problem.is_some(), "Should parse easy URL");
        let p = problem.unwrap();
        assert_eq!(p.height, 6);
        assert_eq!(p.width, 6);
        // Check clues match puzzle_resonance.py
        assert_eq!(p.clues[0][1], Some(2));
        assert_eq!(p.clues[0][4], Some(0));
        assert_eq!(p.clues[0][5], Some(0));
        assert_eq!(p.clues[1][1], Some(1));
        assert_eq!(p.clues[1][2], Some(2));
        assert_eq!(p.clues[3][5], Some(1));
        assert_eq!(p.clues[5][3], Some(0));
        // Check regions decoded properly (region 0 should contain (0,0), (0,1), (1,0))
        assert_eq!(p.regions[0][0], p.regions[0][1]);
        assert_eq!(p.regions[0][0], p.regions[1][0]);
        // Region boundaries: (0,1) and (0,2) should be different regions
        assert_ne!(p.regions[0][1], p.regions[0][2]);
    }

    #[test]
    fn test_deserialize_and_solve_easy() {
        let url =
            "http://localhost:8000/p.html?resonance/6/6/g2h00g12g2011j1j1g1k1g00ganetb5ddddds";
        let problem = deserialize_problem(url).unwrap();
        let result = solve_resonance(&problem);
        assert!(result.is_some(), "Easy URL puzzle should be solvable");
        let grid = result.unwrap();
        // Check uniqueness: all cells determined
        for y in 0..problem.height {
            for x in 0..problem.width {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined for unique solution",
                    y,
                    x
                );
            }
        }
    }

    #[test]
    fn test_deserialize_and_solve_medium() {
        let url = "http://localhost:8000/p.html?resonance/8/8/l00g23i200h2j2i2h0m0i2i1h2g2g100j2004koklq9dqacgej7jcimq4gk0";
        let problem = deserialize_problem(url).unwrap();
        let result = solve_resonance(&problem);
        assert!(result.is_some(), "Medium URL puzzle should be solvable");
        let grid = result.unwrap();
        for y in 0..problem.height {
            for x in 0..problem.width {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined for unique solution",
                    y,
                    x
                );
            }
        }
    }

    #[test]
    fn test_deserialize_and_solve_hard() {
        let url = "http://localhost:8000/p.html?resonance/10/10/2h0o1g12h1n2i01g1i2g1h1p1j10v3h001i2g100001g10h54a9518ih8830g8k120044su314uv83vue1s";
        let problem = deserialize_problem(url).unwrap();
        let result = solve_resonance(&problem);
        assert!(result.is_some(), "Hard URL puzzle should be solvable");
        let grid = result.unwrap();
        for y in 0..problem.height {
            for x in 0..problem.width {
                assert!(
                    grid[y][x].is_some(),
                    "Cell ({},{}) should be determined for unique solution",
                    y,
                    x
                );
            }
        }
    }
}
