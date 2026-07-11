# Query Parser Interfaces Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the stateful query parser with a consumer-facing parser trait and independently extensible trigger providers while preserving current query behavior.

**Architecture:** `DefaultQueryParser` implements the public `QueryParser` trait and routes input to boxed `QueryProvider` implementations. Each existing trigger becomes a dedicated provider in `query_parser.rs`, and the input component consumes the parser through the trait.

**Tech Stack:** Rust 2021, standard library trait objects, existing `chrono`, `freedesktop-desktop-entry`, and `freedesktop-icons` dependencies, Rust unit tests.

---

## File Structure

- Modify `src/app/query_parser.rs`: define both traits, implement routing, move existing trigger logic into provider types, and add routing unit tests.
- Modify `src/app/input.rs`: construct `DefaultQueryParser` and invoke the `QueryParser` trait method.

### Task 1: Define Routing Behavior With Tests

**Files:**
- Modify: `src/app/query_parser.rs`
- Test: `src/app/query_parser.rs`

- [ ] **Step 1: Add failing routing tests**

Add a `#[cfg(test)]` module containing a fake provider that records its arguments in `Rc<RefCell<Vec<Vec<String>>>>` and returns an item identifying the provider. Add tests for:

```rust
#[test]
fn routes_explicit_trigger_to_matching_provider() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let parser = DefaultQueryParser::new(vec![Box::new(FakeProvider::new(
        "code",
        Rc::clone(&calls),
    ))]);

    let items = parser.parse("code gpui demo");

    assert_eq!(items[0].title, "code");
    assert_eq!(calls.borrow().as_slice(), &[vec!["gpui".into(), "demo".into()]]);
}

#[test]
fn routes_unprefixed_query_to_application_provider() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let parser = DefaultQueryParser::new(vec![Box::new(FakeProvider::new(
        "application",
        Rc::clone(&calls),
    ))]);

    parser.parse("visual studio code");

    assert_eq!(
        calls.borrow().as_slice(),
        &[vec!["visual".into(), "studio".into(), "code".into()]]
    );
}

#[test]
fn handles_empty_query_without_panicking() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let parser = DefaultQueryParser::new(vec![Box::new(FakeProvider::new(
        "application",
        Rc::clone(&calls),
    ))]);

    parser.parse("");

    assert_eq!(calls.borrow().as_slice(), &[Vec::<String>::new()]);
}

#[test]
fn returns_no_items_without_matching_or_fallback_provider() {
    let parser = DefaultQueryParser::new(Vec::new());

    assert!(parser.parse("unknown query").is_empty());
}
```

Derive `Debug`, `PartialEq`, and `Eq` for `QueryParserItem` if needed by focused assertions.

- [ ] **Step 2: Run tests and verify they fail**

Run: `cargo test query_parser::tests --lib`

Expected: compilation fails because `DefaultQueryParser`, `QueryParser`, and `QueryProvider` are not defined yet.

- [ ] **Step 3: Define the interfaces and minimal router**

Replace the old stateful `QueryParser` struct and `to_query_parser` function with:

```rust
pub trait QueryParser {
    fn parse(&self, query: &str) -> Vec<QueryParserItem>;
}

pub trait QueryProvider {
    fn trigger(&self) -> &'static str;
    fn parse(&self, args: &[String]) -> Vec<QueryParserItem>;
}

pub struct DefaultQueryParser {
    providers: Vec<Box<dyn QueryProvider>>,
}

impl DefaultQueryParser {
    pub fn new(providers: Vec<Box<dyn QueryProvider>>) -> Self {
        Self { providers }
    }
}

impl QueryParser for DefaultQueryParser {
    fn parse(&self, query: &str) -> Vec<QueryParserItem> {
        let tokens = query.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        let explicit = tokens
            .first()
            .and_then(|trigger| self.providers.iter().find(|p| p.trigger() == trigger));

        if let Some(provider) = explicit {
            return provider.parse(&tokens[1..]);
        }

        self.providers
            .iter()
            .find(|provider| provider.trigger() == "application")
            .map(|provider| provider.parse(&tokens))
            .unwrap_or_default()
    }
}
```

