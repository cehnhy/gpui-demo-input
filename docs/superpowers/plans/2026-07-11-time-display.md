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

### Task 5: Separate the Time Label from the Query Panel

**Files:**
- Modify: `src/app/input.rs`

- [ ] **Step 1: Restore the query/list panel's own base height**

Change the base panel height from `75` back to `48`. The standalone time label belongs to the transparent outer wrapper and must not be counted as part of the dark query/list panel:

```rust
let mut h = 48;
```

- [ ] **Step 2: Introduce a transparent outer wrapper**

Keep the existing event handlers on a new 800-pixel-wide vertical wrapper. Render the time label first, then the dark rounded query/list panel as its sibling:

```rust
.child(
    div()
        .key_context(CONTEXT)
        .on_action(cx.listener(Self::select_last_item))
        .on_action(cx.listener(Self::select_next_item))
        .on_action(cx.listener(Self::cancel))
        .w(px(800.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .px_3()
                .text_sm()
                .text_color(rgb(0x929292))
                .child(self.time.clone()),
        )
        .child(
            div()
                .h(px(h as f32))
                .flex()
                .flex_col()
                .bg(rgba(0x1E1E1EFF))
                .rounded(px(10.0))
                .overflow_hidden()
                .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                    cx.stop_propagation();
                })
                .child(
                    div().p_2().child(
                        Input::new(&self.query)
                            .border_1()
                            .border_color(rgb(0x3a3a3a))
                            .bg(rgba(0x1E1E1EFF)),
                    ),
                )
                .child(
                    div().flex_1().pb_2().px_2().child(
                        list(
                            self.list_state.clone(),
                            cx.processor(move |root, idx, _window, app| {
                                let state = root.state.read(app);
                                let item: &ListItem = state.items.get(idx).unwrap();
                                let mut item = item.clone();
                                item.set_index(idx);
                                if idx == state.selected_id {
                                    item.select();
                                }
                                let root_entity = root_entity.clone();

                                div()
                                    .id(("list-item-click", idx))
                                    .on_click(move |_event, _window, cx| {
                                        cx.stop_propagation();
                                        root_entity
                                            .update(cx, |root, cx| root.open_item(idx, cx))
                                            .ok();
                                    })
                                    .child(item)
                                    .into_any_element()
                            }),
                        )
                        .size_full(),
                    ),
                ),
        ),
)
```

- [ ] **Step 3: Compile and run the complete test suite**

Run:

```bash
cargo test
```

Expected: compilation succeeds and all 11 tests PASS. This layout-only change has no stable unit-test seam in the current GPUI render tree, so compilation plus visual inspection verifies element composition while the existing formatter test continues to cover time content.

- [ ] **Step 4: Format and inspect the render tree**

Run:

```bash
cargo fmt
cargo fmt --check
sed -n '145,245p' src/app/input.rs
```

Expected: the time label is a sibling immediately before the dark rounded panel; only the inner panel has `.bg(...)`, `.rounded(...)`, and `.overflow_hidden()`.

- [ ] **Step 5: Commit the standalone layout**

```bash
git add src/app/input.rs
git commit -m "style: separate time from query panel"
```

- [ ] **Step 6: Run final verification**

Run:

```bash
cargo fmt --check
cargo test
git diff --check
git status --short
```

Expected: formatting and whitespace checks succeed, all 11 tests PASS, and only `.superpowers/` may remain untracked.
