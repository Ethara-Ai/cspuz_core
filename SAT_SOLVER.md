# cspuz_core — SAT-Based Pencil Puzzle Solver

## Overview

cspuz_core solves pencil puzzles (Sudoku, Nurikabe, Slitherlink, etc.) by translating puzzle rules into **Boolean Satisfiability (SAT)** constraints and finding solutions using industrial-strength CDCL solvers. It supports 170+ puzzle types.

The core idea: every pencil puzzle can be expressed as "find an assignment of values to cells such that all rules are satisfied." This is exactly what SAT solvers do — find variable assignments satisfying boolean formulas.

---

## The Pipeline

```
Puzzle URL
  │
  ▼
┌─────────────────────────┐
│  1. URL Parsing          │  Identify puzzle type + decode board state
│     (lib.rs)             │
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  2. Puzzle-Specific      │  Build CSP variables + constraints
│     Solver Module        │  (cspuz_rs_puzzles/src/puzzles/*.rs)
│     via Solver API       │
│     (cspuz_rs/solver/)   │
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  3. CSP Layer            │  High-level constraint representation
│     (csp/repr.rs,        │  BoolExpr, IntExpr, Stmt trees
│      csp/mod.rs)         │
└─────────┬───────────────┘
          │  CSP.optimize() — constant folding + propagation
          ▼
┌─────────────────────────┐
│  4. Normalizer           │  CSP → NormCSP
│     (normalizer.rs)      │  Tseitin transformation, variable merging,
│                          │  expression flattening
└─────────┬───────────────┘
          │  NormCSP.refine_domain() — domain tightening
          ▼
┌─────────────────────────┐
│  5. Encoder              │  NormCSP → SAT clauses
│     (encoder/*.rs)       │  Order / Direct / Log encoding schemes
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  6. SAT Solver           │  CDCL solving
│     (sat.rs,             │  Glucose (default) or CaDiCaL backend
│      backend/*.rs)       │  + native propagators
└─────────┬───────────────┘
          │
          ▼
┌─────────────────────────┐
│  7. Solution Extraction  │  SAT model → CSP assignment → Board
│     + Uniqueness Check   │  (integration.rs, uniqueness.rs)
└─────────────────────────┘
```

Each stage is detailed below.

---

## Stage 1: URL Parsing & Puzzle Dispatch

**File:** `cspuz_solver_backend/src/lib.rs`

Entry point: `decode_and_solve(url: &[u8]) -> Result<Board>`

The solver accepts puzzle URLs in three formats:

| Format | Example | Parser |
|--------|---------|--------|
| puzz.link | `http://puzz.link/p?sudoku/9/9/g3h5...` | `url_to_puzzle_kind()` |
| kudamono | `https://pedros.works/paper-puzzle-player?...` | `get_kudamono_url_info_detailed()` |
| penpa-edit | `https://swaroopg92.github.io/penpa-edit/?...` | Special URL detection |

The URL is parsed to extract:
1. **Puzzle type** (e.g., "sudoku", "nurikabe", "slitherlink")
2. **Grid dimensions** (height × width)
3. **Encoded board state** (clue positions, given numbers, boundary walls)

Dispatch calls the puzzle-specific solver:
```rust
// lib.rs
pub fn decode_and_solve(url: &[u8]) -> Result<Board> {
    // Try puzz_link format first
    if let Some((kind, body)) = url_to_puzzle_kind(url) {
        return puzzle::dispatch_puzz_link(kind, body);
    }
    // Try kudamono format
    if let Some(info) = get_kudamono_url_info_detailed(url) {
        return puzzle::dispatch_kudamono(info);
    }
    // Try penpa_edit format
    // ...
}
```

---

## Stage 2: Puzzle-Specific Constraint Construction

**Files:** `cspuz_rs_puzzles/src/puzzles/*.rs` (170+ modules)

Each puzzle module translates the specific puzzle rules into CSP constraints using the `Solver` API. Two concrete examples:

### Example: Sudoku

```rust
// cspuz_rs_puzzles/src/puzzles/sudoku2.rs
pub fn solve_sudoku2(clues: Vec<Vec<Option<i32>>>) -> Option<Vec<Vec<Option<i32>>>> {
    let n = clues.len();  // 9 for standard sudoku
    let mut solver = Solver::new();

    // Create integer variables: each cell has domain [1, 9]
    let num = &solver.int_var_2d((n, n), 1, n as i32);
    solver.add_answer_key_int(num);

    // Row constraints: all values in each row must be different
    for i in 0..n {
        solver.all_different(num.slice_fixed_y((i, ..)));
    }
    // Column constraints: all values in each column must be different
    for i in 0..n {
        solver.all_different(num.slice_fixed_x((.., i)));
    }
    // Box constraints: all values in each 3×3 box must be different
    for by in 0..block_h {
        for bx in 0..block_w {
            solver.all_different(num.slice((by*h..(by+1)*h, bx*w..(bx+1)*w)));
        }
    }
    // Given clues: fix known cells
    for y in 0..n {
        for x in 0..n {
            if let Some(val) = clues[y][x] {
                solver.add_expr(num.at((y, x)).eq(val));
            }
        }
    }

    // Solve and return irrefutable facts (values that are the same in ALL solutions)
    solver.irrefutable_facts().map(|f| f.get(num))
}
```

