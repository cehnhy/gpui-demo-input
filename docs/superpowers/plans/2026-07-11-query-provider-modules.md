# Query Provider Modules Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move each built-in query provider into its own module without changing query routing or provider behavior.

**Architecture:** `query_parser.rs` retains shared interfaces and routing. A private `query_provider` module owns five provider implementations and exposes one `default_providers()` factory used by `DefaultQueryParser::default()`.

**Tech Stack:** Rust 2021 modules, standard library trait objects, existing query parser unit tests.

---

### Task 1: Define the Provider Factory Contract

**Files:**
- Modify: `src/app/query_parser.rs`
- Test: `src/app/query_parser.rs`

- [ ] **Step 1: Change default construction to call the new factory**

Replace the inline provider vector with:

```rust
impl Default for DefaultQueryParser {
    fn default() -> Self {
        Self::new(super::query_provider::default_providers())
    }
}
```

Keep `default_parser_registers_existing_triggers` unchanged so it continues to
verify the provider names and order.

- [ ] **Step 2: Run the focused test and verify it fails**

Run: `cargo test default_parser_registers_existing_triggers --lib`

Expected: compilation fails because `app::query_provider` and
`default_providers()` do not exist.

### Task 2: Extract Provider Implementations

**Files:**
- Modify: `src/app/mod.rs`
- Modify: `src/app/query_parser.rs`
- Create: `src/app/query_provider/mod.rs`
- Create: `src/app/query_provider/application.rs`
- Create: `src/app/query_provider/clipboard.rs`
- Create: `src/app/query_provider/code.rs`
- Create: `src/app/query_provider/float.rs`
- Create: `src/app/query_provider/time.rs`

- [ ] **Step 1: Declare the private provider module**

Add this declaration to `src/app/mod.rs`:

```rust
mod query_provider;
```

- [ ] **Step 2: Add the provider factory**

Create `src/app/query_provider/mod.rs`:

```rust
mod application;
mod clipboard;
mod code;
mod float;
mod time;

use super::query_parser::QueryProvider;
use application::ApplicationQueryProvider;
use clipboard::ClipboardQueryProvider;
use code::CodeQueryProvider;
use float::FloatQueryProvider;
use time::TimeQueryProvider;

pub(super) fn default_providers() -> Vec<Box<dyn QueryProvider>> {
    vec![
        Box::new(ApplicationQueryProvider),
        Box::new(CodeQueryProvider),
        Box::new(FloatQueryProvider),
        Box::new(TimeQueryProvider),
        Box::new(ClipboardQueryProvider),
    ]
}
```

- [ ] **Step 3: Move each provider unchanged**

Each new file imports `QueryParserItem` and `QueryProvider` from
`crate::app::query_parser`, defines its provider as `pub(super) struct`, and
moves the matching implementation body unchanged from `query_parser.rs`.

Example structure for `float.rs`:

```rust
use crate::app::query_parser::{QueryParserItem, QueryProvider};
use std::path::PathBuf;

pub(super) struct FloatQueryProvider;

impl QueryProvider for FloatQueryProvider {
    fn trigger(&self) -> &'static str {
        "float"
    }

    fn parse(&self, _args: &[String]) -> Vec<QueryParserItem> {
        vec![QueryParserItem {
            title: "toggle floating".to_string(),
            subtitle: String::new(),
            action: "niri msg action toggle-window-floating".to_string(),
            icon: PathBuf::new(),
        }]
    }
}
```

Remove the provider-specific imports and all five concrete provider definitions
from `query_parser.rs`.

- [ ] **Step 4: Run focused tests**

Run: `cargo test query_parser::tests --lib`

Expected: all five query parser tests pass.

- [ ] **Step 5: Commit module extraction**

```bash
git add src/app/mod.rs src/app/query_parser.rs src/app/query_provider
git commit -m "refactor: split query providers into modules"
```

### Task 3: Verify the Refactor

**Files:**
- Verify: `src/app/query_parser.rs`
- Verify: `src/app/query_provider/*.rs`

- [ ] **Step 1: Format all changed Rust files**

Run: `cargo fmt`

Expected: formatting completes successfully.

- [ ] **Step 2: Run all target tests**

Run: `cargo test --all-targets`

Expected: five library tests pass and all other targets report zero failures.

- [ ] **Step 3: Compile all targets**

Run: `cargo check --all-targets`

Expected: compilation succeeds without new warnings.

- [ ] **Step 4: Check scope and whitespace**

Run: `git diff --check && git status --short`

Expected: no whitespace errors and no unrelated files changed.
