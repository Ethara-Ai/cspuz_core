// Parity Pipes:
// - Draw closed loops on grid edges
// - Every vertex is pre-colored: Black (1) or White (2)
// - Black vertex: loop passes through (exactly 2 incident drawn edges)
// - White vertex: loop skips (exactly 0 incident drawn edges)
// - All drawn edges must form valid closed loops (no branches, no dead-ends)

use cspuz_rs::graph;
use cspuz_rs::serializer::strip_prefix;
use cspuz_rs::solver::Solver;

/// Vertex colors: true = black (loop passes through), false = white (loop skips)
pub type Problem = Vec<Vec<bool>>;

pub fn solve_paritypipes(
    vertex_colors: &[Vec<bool>],
) -> Option<graph::BoolGridEdgesIrrefutableFacts> {
    let height = vertex_colors.len() - 1;
    let width = vertex_colors[0].len() - 1;

    let mut solver = Solver::new();
    let is_line = &graph::BoolGridEdges::new(&mut solver, (height, width));
    solver.add_answer_key_bool(&is_line.horizontal);
    solver.add_answer_key_bool(&is_line.vertical);

    // Parity Pipes allows multiple disjoint loops.
    // Vertex degree constraint: black vertex = exactly 2 edges, white = exactly 0.
    // This inherently prevents branches (>2) and dead-ends (1).
    for y in 0..=height {
        for x in 0..=width {
            let neighbors = is_line.vertex_neighbors((y, x));
            let expected = if vertex_colors[y][x] { 2 } else { 0 };
            solver.add_expr(neighbors.count_true().eq(expected));
        }
    }

    solver.irrefutable_facts().map(|f| f.get(is_line))
}

/// Decode base-32 bit-packed vertex colors from URL body.
/// Format: 5 bits per base-32 character, 1=black, 0=white.
/// URL: paritypipes/{cols}/{rows}/{body}
fn decode_parity_body(body: &str, height: usize, width: usize) -> Option<Vec<Vec<bool>>> {
    let num_vertices = (height + 1) * (width + 1);
    let mut bits = Vec::with_capacity(num_vertices);

    for ch in body.chars() {
        let val = ch.to_digit(32)? as u8;
        for bit in (0..5).rev() {
            if bits.len() >= num_vertices {
                break;
            }
            bits.push((val >> bit) & 1 == 1);
        }
    }

    if bits.len() < num_vertices {
        return None;
    }

    let mut grid = Vec::with_capacity(height + 1);
    let mut idx = 0;
    for _ in 0..=height {
        let mut row = Vec::with_capacity(width + 1);
        for _ in 0..=width {
            row.push(bits[idx]);
            idx += 1;
        }
        grid.push(row);
    }
    Some(grid)
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    let body = strip_prefix(url)?;
    let mut parts = body.splitn(4, '/');
    let kind = parts.next()?;
    if kind != "paritypipes" {
        return None;
    }
    let cols: usize = parts.next()?.parse().ok()?;
    let rows: usize = parts.next()?.parse().ok()?;
    let encoded = parts.next()?;
    decode_parity_body(encoded, rows, cols)
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    let height = problem.len() - 1;
    let width = problem[0].len() - 1;

    let mut bits = Vec::new();
    for row in problem {
        for &v in row {
            bits.push(if v { 1u8 } else { 0u8 });
        }
    }

    let mut encoded = String::new();
    let mut i = 0;
    while i < bits.len() {
        let mut val = 0u8;
        for bit in (0..5).rev() {
            if i < bits.len() {
                val |= bits[i] << bit;
                i += 1;
            }
        }
        encoded.push(char::from_digit(val as u32, 32)?);
    }

    Some(format!(
        "https://pzprxs.vercel.app/p?paritypipes/{}/{}/{}",
        width, height, encoded
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem_for_tests() -> Problem {
        let mut grid = vec![vec![false; 5]; 5];
        for y in 1..=3 {
            for x in 1..=3 {
                if y == 1 || y == 3 || x == 1 || x == 3 {
                    grid[y][x] = true;
                }
            }
        }
        grid
    }

    #[test]
    fn test_paritypipes_solve() {
        let problem = problem_for_tests();
        let ans = solve_paritypipes(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        for row in &ans.horizontal {
            for cell in row {
                assert!(cell.is_some());
            }
        }
        for row in &ans.vertical {
            for cell in row {
                assert!(cell.is_some());
            }
        }
    }

    #[test]
    fn test_paritypipes_serializer() {
        let problem = problem_for_tests();
        let url = serialize_problem(&problem).unwrap();
        let deserialized = deserialize_problem(&url).unwrap();
        assert_eq!(problem, deserialized);
    }
}