### Example: Nurikabe (demonstrates richer constraint types)

```rust
// cspuz_rs_puzzles/src/puzzles/nurikabe.rs
pub fn solve_nurikabe(clues: Vec<Vec<Option<i32>>>) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = (clues.len(), clues[0].len());
    let mut solver = Solver::new();

    // Boolean variable per cell: true = black (sea), false = white (island)
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    // Integer variable per cell: which island group does it belong to?
    // 0 = sea, 1..n = island groups
    let group_id = &solver.int_var_2d((h, w), 0, num_clues as i32);

    // Rule: cell is black ↔ its group_id is 0
    solver.add_expr(is_black.iff(group_id.eq(0)));

    // Rule: all black cells must form one connected region
    graph::active_vertices_connected_2d(&mut solver, is_black);

    // Rule: each island (group) must be connected
    for i in 1..=num_clues {
        graph::active_vertices_connected_2d(&mut solver, group_id.eq(i));
    }

    // Rule: adjacent non-black cells must share the same group_id
    // (prevents different islands from touching)

    // Rule: no 2×2 block of all-black cells
    solver.add_expr(!is_black.conv2d_and((2, 2)));

    // Rule: island size must equal the clue number
    for (i, &(cy, cx, n)) in clue_positions.iter().enumerate() {
        solver.add_expr(group_id.eq(i + 1).count_true().eq(n));
        solver.add_expr(group_id.at((cy, cx)).eq(i + 1));
    }

    solver.irrefutable_facts().map(|f| f.get(is_black))
}
```

### The Solver API

**File:** `cspuz_rs/src/solver/mod.rs` (751 lines)

The `Solver` struct provides a high-level ergonomic API that wraps the `IntegratedSolver`:

| Method | Creates |
|--------|---------|
| `bool_var()` | Single boolean CSP variable |
| `bool_var_2d((h, w))` | 2D grid of boolean variables |
| `int_var(low, high)` | Single integer variable with domain [low, high] |
| `int_var_2d((h, w), low, high)` | 2D grid of integer variables |
| `add_expr(bool_expr)` | Boolean constraint (arbitrary expression tree) |
| `all_different(exprs)` | Global all-different constraint |
| `add_active_vertices_connected(exprs, graph)` | Graph connectivity constraint |
| `add_graph_division(sizes, edges, values)` | Graph partitioning constraint |
| `add_custom_constraint(propagator, vars)` | User-defined propagator |
| `solve()` | Find one solution |
| `irrefutable_facts()` | Find values common to ALL solutions |
| `answer_iter()` | Iterate over all solutions |

Variables support rich expression building via operator overloading:
- `num.at((y, x)).eq(5)` → BoolExpr comparing cell to constant
- `is_black.iff(group_id.eq(0))` → biconditional
- `!is_black.conv2d_and((2, 2))` → 2D convolution AND over 2×2 windows, negated
- `group_id.eq(i).count_true().eq(n)` → count cells matching condition, compare to n

---

## Stage 3: CSP — Constraint Satisfaction Problem Representation

**Files:** `cspuz_core/src/csp/repr.rs` (453 lines), `cspuz_core/src/csp/mod.rs` (1222 lines)

The Solver API builds an internal **CSP** (Constraint Satisfaction Problem), a tree-structured representation of variables and constraints.

### Variables

```rust
BoolVar  // Boolean variable (true/false)
IntVar   // Integer variable with a Domain
```

**Domains** (`domain.rs`, 252 lines):
```rust
enum Domain {
    Range(low, high),           // Contiguous: [low, high]
    Enumerative(Vec<CheckedInt>) // Sparse: {2, 5, 7, 13}
}
```
Domains support refinement (`refine_upper_bound`, `refine_lower_bound`) and arithmetic operations.

### Expression Trees

