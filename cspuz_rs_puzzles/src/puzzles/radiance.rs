use cspuz_rs::serializer::strip_prefix;
use cspuz_rs::solver::Solver;

const DIR_UP: i32 = 1;
const DIR_DN: i32 = 2;
const DIR_LT: i32 = 3;
const DIR_RT: i32 = 4;

pub const MIRROR_SLASH: i32 = 0;
pub const MIRROR_BACKSLASH: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellKind {
    Empty,
    Emitter(i32),
    Target,
    MirrorSlot,
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub height: usize,
    pub width: usize,
    pub grid: Vec<Vec<CellKind>>,
}

impl Problem {
    pub fn find_emitter(&self) -> Option<(usize, usize, i32)> {
        for y in 0..self.height {
            for x in 0..self.width {
                if let CellKind::Emitter(dir) = self.grid[y][x] {
                    return Some((y, x, dir));
                }
            }
        }
        None
    }

    pub fn find_target(&self) -> Option<(usize, usize)> {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.grid[y][x] == CellKind::Target {
                    return Some((y, x));
                }
            }
        }
        None
    }

    pub fn mirror_slots(&self) -> Vec<(usize, usize)> {
        let mut slots = vec![];
        for y in 0..self.height {
            for x in 0..self.width {
                if self.grid[y][x] == CellKind::MirrorSlot {
                    slots.push((y, x));
                }
            }
        }
        slots
    }
}

fn reflect_direction(dir: i32, mirror: i32) -> i32 {
    match mirror {
        MIRROR_SLASH => match dir {
            DIR_UP => DIR_RT,
            DIR_RT => DIR_UP,
            DIR_DN => DIR_LT,
            DIR_LT => DIR_DN,
            _ => dir,
        },
        MIRROR_BACKSLASH => match dir {
            DIR_UP => DIR_LT,
            DIR_LT => DIR_UP,
            DIR_DN => DIR_RT,
            DIR_RT => DIR_DN,
            _ => dir,
        },
        _ => dir,
    }
}

fn step(y: usize, x: usize, dir: i32, height: usize, width: usize) -> Option<(usize, usize)> {
    match dir {
        DIR_UP => {
            if y == 0 {
                None
            } else {
                Some((y - 1, x))
            }
        }
        DIR_DN => {
            if y + 1 >= height {
                None
            } else {
                Some((y + 1, x))
            }
        }
        DIR_LT => {
            if x == 0 {
                None
            } else {
                Some((y, x - 1))
            }
        }
        DIR_RT => {
            if x + 1 >= width {
                None
            } else {
                Some((y, x + 1))
            }
        }
        _ => None,
    }
}

fn trace_beam(
    problem: &Problem,
    mirror_assignment: &[(usize, usize, i32)],
) -> (Vec<(usize, usize)>, bool) {
    let (ey, ex, mut dir) = problem.find_emitter().unwrap();
    let (ty, tx) = problem.find_target().unwrap();

    let mut visited = vec![];
    let mut cy = ey;
    let mut cx = ex;
    let max_steps = problem.height * problem.width * 4;

    for _ in 0..max_steps {
        match step(cy, cx, dir, problem.height, problem.width) {
            None => return (visited, false),
            Some((ny, nx)) => {
                cy = ny;
                cx = nx;
                visited.push((cy, cx));

                if cy == ty && cx == tx {
                    return (visited, true);
                }

                if cy == ey && cx == ex {
                    return (visited, false);
                }

                if let Some(assignment) = mirror_assignment
                    .iter()
                    .find(|(my, mx, _)| *my == cy && *mx == cx)
                {
                    dir = reflect_direction(dir, assignment.2);
                }
            }
        }
    }

    (visited, false)
}

