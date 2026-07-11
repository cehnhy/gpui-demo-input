# Code Provider Search Roots Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Search directories up to two levels below `$HOME/repo` and first-level directories under `$HOME`, returning at most six unique code targets.

**Architecture:** The code provider builds two search specifications, runs `fd` independently for each, then merges successful path output through a pure deduplication and limiting helper. Pure helpers are unit tested without invoking external commands.

**Tech Stack:** Rust 2021, `std::process::Command`, `PathBuf`, `HashSet`, Rust unit tests.

---

### Task 1: Define Search Specifications

**Files:**
- Modify: `src/app/query_provider/code.rs`
- Test: `src/app/query_provider/code.rs`

- [ ] **Step 1: Add a failing test for root order and depth**

Add a test expecting `search_specs(Path::new("/home/test"))` to return:

```rust
vec![
    SearchSpec {
        root: PathBuf::from("/home/test/repo"),
        max_depth: Some(2),
    },
    SearchSpec {
        root: PathBuf::from("/home/test"),
        max_depth: Some(1),
    },
]
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test code::tests::builds_repo_and_home_search_specs --lib`

Expected: compilation fails because `SearchSpec` and `search_specs` do not exist.

- [ ] **Step 3: Implement the search specification helper**

Define a private `SearchSpec { root: PathBuf, max_depth: Option<usize> }` and a
`search_specs(home: &Path) -> Vec<SearchSpec>` helper producing the expected
ordered specifications, limiting the repository root to two levels.

- [ ] **Step 4: Run the focused test**

Run: `cargo test code::tests::builds_repo_and_home_search_specs --lib`

Expected: the test passes.

### Task 2: Define Result Merging

**Files:**
- Modify: `src/app/query_provider/code.rs`
- Test: `src/app/query_provider/code.rs`

- [ ] **Step 1: Add failing merge tests**

Add tests for a pure `merge_paths` helper:

```rust
#[test]
fn merges_paths_in_order_without_duplicates() {
    let paths = merge_paths([
        vec![PathBuf::from("/home/test/repo/a"), PathBuf::from("/home/test/repo/b")],
        vec![PathBuf::from("/home/test/repo/a"), PathBuf::from("/home/test/nixos-config")],
    ]);

    assert_eq!(
        paths,
        vec![
            PathBuf::from("/home/test/repo/a"),
            PathBuf::from("/home/test/repo/b"),
            PathBuf::from("/home/test/nixos-config"),
        ]
    );
}

#[test]
fn limits_merged_paths_to_six() {
    let paths = merge_paths([("0"..="7").map(PathBuf::from).collect()]);

    assert_eq!(paths.len(), 6);
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run: `cargo test code::tests --lib`

Expected: compilation fails because `merge_paths` does not exist.

- [ ] **Step 3: Implement ordered deduplication and limiting**

Use a `HashSet<PathBuf>` for seen paths, append first occurrences in input
order, and stop once six paths have been collected.

- [ ] **Step 4: Run code provider unit tests**

Run: `cargo test code::tests --lib`

Expected: all three code provider tests pass.

### Task 3: Integrate Both Searches

**Files:**
- Modify: `src/app/query_provider/code.rs`

- [ ] **Step 1: Replace the single `fd` invocation**

Read `HOME` with `std::env::var_os`. Return an empty result if it is missing or
empty. For each `SearchSpec`, run `fd` with `--type d` and the query. Add
`--max-depth 1` only for the home specification. Do not apply `--max-results`
to individual searches because the final combined helper owns the six-item
limit.

Collect stdout paths only from successful commands, merge them with
`merge_paths`, and map them through the existing title, subtitle, action, and
icon formatting.

- [ ] **Step 2: Format and run focused tests**

Run: `cargo fmt && cargo test code::tests --lib`

Expected: all code provider tests pass.

- [ ] **Step 3: Run complete verification**

Run: `cargo test --all-targets && cargo check --all-targets && git diff --check`

Expected: all tests pass, all targets compile, and no whitespace errors exist.

- [ ] **Step 4: Commit**

```bash
git add src/app/query_provider/code.rs
git commit -m "feat: expand code provider search roots"
```