**Boolean expressions** (`BoolExpr`):
```
BoolExpr ::= Const(bool)
           | Var(BoolVar)
           | NVar(BoolVar, IntVar, IntExpr)     // "named" var reference
           | And(Vec<BoolExpr>)
           | Or(Vec<BoolExpr>)
           | Not(BoolExpr)
           | Xor(BoolExpr, BoolExpr)
           | Iff(BoolExpr, BoolExpr)             // biconditional
           | Imp(BoolExpr, BoolExpr)             // implication
           | Cmp(CmpOp, IntExpr, IntExpr)        // integer comparison
```

**Integer expressions** (`IntExpr`):
```
IntExpr ::= Const(i32)
          | Var(IntVar)
          | NVar(BoolVar, IntVar, IntExpr)
          | Linear(Vec<(IntExpr, coef)>)          // weighted sum
          | If(BoolExpr, IntExpr, IntExpr)         // conditional
          | Abs(IntExpr)                           // absolute value
          | Mul(IntExpr, IntExpr)                  // multiplication
```

### Constraint Statements

```rust
enum Stmt {
    Expr(BoolExpr),                              // arbitrary boolean constraint
    AllDifferent(Vec<IntExpr>),                   // global constraint
    ActiveVerticesConnected(Vec<BoolExpr>, Graph), // connectivity
    Circuit(Vec<IntExpr>),                        // Hamiltonian circuit
    ExtensionSupports(Vec<IntExpr>, Vec<Vec<i32>>), // table constraint
    GraphDivision { ... },                        // graph partitioning
    CustomConstraint(Box<dyn ...>, ...),          // user propagator
}
```

### CSP Optimization

Before normalization, the CSP runs two optimizations:

1. **Constant Folding** (`apply_constant_folding`): Simplifies expressions where variable values are already known. E.g., `And(true, x)` → `x`.

2. **Constant Propagation** (`constant_prop_bool`): Iteratively fixes variables whose values can be deduced from existing constraints. E.g., if constraint `!z` exists, z is forced to `false`, which may cascade to simplify other constraints.

---

## Stage 4: Normalization — CSP → NormCSP

**File:** `cspuz_core/src/normalizer.rs` (1569 lines)

The normalizer transforms the tree-structured CSP into a flattened **NormCSP** that's closer to what the SAT encoder can consume. This is the heaviest transformation stage.

### NormCSP Representation

**Files:** `cspuz_core/src/norm_csp/mod.rs` (673 lines)

```rust
// A normalized boolean literal
struct BoolLit { var: BoolVar, negated: bool }

// A normalized integer linear sum: c₁·x₁ + c₂·x₂ + ... + constant
struct LinearSum<IntVar> { terms: Vec<(IntVar, coef)>, constant: i32 }

// A normalized linear constraint: "sum op 0" where op ∈ {Eq, Ne, Le, Lt, Ge, Gt}
struct LinearLit { sum: LinearSum, op: CmpOp }

// A clause-like constraint: disjunction of bool lits and linear lits
// "at least one of these must be true/satisfied"
struct Constraint {
    bool_lit: Vec<BoolLit>,
    linear_lit: Vec<LinearLit>,
}

// Constraints that can't be expressed as simple clauses
enum ExtraConstraint {
    ActiveVerticesConnected(Vec<Lit>, Graph),
    Mul(IntVar, IntVar, IntVar),            // z = x * y
    ExtensionSupports(Vec<IntVar>, Vec<Vec<i32>>),
    GraphDivision { ... },
    CustomConstraint { ... },
}
```

### What the Normalizer Does

#### Step 1: Merge Equivalent Variables

Scans for `Iff(x, y)` and `Xor(x, y)` constraints that establish variable equivalences. Uses a union-find structure to merge them, reducing variable count.

#### Step 2: Remove Fixed Variables

Variables that constant propagation determined are fixed get removed from the constraint set and marked `Removed` in the `NormalizeMap`.

#### Step 3: Tseitin Transformation

The critical step for avoiding exponential blowup. Consider converting `Iff(a, b)` to CNF naively:

```
a ↔ b = (a → b) ∧ (b → a) = (¬a ∨ b) ∧ (¬b ∨ a)
```

That's fine for simple cases, but nested Iff/Xor in deeply nested expressions can produce an exponential number of clauses. The **Tseitin transformation** introduces auxiliary boolean variables to keep clause count polynomial:

```
Original:  Iff(Iff(a, b), Iff(c, d))

Tseitin:   Let t₁ = (a ↔ b)    — new aux var
           Let t₂ = (c ↔ d)    — new aux var
           Clauses for t₁ ↔ (a ↔ b):  4 clauses
           Clauses for t₂ ↔ (c ↔ d):  4 clauses
           Clause for t₁ ↔ t₂:         4 clauses
           Total: 12 clauses (linear) instead of potentially exponential
```

#### Step 4: Boolean Expression Normalization

Recursively converts `BoolExpr` trees into `Vec<Constraint>` (conjunction of clauses):