pub fn solve_radiance(problem: &Problem) -> Option<Vec<Vec<Option<i32>>>> {
    let slots = problem.mirror_slots();
    let n = slots.len();
    let (ty, tx) = problem.find_target()?;

    if n == 0 {
        let (_, reaches) = trace_beam(problem, &[]);
        if reaches {
            let ans = vec![vec![None; problem.width]; problem.height];
            return Some(ans);
        } else {
            return None;
        }
    }

    let mut valid_assignments: Vec<Vec<i32>> = vec![];

    for mask in 0..(1u64 << n) {
        let assignment: Vec<(usize, usize, i32)> = slots
            .iter()
            .enumerate()
            .map(|(i, &(y, x))| {
                let mirror_type = if (mask >> i) & 1 == 0 {
                    MIRROR_SLASH
                } else {
                    MIRROR_BACKSLASH
                };
                (y, x, mirror_type)
            })
            .collect();

        let (visited, reaches_target) = trace_beam(problem, &assignment);

        if !reaches_target {
            continue;
        }

        let all_slots_used = slots
            .iter()
            .all(|&(sy, sx)| visited.iter().any(|&(vy, vx)| vy == sy && vx == sx));

        if !all_slots_used {
            continue;
        }

        if let Some(&last) = visited.last() {
            if last != (ty, tx) {
                continue;
            }
        }

        let mirror_values: Vec<i32> = (0..n)
            .map(|i| {
                if (mask >> i) & 1 == 0 {
                    MIRROR_SLASH
                } else {
                    MIRROR_BACKSLASH
                }
            })
            .collect();
        valid_assignments.push(mirror_values);
    }

    if valid_assignments.is_empty() {
        return None;
    }

    if valid_assignments.len() == 1 {
        let assignment = &valid_assignments[0];
        let mut ans = vec![vec![None; problem.width]; problem.height];
        for (i, &(sy, sx)) in slots.iter().enumerate() {
            ans[sy][sx] = Some(assignment[i]);
        }
        return Some(ans);
    }

    let mut solver = Solver::new();
    let mirror_vars = &solver.int_var_1d(n, 0, 1);
    solver.add_answer_key_int(mirror_vars);

    let clauses: Vec<_> = valid_assignments
        .iter()
        .map(|assignment| {
            let conjuncts: Vec<_> = assignment
                .iter()
                .enumerate()
                .map(|(i, &val)| mirror_vars.at(i).eq(val))
                .collect();
            cspuz_rs::solver::all(conjuncts)
        })
        .collect();

    solver.add_expr(cspuz_rs::solver::any(clauses));

    let model = solver.irrefutable_facts()?;
    let mirror_values: Vec<Option<i32>> = model.get(mirror_vars);

    let mut ans = vec![vec![None; problem.width]; problem.height];
    for (i, &(sy, sx)) in slots.iter().enumerate() {
        ans[sy][sx] = mirror_values[i];
    }
    Some(ans)
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    let body = strip_prefix(url)?;
    let mut parts = body.splitn(4, '/');
    let kind = parts.next()?;
    if kind != "radiance" {
        return None;
    }
    let cols: usize = parts.next()?.parse().ok()?;
    let rows: usize = parts.next()?.parse().ok()?;
    let encoded = parts.next().unwrap_or("");

    let height = rows;
    let width = cols;
    let mut grid = vec![vec![CellKind::Empty; width]; height];

    let bytes = encoded.as_bytes();
    let mut pos = 0;
    let mut cell_idx = 0;

    while pos < bytes.len() && cell_idx < height * width {
        let ch = bytes[pos];
        match ch {
            b'1' => {
                pos += 1;
                if pos >= bytes.len() {
                    break;
                }
                let dir = (bytes[pos] - b'0') as i32;
                let y = cell_idx / width;
                let x = cell_idx % width;
                grid[y][x] = CellKind::Emitter(dir);
                cell_idx += 1;
                pos += 1;
            }
            b'2' => {
                let y = cell_idx / width;
                let x = cell_idx % width;
                grid[y][x] = CellKind::Target;
                cell_idx += 1;
                pos += 1;
            }
            b'3' => {
                let y = cell_idx / width;
                let x = cell_idx % width;
                grid[y][x] = CellKind::MirrorSlot;
                cell_idx += 1;
                pos += 1;
            }
            b'a'..=b'z' => {
                let skip = (ch - b'a') as usize + 1;
                cell_idx += skip;
                pos += 1;
            }
            _ => {
                pos += 1;
            }
        }
    }

    Some(Problem {
        height,
        width,
        grid,
    })
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    let mut body = String::new();
    let mut empty_count = 0usize;

    for y in 0..problem.height {
        for x in 0..problem.width {
            match &problem.grid[y][x] {
                CellKind::Empty => {
                    empty_count += 1;
                }
                cell => {
                    while empty_count > 0 {
                        let chunk = empty_count.min(26);
                        body.push((b'a' + chunk as u8 - 1) as char);
                        empty_count -= chunk;
                    }
                    match cell {
                        CellKind::Emitter(dir) => {
                            body.push('1');
                            body.push(char::from_digit(*dir as u32, 10)?);
                        }
                        CellKind::Target => {
                            body.push('2');
                        }
                        CellKind::MirrorSlot => {
                            body.push('3');
                        }
                        CellKind::Empty => unreachable!(),
                    }
                }
            }
        }
    }
    while empty_count > 0 {
        let chunk = empty_count.min(26);
        body.push((b'a' + chunk as u8 - 1) as char);
        empty_count -= chunk;
    }

    Some(format!(
        "https://pzprxs.vercel.app/p?radiance/{}/{}/{}",
        problem.width, problem.height, body
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem_for_tests() -> Problem {
        let mut grid = vec![vec![CellKind::Empty; 6]; 6];
        grid[0][0] = CellKind::Emitter(DIR_RT);
        grid[0][5] = CellKind::MirrorSlot;
        grid[3][5] = CellKind::MirrorSlot;
        grid[3][0] = CellKind::Target;
        Problem {
            height: 6,
            width: 6,
            grid,
        }
    }

    #[test]
    fn test_radiance_solve() {
        let problem = problem_for_tests();
        let ans = solve_radiance(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        assert_eq!(ans[0][5], Some(MIRROR_BACKSLASH));
        assert_eq!(ans[3][5], Some(MIRROR_SLASH));
    }

    #[test]
    fn test_radiance_serializer() {
        let problem = problem_for_tests();
        let url = serialize_problem(&problem).unwrap();
        let deserialized = deserialize_problem(&url).unwrap();
        assert_eq!(deserialized.height, problem.height);
        assert_eq!(deserialized.width, problem.width);
        assert_eq!(deserialized.grid, problem.grid);
    }

    #[test]
    fn test_radiance_from_url() {
        let url = "https://pzprxs.vercel.app/p?radiance/6/6/14d3l2d3l";
        let problem = deserialize_problem(url).unwrap();
        assert_eq!(problem.height, 6);
        assert_eq!(problem.width, 6);
        assert_eq!(problem.grid[0][0], CellKind::Emitter(DIR_RT));
        assert_eq!(problem.grid[0][5], CellKind::MirrorSlot);
        assert_eq!(problem.grid[3][5], CellKind::MirrorSlot);
        assert_eq!(problem.grid[3][0], CellKind::Target);

        let ans = solve_radiance(&problem).unwrap();
        assert_eq!(ans[0][5], Some(MIRROR_BACKSLASH));
        assert_eq!(ans[3][5], Some(MIRROR_SLASH));
    }

    #[test]
    fn test_radiance_medium() {
        let url = "https://pzprxs.vercel.app/p?radiance/8/8/14f3p3f3x2g";
        let problem = deserialize_problem(url).unwrap();
        let ans = solve_radiance(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        for &(sy, sx) in &problem.mirror_slots() {
            assert!(ans[sy][sx].is_some());
        }
    }

    #[test]
    fn test_radiance_hard() {
        let url = "https://pzprxs.vercel.app/p?radiance/10/10/14h3zd3h3t3h3s2";
        let problem = deserialize_problem(url).unwrap();
        let ans = solve_radiance(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        for &(sy, sx) in &problem.mirror_slots() {
            assert!(ans[sy][sx].is_some());
        }
    }
}
