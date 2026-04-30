/// Custom Lightup with 3x3 box illumination instead of orthogonal rays
use crate::util;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, NumSpaces, Spaces,
};
use cspuz_rs::solver::{count_true, Solver};

pub fn solve_lightup_custom(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let has_light = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(has_light);

    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                // Wall cells cannot have lights
                solver.add_expr(!has_light.at((y, x)));
                if n >= 0 {
                    // Numbered walls: count of orthogonal neighbor lights = n
                    solver.add_expr(has_light.four_neighbors((y, x)).count_true().eq(n));
                }
            }
        }
    }

    // CUSTOM: 3x3 box illumination — each non-wall cell is illuminated if
    // any cell in its 3x3 neighborhood (including itself) has a light
    for y in 0..h {
        for x in 0..w {
            if clues[y][x].is_some() {
                continue;
            }
            let mut neighborhood_lights = vec![has_light.at((y, x)).expr()];
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dy == 0 && dx == 0 {
                        continue;
                    }
                    let ny = y as i32 + dy;
                    let nx = x as i32 + dx;
                    if ny >= 0 && ny < h as i32 && nx >= 0 && nx < w as i32 {
                        let ny = ny as usize;
                        let nx = nx as usize;
                        if clues[ny][nx].is_none() {
                            neighborhood_lights.push(has_light.at((ny, nx)).expr());
                        }
                    }
                }
            }
            // At least one light in 3x3 box
            solver.add_expr(count_true(&neighborhood_lights).ge(1));
        }
    }

    // No two lights can see each other within 3x3 boxes:
    // For each non-wall cell, at most 1 light in the 3x3 neighborhood
    // Actually the constraint is just: each non-wall cell sees at least one light
    // AND no two adjacent lights (the standard constraint handles mutual visibility)
    // In 3x3-box mode, two lights don't block each other — so we need
    // no two non-wall cells that are orthogonally adjacent both have lights
    for y in 0..h {
        for x in 0..w {
            if clues[y][x].is_some() {
                continue;
            }
            // H-segment: no two horizontally adjacent non-wall cells both have lights
            if x + 1 < w && clues[y][x + 1].is_none() {
                solver.add_expr(!(has_light.at((y, x)) & has_light.at((y, x + 1))));
            }
            // V-segment: no two vertically adjacent non-wall cells both have lights
            if y + 1 < h && clues[y + 1][x].is_none() {
                solver.add_expr(!(has_light.at((y, x)) & has_light.at((y + 1, x))));
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
    problem_to_url(combinator(), "lightup", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["akari", "lightup", "lightup_custom"], url)
}