- `And(a, b)` → clauses(a) ∪ clauses(b)
- `Or(a, b)` → distribute: cross-product of clause sets
- `Not(And(a, b))` → Or(Not(a), Not(b)) via De Morgan
- `Cmp(op, int_a, int_b)` → `LinearLit` comparing normalized integer expressions

#### Step 5: Integer Expression Normalization

Converts `IntExpr` trees into `LinearSum`:

- `Const(c)` → LinearSum with constant c
- `Var(x)` → LinearSum: 1·x + 0
- `Linear(terms)` → Flatten and combine coefficients
- `If(cond, t, f)` → Introduce a binary integer variable: `b ∈ {0, 1}`, constrain `b ↔ cond`, result = `f + b·(t - f)`
- `Abs(x)` → Convert to `If(x ≥ 0, x, -x)`
- `Mul(x, y)` → Cannot be linearized; emitted as `ExtraConstraint::Mul`

#### Step 6: Global Constraint Normalization

- `AllDifferent(exprs)` → Pairwise `Ne` constraints: for all i < j, `exprs[i] ≠ exprs[j]`. Plus optional bijection optimization when domain size equals variable count.
- `Circuit(exprs)` → Edge variables + connectivity constraint + in-degree/out-degree = 1.
- `ActiveVerticesConnected` → Passed through as `ExtraConstraint`.
- `GraphDivision`, `CustomConstraint` → Wrapped as `ExtraConstraint`.

### Domain Refinement

After normalization, `NormCSP.refine_domain()` iteratively tightens variable domains by analyzing the normalized constraints. This reduces the number of SAT variables needed in the encoding stage.

---

## Stage 5: Encoding — NormCSP → SAT Clauses

**Files:** `cspuz_core/src/encoder/mod.rs` (1568 lines), `encoder/order.rs`, `encoder/direct.rs`, `encoder/log.rs`, `encoder/mixed.rs`

This stage converts normalized CSP constraints into raw SAT clauses — the input format for CDCL solvers.

### The Core Challenge

SAT solvers work with boolean variables and clauses (disjunctions of literals). But our constraints involve **integer variables** with domains like [1, 9]. We need a scheme to represent "integer variable x has value 5" using boolean variables.

### Three Encoding Schemes

#### Order Encoding (Default)

For an integer variable x with domain {d₀, d₁, ..., dₙ}, create n boolean literals:

```
lit₀ = "x ≥ d₁"    (i.e., x is at least the second-smallest value)
lit₁ = "x ≥ d₂"
...
litₙ₋₁ = "x ≥ dₙ"
```

**Chain constraint**: lit₀ ← lit₁ ← ... ← litₙ₋₁ (if x ≥ d₃, then x ≥ d₂ and x ≥ d₁)

**Reading the value**: The value of x is determined by finding where the chain transitions from true to false:
- All false → x = d₀
- lit₀ true, lit₁ false → x = d₁
- All true → x = dₙ

**Why it's the default**: Order encoding naturally supports **linear constraints** (inequalities, sums). The clause `x ≥ 5` is a single literal. The constraint `x + y ≤ 10` decomposes naturally into order-encoded clauses.

```rust
// encoder/order.rs
struct OrderEncoding {
    domain: Vec<CheckedInt>,  // [d₀, d₁, ..., dₙ]
    lits: Vec<Lit>,           // [lit₀, lit₁, ..., litₙ₋₁]
    // lits[i] means "x ≥ domain[i+1]"
}
```

#### Direct Encoding

For an integer variable x with domain {d₀, d₁, ..., dₙ}, create n+1 boolean literals:

```
lit₀ = "x = d₀"
lit₁ = "x = d₁"
...
litₙ = "x = dₙ"
```

**Exactly-one constraint**: Exactly one litᵢ must be true (at-least-one + pairwise at-most-one).

**Why use it**: Direct encoding is more efficient for **equality/inequality constraints** (x = 5, x ≠ 3), which are single-literal operations. Used when:
- `config.use_direct_encoding` is enabled, AND
- The variable appears only in simple Eq/Ne constraints with ≤ 2 terms, AND
- Domain size ≤ 500

**Exception**: Variables involved in `GraphDivision` size constraints are always order-encoded.

```rust
// encoder/direct.rs
struct DirectEncoding {
    domain: Vec<CheckedInt>,  // [d₀, d₁, ..., dₙ]
    lits: Vec<Lit>,           // [lit₀, lit₁, ..., litₙ]
    // lits[i] means "x = domain[i]"
}
```

#### Log Encoding (Feature-gated)

Represent x using ⌈log₂(|domain|)⌉ boolean bits. The value is reconstructed as a binary number.

