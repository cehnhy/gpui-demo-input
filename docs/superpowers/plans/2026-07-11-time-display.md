# Live Time Display Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the `time` query provider and render a left-aligned local clock above the query input that refreshes every second.

**Architecture:** Keep time display state inside `Root`, with a small pure formatting helper and a GPUI background task that updates the state once per second through a weak entity reference. Remove the provider module and registration so `time` is no longer parsed as a special query.

**Tech Stack:** Rust 2021, GPUI, gpui-component, chrono, futures timer utilities, Cargo tests and rustfmt.

---

### Task 1: Remove the Time Query Provider

**Files:**
- Delete: `src/app/query_provider/time.rs`
- Modify: `src/app/query_provider/mod.rs`
- Modify: `src/app/query_parser.rs`

- [ ] **Step 1: Change the provider trigger test to describe the desired registry**

In `src/app/query_parser.rs`, change the expected trigger list to:

```rust
assert_eq!(triggers, vec!["application", "code", "float", "c"]);
```

- [ ] **Step 2: Run the provider trigger test and verify RED**

Run:

```bash
cargo test default_parser_registers_expected_providers
```

Expected: FAIL because the actual trigger list still contains `"time"`.

- [ ] **Step 3: Remove the provider implementation and registration**

Delete `src/app/query_provider/time.rs`. In `src/app/query_provider/mod.rs`, remove:

```rust
mod time;
use time::TimeQueryProvider;
```

and remove this entry from `default_providers()`:

```rust
Box::new(TimeQueryProvider),
```

- [ ] **Step 4: Run the provider tests and verify GREEN**

Run:

```bash
cargo test query_parser
```

Expected: all query parser tests PASS and the trigger list contains no `time` provider.

- [ ] **Step 5: Commit the provider removal**

```bash
git add src/app/query_provider/mod.rs src/app/query_provider/time.rs src/app/query_parser.rs
git commit -m "refactor: remove time query provider"
```

### Task 2: Add a Testable Time Formatter

**Files:**
- Modify: `src/app/input.rs`

- [ ] **Step 1: Add a failing formatter test**

Add this test module at the end of `src/app/input.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::format_time;
    use chrono::TimeZone;

    #[test]
    fn formats_time_for_the_launcher_label() {
        let time = chrono::Local
            .with_ymd_and_hms(2026, 7, 11, 14, 30, 25)
            .single()
            .expect("test time should be valid");

        assert_eq!(format_time(time), "2026-07-11 14:30:25");
    }
}
```

- [ ] **Step 2: Run the formatter test and verify RED**

Run:

```bash
cargo test formats_time_for_the_launcher_label
```

Expected: compilation FAIL because `format_time` does not exist.

- [ ] **Step 3: Add the minimal formatter**

Add the chrono imports and helper near the constants in `src/app/input.rs`:

```rust
use chrono::{DateTime, Local};

fn format_time(time: DateTime<Local>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}
```

- [ ] **Step 4: Run the formatter test and verify GREEN**

Run:

```bash
cargo test formats_time_for_the_launcher_label
```

Expected: PASS.

- [ ] **Step 5: Commit the formatter**

```bash
git add src/app/input.rs
git commit -m "test: define launcher time format"
```

### Task 3: Render and Refresh the Live Time

**Files:**
- Modify: `src/app/input.rs`

- [ ] **Step 1: Add time state and initialize it**

Add these fields to `Root`:

```rust
time: SharedString,
_update_time_task: Task<()>,
```

In `Root::new`, initialize the label synchronously and spawn the refresh loop:

```rust
let time = format_time(Local::now()).into();

Self {
    window_handle,
    query,
    state,
    list_state,
    time,
    _update_time_task: cx.spawn(Self::do_update_time_task),
    _update_list_task: Some(cx.spawn(Self::do_update_list_task)),
    _open_application_task: None,
}
```

- [ ] **Step 2: Implement the one-second refresh loop**

Import `std::time::Duration` and add this method to `impl Root`:

```rust
async fn do_update_time_task(self_weak_entity: WeakEntity<Self>, cx: &mut AsyncApp) {
    loop {
        cx.background_executor().timer(Duration::from_secs(1)).await;

        if self_weak_entity
            .update(cx, |root, cx| {
                root.time = format_time(Local::now()).into();
                cx.notify();
            })
            .is_err()
        {
            break;
        }
    }
}
```

- [ ] **Step 3: Render the label above the input**

Increase the base launcher height from `48` to `75` and replace the input wrapper with:

```rust
.child(
    div()
        .p_2()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .px_1()
                .text_sm()
                .text_color(rgb(0x929292))
                .child(self.time.clone()),
        )
        .child(
            Input::new(&self.query)
                .border_1()
                .border_color(rgb(0x3a3a3a))
                .bg(rgba(0x1E1E1EFF)),
        ),
)
```

- [ ] **Step 4: Compile and run all tests**

Run:

```bash
cargo test
```

Expected: compilation succeeds and all tests PASS. If the GPUI timer API differs in the pinned dependency revision, adjust only the timer call to the compiler-confirmed API while preserving the one-second weak-entity loop.

- [ ] **Step 5: Format and verify formatting**

Run:

```bash
cargo fmt
cargo fmt --check
```

Expected: both commands succeed, and `cargo fmt --check` has no output.

- [ ] **Step 6: Commit the live UI**

```bash
git add src/app/input.rs
git commit -m "feat: show live time above query input"
```

### Task 4: Final Verification

**Files:**
- Verify: `src/app/input.rs`
- Verify: `src/app/query_provider/mod.rs`
- Verify deletion: `src/app/query_provider/time.rs`
- Verify: `src/app/query_parser.rs`

- [ ] **Step 1: Confirm the removed provider has no remaining references**

Run:

```bash
rg -n "TimeQueryProvider|mod time|trigger\(.*time|\"time\"" src/app
```

Expected: no references related to the removed provider. The command may exit with status 1 because no matches are found.

- [ ] **Step 2: Run the complete verification suite**

Run:

```bash
cargo fmt --check
cargo test
git diff --check
```

Expected: all commands succeed; all tests pass; no formatting or whitespace errors appear.

- [ ] **Step 3: Inspect repository state**

Run:

```bash
git status --short
git log -4 --oneline
```

Expected: implementation files are committed. Browser-companion files under `.superpowers/` may remain untracked and must not be committed.
