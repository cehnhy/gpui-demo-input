# Query Input Time Display Design

## Goal

Remove the `time` query provider and show the current local time above the query input. The displayed time refreshes once per second.

## Scope

- Delete the dedicated `time` query provider.
- Remove `time` from the default provider registry and its provider-list expectation.
- Add a live time label above the query input.
- Preserve all existing query, result-list, keyboard, mouse, and launch behavior.

## UI Design

The time uses the format `YYYY-MM-DD HH:MM:SS`. It is rendered as small, muted, left-aligned text above the input and aligned with the input's content area. The launcher height increases enough to accommodate the label without reducing the visible result-list area.

## Implementation

`Root` owns the formatted time string and a long-lived GPUI task. The task waits for one second, upgrades the weak `Root` entity, updates the string from `chrono::Local::now()`, and calls `cx.notify()` to redraw the window. The first value is initialized synchronously so the label is populated as soon as the window opens.

Time formatting is isolated in a small helper so its output can be tested without running a GPUI window.

The provider registry no longer declares, imports, or constructs `TimeQueryProvider`, and `src/app/query_provider/time.rs` is deleted.

## Failure and Lifecycle Behavior

The refresh task uses a weak entity reference and exits when the window's root entity no longer exists. It does not keep the window alive and does not treat window closure as an error.

## Tests and Verification

- Add a unit test for the time formatting helper.
- Update the default-provider trigger test to expect `application`, `code`, `float`, and `c`, with no `time` trigger.
- Run the relevant tests, then the complete test suite and `cargo fmt --check`.