**Why use it**: Dramatically fewer SAT variables for large domains. Used when:
- `config.force_use_log_encoding` is set, OR
- Domain has > 500 values AND variable appears in linear constraints with 3+ terms

**Propagation**: If one variable in a linear constraint is log-encoded, all co-occurring variables in that constraint must also be log-encoded.

### Encoding Scheme Selection

```rust
fn decide_encode_schemes(norm: &NormCSP, config: &Config) -> Vec<EncodeScheme> {
    // 1. Check if log encoding is forced
    // 2. Find vars with domain > 500 in 3+ term linear constraints → Log
    // 3. Propagate: co-occurring vars with log vars → Log
    // 4. Check direct encoding eligibility (simple constraints, small domain)
    // 5. Everything else → Order (the default)
}
```

### Encoding Constraints into Clauses

Each `Constraint { bool_lits, linear_lits }` is encoded based on the `suggest_encoder()` decision:

| Encoder | Used When | How |
|---------|-----------|-----|
| `MixedGe` | Order-encoded vars in ≥ constraint | Builds clauses from order-encoding lits directly; for complex linear sums, decomposes into auxiliary order-encoded vars |
| `DirectSimple` | Direct-encoded vars in Eq/Ne | Single literal for Eq; negated literal for Ne |
| `DirectEqNe` | Mixed direct-encoded vars | Combines eq/ne lits into clauses |
| `Log` | Log-encoded vars | Binary arithmetic to SAT clauses |

### Linear Constraint Decomposition

When a linear constraint `c₁x₁ + c₂x₂ + ... + cₖxₖ ≤ b` involves many variables or large domain products, the encoder decomposes it:

1. Split the sum into sub-sums: `(c₁x₁ + c₂x₂) + (c₃x₃ + c₄x₄) + ... ≤ b`
2. Introduce auxiliary order-encoded variables for each sub-sum
3. Encode the smaller constraints individually
4. Combine via the auxiliary variables

This keeps clause count manageable for constraints that would otherwise produce enormous cross-products.

### Extra Constraint Encoding

Constraints that can't be expressed as SAT clauses are dispatched to **native solver propagators**:

| Extra Constraint | SAT Backend Method | What It Does |
|------------------|--------------------|--------------|
| `ActiveVerticesConnected` | `sat.add_active_vertices_connected()` | Inline propagator ensuring marked vertices form a connected subgraph |
| `GraphDivision` | `sat.add_graph_division()` | Inline propagator for graph partitioning with size bounds |
| `ExtensionSupports` | `sat.add_direct_encoding_extension_supports()` | Table constraint: variable tuple must match one of the given support tuples |
| `Mul` | Log: `encode_mul_log()` / Order: `encode_mul_naive()` | Multiplication z = x·y via bit-level or case-splitting |
| `CustomConstraint` | `sat.add_custom_constraint()` | User-defined propagator (Glucose only) |

---

## Stage 6: SAT Solving

**Files:** `cspuz_core/src/sat.rs` (655 lines), `cspuz_core/src/backend/glucose.rs` (731 lines), `cspuz_core/src/backend/cadical.rs` (128 lines)

### The SAT Abstraction Layer

```rust
// sat.rs
struct Var(i32);  // SAT variable (internal id)
struct Lit(i32);  // Literal: var * 2 + negated_bit. !lit = lit XOR 1.

enum SAT {
    Glucose(glucose::Solver),
    CaDiCaL(cadical::Solver),
    External(external::Solver),  // feature-gated
}
```

Key operations:
```rust
impl SAT {
    fn new_var(&mut self) -> Var;
    fn add_clause(&mut self, clause: &[Lit]);
    fn solve(&mut self) -> Option<SATModel>;
    fn solve_without_model(&mut self) -> bool;

    // Native propagator interfaces (Glucose only for most):
    fn add_order_encoding_linear(...);
    fn add_active_vertices_connected(...);
    fn add_graph_division(...);
    fn add_direct_encoding_extension_supports(...);
    fn add_custom_constraint(...);
}
```

### Backend: Glucose (Default)

**File:** `cspuz_core/src/backend/glucose.rs`

Glucose is a high-performance CDCL (Conflict-Driven Clause Learning) SAT solver. cspuz_core uses it via FFI bindings to the C++ implementation:

