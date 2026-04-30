use crate::util;
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Grid, HexInt, Optionalize, Spaces,
};
use cspuz_rs::solver::Solver;

pub fn solve_tidepool(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    graph::active_vertices_connected_2d(&mut solver, !is_black);
    solver.add_expr(!is_black.conv2d_and((2, 2)));

    for y in 0..h {
        for x in 0..w {
            if clues[y][x].is_some() {
                solver.add_expr(!is_black.at((y, x)));
            }
        }
    }

    // BFS distance as SAT: depth[y][x] = 0 for border unshaded,
    // else 1 + min(unshaded neighbor depths).
    // Encoded via: (1) adjacent unshaded cells differ by ≤1,
    // (2) each interior unshaded cell has a neighbor with strictly lower depth.
    let max_depth = (h * w / 2) as i32;
    let depth = &solver.int_var_2d((h, w), 0, max_depth);

    for y in 0..h {
        for x in 0..w {
            // Pin black cells' depth to 0 to eliminate spurious solutions
            solver.add_expr(is_black.at((y, x)).imp(depth.at((y, x)).eq(0)));

            if y == 0 || y == h - 1 || x == 0 || x == w - 1 {
                solver.add_expr((!is_black.at((y, x))).imp(depth.at((y, x)).eq(0)));
            } else {
                solver.add_expr((!is_black.at((y, x))).imp(depth.at((y, x)).ge(1)));
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            if x + 1 < w {
                let both_white = !is_black.at((y, x)) & !is_black.at((y, x + 1));
                solver.add_expr(
                    both_white.imp(depth.at((y, x)).le(depth.at((y, x + 1)) + 1)),
                );
                solver.add_expr(
                    both_white.imp(depth.at((y, x + 1)).le(depth.at((y, x)) + 1)),
                );
            }
            if y + 1 < h {
                let both_white = !is_black.at((y, x)) & !is_black.at((y + 1, x));
                solver.add_expr(
                    both_white.imp(depth.at((y, x)).le(depth.at((y + 1, x)) + 1)),
                );
                solver.add_expr(
                    both_white.imp(depth.at((y + 1, x)).le(depth.at((y, x)) + 1)),
                );
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            if y == 0 || y == h - 1 || x == 0 || x == w - 1 {
                continue;
            }
            let dirs: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            let mut has_lower = vec![];
            for &(dy, dx) in &dirs {
                let ny = y as i32 + dy;
                let nx = x as i32 + dx;
                if ny >= 0 && ny < h as i32 && nx >= 0 && nx < w as i32 {
                    let ny = ny as usize;
                    let nx = nx as usize;
                    has_lower.push(
                        !is_black.at((ny, nx)) & depth.at((ny, nx)).lt(depth.at((y, x))),
                    );
                }
            }
            let any_lower = has_lower
                .into_iter()
                .reduce(|a, b| a | b)
                .unwrap();
            solver.add_expr((!is_black.at((y, x))).imp(any_lower));
        }
    }

    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                if n >= 0 {
                    solver.add_expr(depth.at((y, x)).eq(n));
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
    ]))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    problem_to_url(combinator(), "tidepool", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["tidepool"], url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem_for_tests() -> Problem {
        let mut grid = vec![vec![None; 6]; 6];
        grid[0][0] = Some(0);
        grid[0][1] = Some(0);
        grid[0][2] = Some(0);
        grid[0][3] = Some(0);
        grid[0][4] = Some(0);
        grid[0][5] = Some(0);
        grid[1][0] = Some(0);
        grid[1][1] = Some(1);
        grid[1][4] = Some(1);
        grid[1][5] = Some(0);
        grid[2][0] = Some(0);
        grid[2][2] = Some(3);
        grid[2][3] = Some(3);
        grid[2][5] = Some(0);
        grid[3][0] = Some(0);
        grid[3][1] = Some(1);
        grid[3][2] = Some(2);
        grid[3][3] = Some(2);
        grid[3][4] = Some(1);
        grid[3][5] = Some(0);
        grid[4][0] = Some(0);
        grid[4][1] = Some(1);
        grid[4][2] = Some(1);
        grid[4][3] = Some(1);
        grid[4][4] = Some(1);
        grid[4][5] = Some(0);
        grid[5][0] = Some(0);
        grid[5][1] = Some(0);
        grid[5][2] = Some(0);
        grid[5][3] = Some(0);
        grid[5][4] = Some(0);
        grid[5][5] = Some(0);
        grid
    }

    #[test]
    fn test_tidepool_problem() {
        let problem = problem_for_tests();
        let ans = solve_tidepool(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        for row in &ans {
            for cell in row {
                assert!(cell.is_some());
            }
        }
    }

    #[test]
    fn test_tidepool_serializer() {
        let problem = problem_for_tests();
        let url = "https://puzz.link/p?tidepool/6/6/00000001h100g33g0g1h1g01111000h00";
        util::tests::serializer_test(problem, url, serialize_problem, deserialize_problem);
    }
}
