// Custom Lightup2 variant (based on akari.rs):
// - checkDiag4Akari: Numbered walls count DIAGONAL neighbours (4 diagonals) instead of orthogonal
// - checkOrthAdjacentAkari: No two bulbs may be orthogonally adjacent
// - Diagonal illumination: Lights illuminate along diagonal lines (not orthogonal)
//   A light at (y,x) illuminates (y-k,x-k), (y-k,x+k), (y+k,x-k), (y+k,x+k) for k=1,2,...
//   until hitting a wall
// - Every non-wall cell must be illuminated (by itself if it has a light, or by diagonal visibility)

use crate::util;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, NumSpaces, Spaces,
};
use cspuz_rs::solver::{BoolVar, Solver};

pub fn solve_lightup2(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let has_light = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(has_light);

    // Wall cells can't have lights
    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                solver.add_expr(!has_light.at((y, x)));
                // CUSTOM: numbered walls count DIAGONAL neighbors
                if n >= 0 {
                    let mut diag_neighbors: Vec<BoolVar> = vec![];
                    if y > 0 && x > 0 && clues[y - 1][x - 1].is_none() {
                        diag_neighbors.push(has_light.at((y - 1, x - 1)).clone());
                    }
                    if y > 0 && x + 1 < w && clues[y - 1][x + 1].is_none() {
                        diag_neighbors.push(has_light.at((y - 1, x + 1)).clone());
                    }
                    if y + 1 < h && x > 0 && clues[y + 1][x - 1].is_none() {
                        diag_neighbors.push(has_light.at((y + 1, x - 1)).clone());
                    }
                    if y + 1 < h && x + 1 < w && clues[y + 1][x + 1].is_none() {
                        diag_neighbors.push(has_light.at((y + 1, x + 1)).clone());
                    }
                    // Sum of diagonal neighbors == n
                    solver.add_expr(cspuz_rs::solver::count_true(&diag_neighbors).eq(n));
                }
            }
        }
    }

    // CUSTOM: No two lights orthogonally adjacent
    for y in 0..h {
        for x in 0..w {
            if clues[y][x].is_some() {
                continue;
            }
            if y + 1 < h && clues[y + 1][x].is_none() {
                solver.add_expr(!(has_light.at((y, x)) & has_light.at((y + 1, x))));
            }
            if x + 1 < w && clues[y][x + 1].is_none() {
                solver.add_expr(!(has_light.at((y, x)) & has_light.at((y, x + 1))));
            }
        }
    }

    // CUSTOM: Diagonal illumination
    // For each diagonal line segment (group of non-wall cells on the same diagonal),
    // at most 1 light (lights see each other diagonally and would conflict)
    // Also, every non-wall cell must be illuminated by at least one diagonal group

    // There are 4 diagonal directions, but we can group by 2 diagonal families:
    // Family 1: y-x = constant (NW-SE diagonals: direction (1,1) and (-1,-1))
    // Family 2: y+x = constant (NE-SW diagonals: direction (1,-1) and (-1,1))

    // For each diagonal family, split into segments separated by walls.
    // Each segment must have exactly 0 or 1 light (like akari's h/v groups).
    // Each non-wall cell is illuminated if ANY of its diagonal groups has a light.

    let mut diag_groups: Vec<Vec<(usize, usize)>> = vec![];

    // Family 1: y - x = constant (NW-SE), traverse top-left to bottom-right
    for start_sum in -(w as i32 - 1)..=(h as i32 - 1) {
        // y - x = start_sum
        let mut segment: Vec<(usize, usize)> = vec![];
        let y_start = if start_sum >= 0 {
            start_sum as usize
        } else {
            0
        };
        let y_end = std::cmp::min(h, (w as i32 + start_sum) as usize);
        for y in y_start..y_end {
            let x = (y as i32 - start_sum) as usize;
            if x >= w {
                continue;
            }
            if clues[y][x].is_some() {
                if !segment.is_empty() {
                    diag_groups.push(segment.clone());
                    segment.clear();
                }
            } else {
                segment.push((y, x));
            }
        }
        if !segment.is_empty() {
            diag_groups.push(segment);
        }
    }

    // Family 2: y + x = constant (NE-SW), traverse top-right to bottom-left
    for diag_sum in 0..(h + w - 1) {
        // y + x = diag_sum
        let mut segment: Vec<(usize, usize)> = vec![];
        let y_start = if diag_sum >= w { diag_sum - w + 1 } else { 0 };
        let y_end = std::cmp::min(h, diag_sum + 1);
        for y in y_start..y_end {
            let x = diag_sum - y;
            if x >= w {
                continue;
            }
            if clues[y][x].is_some() {
                if !segment.is_empty() {
                    diag_groups.push(segment.clone());
                    segment.clear();
                }
            } else {
                segment.push((y, x));
            }
        }
        if !segment.is_empty() {
            diag_groups.push(segment);
        }
    }

    // For each group: at most 1 light, create a BoolVar indicating "this group has a light"
    let mut cell_groups: Vec<Vec<BoolVar>> = vec![vec![]; h * w];
    for group in &diag_groups {
        let v = solver.bool_var();
        let group_lights: Vec<BoolVar> = group
            .iter()
            .map(|&(y, x)| has_light.at((y, x)).clone())
            .collect();
        solver.add_expr(cspuz_rs::solver::count_true(&group_lights).eq(v.clone().ite(1, 0)));
        for &(y, x) in group {
            cell_groups[y * w + x].push(v.clone());
        }
    }

    // Every non-wall cell must be illuminated (at least one of its groups has a light)
    for y in 0..h {
        for x in 0..w {
            if clues[y][x].is_none() {
                let groups = &cell_groups[y * w + x];
                if groups.is_empty() {
                    // Isolated cell with no diagonal groups — must have a light itself
                    solver.add_expr(has_light.at((y, x)));
                } else {
                    let mut or_expr = groups[0].expr();
                    for g in &groups[1..] {
                        or_expr = or_expr | g.expr();
                    }
                    solver.add_expr(or_expr);
                }
            }
        }
    }

    solver.irrefutable_facts().map(|f| f.get(has_light))
}

type Problem = Vec<Vec<Option<i32>>>;

fn combinator() -> impl Combinator<Problem> {
    Grid::new(Choice::new(vec![
        Box::new(NumSpaces::new(4, 2)),
        Box::new(Spaces::new(None, 'g')),
        Box::new(Dict::new(Some(-1), ".")),
    ]))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    problem_to_url(combinator(), "lightup2", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["akari", "lightup", "lightup2"], url)
}