```rust
extern "C-unwind" {
    fn Glucose_CreateSolver() -> *mut c_void;
    fn Glucose_NewVar(solver: *mut c_void) -> i32;
    fn Glucose_AddClause(solver: *mut c_void, lits: *const i32, n: i32);
    fn Glucose_Solve(solver: *mut c_void) -> i32;
    fn Glucose_GetModelValueVar(solver: *mut c_void, var: i32) -> i32;
    fn Glucose_SetPolarity(solver: *mut c_void, var: i32, pol: i32);

    // Native propagators — these run INSIDE the CDCL loop:
    fn Glucose_AddOrderEncodingLinear(...);
    fn Glucose_AddActiveVerticesConnected(...);
    fn Glucose_AddGraphDivision(...);
    fn Glucose_AddDirectEncodingExtensionSupports(...);

    // Statistics:
    fn Glucose_SolverStats_decisions(solver: *mut c_void) -> u64;
    fn Glucose_SolverStats_propagations(solver: *mut c_void) -> u64;
    fn Glucose_SolverStats_conflicts(solver: *mut c_void) -> u64;
}
```

**Why Glucose is the default**: It supports native constraint propagators embedded directly in the CDCL search loop. This is critical for puzzle solving — constraints like "these cells must form a connected region" are vastly more efficient as inline propagators than as clause decompositions.

### CDCL Algorithm (How SAT Solvers Work)

The solver maintains:
- **Assignment trail**: Current partial assignment of variables to true/false
- **Clause database**: All clauses (original + learned)
- **Watch lists**: For each literal, which clauses are "watching" it

The algorithm:
1. **Unit Propagation**: If a clause has only one unassigned literal, that literal must be true. Propagate transitively.
2. **Decision**: If no conflict and not all assigned, pick an unassigned variable and guess a value. Push a decision level.
3. **Conflict Analysis**: If a clause becomes falsified (all literals false), analyze the conflict:
   - Trace back implications to find a **conflict clause** (1-UIP)
   - **Learn** this clause (add to database) — prevents the same conflict pattern forever
   - **Backjump** to the appropriate decision level (non-chronological backtracking)
4. **Restart**: Periodically restart from scratch (keeping learned clauses) for diversification.
5. **Repeat** until: all variables assigned (SAT) or conflict at decision level 0 (UNSAT).

### Native Propagators — The Secret Weapon

Standard SAT solving encodes everything as clauses. But some puzzle constraints (connectivity, graph partitioning) would require an astronomical number of clauses. Instead, cspuz_core embeds **custom propagators** directly in the CDCL loop.

#### Custom Propagator Interface

```rust
trait CustomPropagator<T> {
    fn initialize(&mut self, solver: &mut T);
    fn propagate(&mut self, solver: &mut T, p: Lit, num_pending: i32) -> bool;
    fn calc_reason(&mut self, solver: &mut T, p: Lit, extra: Lit) -> Vec<Lit>;
    fn undo(&mut self, solver: &mut T, p: Lit);
}
```

- **`propagate`**: Called by the solver whenever a watched literal `p` is assigned. The propagator can deduce new assignments (return implications) or detect conflicts (return false).
- **`calc_reason`**: When the solver needs to understand WHY a propagated literal was implied, the propagator must produce a clause (reason) explaining it.
- **`undo`**: Called on backtrack. The propagator reverts its internal state.

This means the propagator participates fully in CDCL: its implications can be part of conflict analysis, and its deductions are undone on backtrack.

#### Graph Division Propagator

**File:** `cspuz_core/src/propagators/graph_division.rs` (1522 lines)

The most complex propagator. It handles constraints of the form "divide this graph into connected regions of specified sizes."

Internal state:
- **Union-find** for tracking connected components
- **Edge states**: Undecided / Connected / Disconnected
- **Region size bounds**: lower and upper bounds per component

Propagation reasons:
- `EdgeInSameGroup`: Two endpoints assigned to the same group → edge must be connected
- `EdgeBetweenDifferentGroups`: Endpoints in different groups → edge must be disconnected
- `TooLargeIfRegionsAreMerged`: Merging regions would exceed size bound → edge must be disconnected
- `RegionAlreadyLarge/Small`: Region can't grow/shrink further
- `InconsistentBoundsIfRegionsAreMerged`: Merged bounds would be contradictory

#### Active Vertices Connected Propagator

Ensures that all "active" vertices (those whose boolean literal is true) form a single connected component in the graph. Used for constraints like "all black cells must be connected" in Nurikabe.

### Backend: CaDiCaL (Alternative)

**File:** `cspuz_core/src/backend/cadical.rs`

A more minimal backend. CaDiCaL is another top-tier CDCL solver, but the cspuz_core integration supports only:
- Basic clause operations
- `AddActiveVerticesConnected` (connectivity propagator)

No support for: custom propagators, graph division, table constraints, or order-encoding linear constraints. Use Glucose for puzzles requiring these features.

### Backend Selection

Feature-gated at compile time:
- `glucose` — Always available (default)
- `backend-cadical` — Optional
- `backend-external` — External solver via stdio

---

## Stage 7: Solution Extraction & Uniqueness

