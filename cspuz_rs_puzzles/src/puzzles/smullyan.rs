use crate::util;
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url_pzprxs, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Optionalize,
    Spaces,
};
use cspuz_rs::solver::Solver;

pub fn solve_smullyan(clues: &Vec<Vec<Option<i32>>>) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    graph::active_vertices_connected_2d(&mut solver, !is_black);
    solver.add_expr(!is_black.conv2d_and((1, 2)));
    solver.add_expr(!is_black.conv2d_and((2, 1)));

    for y in 0..h {
        for x in 0..w {
            if let Some(c) = clues[y][x] {
                if c >= 0 {
                    solver.add_expr(
                        !is_black.at((y, x)).iff(
                            (is_black.eight_neighbors((y, x)).count_true()
                                + is_black.at((y, x)).count_true())
                            .eq(c),
                        ),
                    )
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
    problem_to_url_pzprxs(combinator(), "smullyan", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["smullyan"], url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem_for_tests() -> Problem {
        vec![
            vec![Some(1), Some(1), Some(2), Some(2), Some(1)],
            vec![Some(2), Some(0), Some(3), Some(2), Some(2)],
            vec![Some(2), Some(3), Some(3), Some(2), Some(1)],
            vec![Some(2), Some(3), Some(2), Some(2), Some(2)],
            vec![Some(1), Some(1), Some(1), Some(1), Some(2)],
        ]
    }

    #[test]
    fn test_smullyan_problem() {
        let problem = problem_for_tests();
        let ans = solve_smullyan(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        let expected = crate::util::tests::to_option_bool_2d([
            [0, 0, 0, 1, 0],
            [0, 1, 0, 0, 0],
            [1, 0, 0, 1, 0],
            [0, 1, 0, 0, 0],
            [0, 0, 0, 0, 1],
        ]);
        assert_eq!(ans, expected);
    }

    #[test]
    fn test_smullyan_serializer() {
        let problem = problem_for_tests();
        let url = "https://pzprxs.vercel.app/p?smullyan/5/5/1122120322233212322211112";
        util::tests::serializer_test(problem, url, serialize_problem, deserialize_problem);
    }
}
