use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url_with_context, url_to_problem, Choice, Combinator, Context, Dict, HexInt,
    Optionalize, RoomsWithValues, Size, Spaces,
};

/// Nori Bridge Custom: delegates to the standard noribridge solver and wraps
/// the result with an all-None cell shading layer so the backend can render
/// both shading (currently unused) and bridges uniformly.
///
/// Return type: `(shading, bridges, edges)` where
///   - `shading[y][x]` = `None` for every cell (no shading in this variant)
///   - `bridges[edge_idx]` = `Some(true/false)` per region-adjacency edge
///   - `edges` = list of `(room_a, room_b)` pairs

pub fn solve_noribridge_custom(
    borders: &graph::InnerGridEdges<Vec<Vec<bool>>>,
    clues: &[Option<i32>],
) -> Option<(
    Vec<Vec<Option<bool>>>,
    Vec<Option<bool>>,
    Vec<(usize, usize)>,
)> {
    let (h, w) = borders.base_shape();

    let result = super::noribridge::solve_noribridge(borders, clues)?;
    let (bridges, edges) = result;

    let shading = vec![vec![None; w]; h];

    Some((shading, bridges, edges))
}

pub(super) type Problem = (graph::InnerGridEdges<Vec<Vec<bool>>>, Vec<Option<i32>>);

pub(super) fn combinator() -> impl Combinator<Problem> {
    Size::new(RoomsWithValues::new(Choice::new(vec![
        Box::new(Optionalize::new(HexInt)),
        Box::new(Spaces::new(None, 'g')),
        Box::new(Dict::new(Some(-1), ".")),
    ])))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    let height = problem.0.vertical.len();
    let width = problem.0.vertical[0].len() + 1;
    problem_to_url_with_context(
        combinator(),
        "noribridge",
        problem.clone(),
        &Context::sized(height, width),
    )
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["noribridge"], url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noribridge_custom_solver() {
        let url = "https://puzz.link/p?noribridge/4/2/a0h1g1";
        let problem = deserialize_problem(url);
        assert!(problem.is_some());
        let (borders, clues) = problem.unwrap();
        let result = solve_noribridge_custom(&borders, &clues);
        assert!(result.is_some());
        let (shading, bridges, edges) = result.unwrap();
        assert!(shading.iter().all(|row| row.iter().all(|c| c.is_none())));
        assert!(!edges.is_empty());
        assert_eq!(bridges.len(), edges.len());
    }

    #[test]
    fn test_noribridge_custom_easy_url() {
        let url = "https://puzz.link/p?noribridge/6/6/aaaaaa0fo000232111";
        let problem = deserialize_problem(url);
        assert!(problem.is_some());
        let (borders, clues) = problem.unwrap();
        let result = solve_noribridge_custom(&borders, &clues);
        assert!(result.is_some());
    }
}