**Files:** `cspuz_core/src/integration.rs` (451 lines), `cspuz_solver_backend/src/uniqueness.rs` (70 lines)

### The IntegratedSolver

```rust
struct IntegratedSolver {
    csp: CSP,                     // High-level constraints
    normalize_map: NormalizeMap,   // CSP var → NormCSP var mapping
    norm: NormCSP,                // Normalized constraints
    encode_map: EncodeMap,        // NormCSP var → SAT var mapping
    sat: SAT,                     // The SAT solver instance
    config: Config,
    perf_stats: PerfStats,
}
```

### Solving

```rust
impl IntegratedSolver {
    fn solve(&mut self) -> Option<Model> {
        self.encode();              // Run stages 4-5
        if self.sat.solve_without_model() {
            Some(Model { ... })     // SAT → extract model
        } else {
            None                    // UNSAT → no solution
        }
    }

    fn encode(&mut self) {
        self.csp.optimize(true, false);           // Stage 3: constant folding/propagation
        normalize(&self.csp, &mut self.norm, ...); // Stage 4: CSP → NormCSP
        self.norm.refine_domain();                 // Domain tightening
        encode(&self.norm, &mut self.sat, ...);    // Stage 5: NormCSP → SAT clauses
    }
}
```

### Reading Solutions Back

The `Model` struct traces backward through the pipeline layers:

```
SAT model (booleans) → EncodeMap → NormCSP assignment → NormalizeMap → CSP assignment
```

```rust
impl Model {
    fn get_bool(&self, var: BoolVar) -> bool {
        // normalize_map: BoolVar → Lit (NormCSP level)
        // encode_map: NormCSP Lit → SAT Lit
        // sat_model: SAT Lit → true/false
    }

    fn get_int(&self, var: IntVar) -> i32 {
        // normalize_map: IntVar → NIntVar
        // encode_map.get_int_value_checked(sat_model, NIntVar):
        //   Order: binary search on order-encoding lits
        //   Direct: find the one true lit
        //   Log: reconstruct integer from binary bits
    }
}
```

### Uniqueness Checking — `decide_irrefutable_facts()`

This is how cspuz_core determines not just a solution, but whether the solution is **unique**. It finds values that are the same across ALL valid solutions:

```
Algorithm: decide_irrefutable_facts(answer_vars)
1. Solve the puzzle → get solution S₁
2. Record assignment A = {var₁=v₁, var₂=v₂, ...} for answer variables
3. Add refutation clause: ¬(var₁=v₁ ∧ var₂=v₂ ∧ ...) — "find a DIFFERENT solution"
4. Solve again → get solution S₂ (or UNSAT)
5. If UNSAT: A contains the irrefutable facts. Every answer var has a unique value. DONE.
6. If SAT: Compare S₁ and S₂. Remove from A any variable where S₁ and S₂ disagree.
7. Add another refutation clause excluding S₂.
8. Repeat from step 4.

Converges when: UNSAT (all remaining assignments in A are irrefutable)
```

**Polarity optimization**: The solver sets variable polarities to prefer the *opposite* of the current assignment, accelerating discovery of differing solutions.

The result:
- If all answer variables have irrefutable values → **Unique** solution
- If some variables have `None` (no irrefutable value) → **NonUnique** (multiple solutions exist)
- If first solve returns UNSAT → **NoAnswer** (no solution exists)

```rust
// uniqueness.rs
enum Uniqueness {
    Unique,
    NonUnique,
    NotApplicable,
    NoAnswer,
}
```

### Performance Statistics

The `PerfStats` struct tracks:
- `time_normalize` — Time spent in normalization (Stage 4)
- `time_encode` — Time spent in encoding (Stage 5)
- `time_sat_solver` — Time spent in SAT solving (Stage 6)
- `decisions` — Number of CDCL decisions
- `propagations` — Number of unit propagations
- `conflicts` — Number of conflicts (each produces a learned clause)
- `iterations` — Number of solve iterations (for uniqueness checking)

---

## End-to-End Example: Solving a 4×4 Sudoku

To make the pipeline concrete, here's what happens for a simple 4×4 Sudoku with clues:

```
 _  2  _  _
 _  _  _  3
 3  _  _  _
 _  _  1  _
```

### Stage 2 — Constraint Construction