- [ ] **Step 4: Run routing tests and verify they pass**

Run: `cargo test query_parser::tests --lib`

Expected: all four routing tests pass.

- [ ] **Step 5: Commit the routing interfaces**

```bash
git add src/app/query_parser.rs
git commit -m "refactor: add query parser interfaces"
```

### Task 2: Extract Existing Trigger Providers

**Files:**
- Modify: `src/app/query_parser.rs`

- [ ] **Step 1: Add provider registration test**

Add a test proving the default parser recognizes every existing explicit trigger without falling back to application. Inspect provider triggers directly inside the same module:

```rust
#[test]
fn default_parser_registers_existing_triggers() {
    let parser = DefaultQueryParser::default();
    let triggers = parser
        .providers
        .iter()
        .map(|provider| provider.trigger())
        .collect::<Vec<_>>();

    assert_eq!(triggers, vec!["application", "code", "float", "time", "c"]);
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test default_parser_registers_existing_triggers --lib`

Expected: compilation fails because `Default` is not implemented for `DefaultQueryParser`.

- [ ] **Step 3: Move each implementation into a provider**

Create `ApplicationQueryProvider`, `CodeQueryProvider`, `FloatQueryProvider`, `TimeQueryProvider`, and `ClipboardQueryProvider`. For each type, implement `trigger()` with its existing trigger and move the corresponding old `parse_*` body into `QueryProvider::parse(&self, args: &[String])`.

Use guarded argument access instead of indexing:

```rust
let Some(arg) = args.first() else {
    return vec![];
};
```

For application lookup, allow an empty argument by joining the arguments and matching every application when the query is empty:

```rust
let arg = args.join(" ");
```

For code and clipboard providers, preserve their existing requirement for a first argument and existing command behavior. For float and time providers, ignore `args`.

Implement the default provider collection:

```rust
impl Default for DefaultQueryParser {
    fn default() -> Self {
        Self::new(vec![
            Box::new(ApplicationQueryProvider),
            Box::new(CodeQueryProvider),
            Box::new(FloatQueryProvider),
            Box::new(TimeQueryProvider),
            Box::new(ClipboardQueryProvider),
        ])
    }
}
```

- [ ] **Step 4: Run all parser tests**

Run: `cargo test query_parser::tests --lib`

Expected: all routing and provider registration tests pass.

- [ ] **Step 5: Commit provider extraction**

```bash
git add src/app/query_parser.rs
git commit -m "refactor: extract query providers"
```

### Task 3: Migrate the Input Consumer

**Files:**
- Modify: `src/app/input.rs`

- [ ] **Step 1: Update imports and parser invocation**

Replace the module-only import with the interface and implementation:

```rust
use crate::app::query_parser::{DefaultQueryParser, QueryParser};
```

Replace the old construction in `search`:

```rust
let query_parser = DefaultQueryParser::default();
let query_items = query_parser.parse(query);
```

Keep the existing conversion from `QueryParserItem` to `ListItem` unchanged.

- [ ] **Step 2: Format the changed files**

Run: `cargo fmt -- src/app/query_parser.rs src/app/input.rs`

Expected: command exits successfully and Rust formatting is applied.

- [ ] **Step 3: Run the full library test suite**

Run: `cargo test --lib`

Expected: all library tests pass.

- [ ] **Step 4: Run a compile check for all targets**

Run: `cargo check --all-targets`

Expected: all targets compile without errors or warnings introduced by the refactor.

- [ ] **Step 5: Check the final diff**

Run: `git diff --check && git status --short`

Expected: no whitespace errors; only `src/app/query_parser.rs`, `src/app/input.rs`, and this plan document differ from the pre-implementation state.

- [ ] **Step 6: Commit consumer migration**

```bash
git add src/app/query_parser.rs src/app/input.rs
git commit -m "refactor: consume query parser interface"
```