```
Variables: 16 integer variables, each with domain {1, 2, 3, 4}
  x₀₀ x₀₁ x₀₂ x₀₃
  x₁₀ x₁₁ x₁₂ x₁₃
  x₂₀ x₂₁ x₂₂ x₂₃
  x₃₀ x₃₁ x₃₂ x₃₃

Constraints:
  Clues:        x₀₁ = 2, x₁₃ = 3, x₂₀ = 3, x₃₂ = 1
  Row alldiff:  AllDifferent(x₀₀, x₀₁, x₀₂, x₀₃), ... (4 constraints)
  Col alldiff:  AllDifferent(x₀₀, x₁₀, x₂₀, x₃₀), ... (4 constraints)
  Box alldiff:  AllDifferent(x₀₀, x₀₁, x₁₀, x₁₁), ... (4 constraints)
```

### Stage 3 — CSP Optimization

Constant propagation fixes the clue variables. This ripples:
- x₀₁ = 2 → removed from all_different constraints in row 0, col 1, box 0
- Other cells' effective domains shrink

### Stage 4 — Normalization

- `AllDifferent(x₀₀, x₀₂, x₀₃)` → pairwise: `x₀₀ ≠ x₀₂`, `x₀₀ ≠ x₀₃`, `x₀₂ ≠ x₀₃`
- Each `xᵢⱼ ≠ xₖₗ` → `LinearLit` or boolean constraint
- Fixed variables removed, domains tightened

### Stage 5 — Encoding (Order)

For each remaining variable with domain {1, 2, 3, 4}:
```
x₀₀: lit₀ = "x₀₀ ≥ 2", lit₁ = "x₀₀ ≥ 3", lit₂ = "x₀₀ ≥ 4"
Chain: lit₂ → lit₁ → lit₀
```

`x₀₀ ≠ x₀₂` becomes: for each value v, ¬("x₀₀ = v" ∧ "x₀₂ = v").

In order encoding, "x₀₀ = 2" = "x₀₀ ≥ 2" ∧ "x₀₀ < 3" = lit₀ ∧ ¬lit₁.

### Stage 6 — SAT Solving

The CDCL solver processes ~50–100 clauses for this small puzzle. With the chain constraints and inequality clauses, it finds a satisfying assignment in a few decisions.

### Stage 7 — Solution Extraction

```
SAT model: lit₀₀₀ = true, lit₀₀₁ = false, ...
→ Order decode: x₀₀ ≥ 2 but x₀₀ < 3, so x₀₀ = 2? No wait, x₀₁ = 2...
→ (After proper decode through all layers)

 1  2  3  4
 4  1  2  3
 3  4  1  2?  (Not necessarily — actual solution depends on constraints)

Uniqueness: solve again with refutation clause → UNSAT → Unique!
```

---

## Source File Reference

| File | Lines | Role |
|------|-------|------|
| `cspuz_solver_backend/src/lib.rs` | ~200 | Entry: URL parse → dispatch |
| `cspuz_solver_backend/src/puzzle/*.rs` | 170+ files | Puzzle-specific dispatchers |
| `cspuz_rs_puzzles/src/puzzles/*.rs` | 170+ files | Puzzle constraint construction |
| `cspuz_rs/src/solver/mod.rs` | 751 | High-level Solver API |
| `cspuz_rs/src/graph.rs` | 1273 | Graph utilities + connectivity |
| `cspuz_core/src/csp/repr.rs` | 453 | BoolExpr, IntExpr, Stmt types |
| `cspuz_core/src/csp/mod.rs` | 1222 | CSP struct + optimization |
| `cspuz_core/src/domain.rs` | 252 | Variable domains |
| `cspuz_core/src/normalizer.rs` | 1569 | CSP → NormCSP (Tseitin etc.) |
| `cspuz_core/src/norm_csp/mod.rs` | 673 | NormCSP representation |
| `cspuz_core/src/encoder/mod.rs` | 1568 | NormCSP → SAT clauses |
| `cspuz_core/src/encoder/order.rs` | 235 | Order encoding impl |
| `cspuz_core/src/encoder/direct.rs` | 509 | Direct encoding impl |
| `cspuz_core/src/encoder/log.rs` | ~200 | Log encoding impl |
| `cspuz_core/src/encoder/mixed.rs` | ~300 | Mixed encoding strategies |
| `cspuz_core/src/sat.rs` | 655 | SAT abstraction layer |
| `cspuz_core/src/backend/glucose.rs` | 731 | Glucose FFI bindings |
| `cspuz_core/src/backend/cadical.rs` | 128 | CaDiCaL FFI bindings |
| `cspuz_core/src/integration.rs` | 451 | IntegratedSolver orchestrator |
| `cspuz_core/src/propagators/graph_division.rs` | 1522 | Graph division propagator |
| `cspuz_core/src/propagators/order_encoding_linear.rs` | ~200 | Order-encoded linear propagator |
| `cspuz_core/src/custom_constraints.rs` | ~100 | Custom constraint interface |
| `cspuz_solver_backend/src/uniqueness.rs` | 70 | Uniqueness checking |
