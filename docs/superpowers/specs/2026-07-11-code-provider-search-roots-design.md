# Code Provider Search Roots Design

## Goal

Expand the `code` query provider so it can find deeply nested projects under
`$HOME/repo` and any first-level directory under `$HOME`.

## Search Behavior

The provider performs two ordered searches with the same query:

1. Recursively search all directories below `$HOME/repo` without a depth limit.
2. Search directories directly below `$HOME` with a maximum depth of one.

Results from the repository search come first. Results are deduplicated by full
path because `$HOME/repo` can also appear in the home-directory search. The
combined result list is capped at six items, preserving the current UI limit.

Each search is independent. If one `fd` invocation fails, successful results
from the other search are still returned. Missing or empty `HOME` produces no
results rather than searching relative paths.

## Result Formatting

Existing formatting remains unchanged:

- title: directory basename;
- subtitle: parent directory;
- action: `code <full-path>`;
- icon: empty path.

## Testing

Unit tests cover search specification construction and result merging without
requiring the external `fd` command. They verify root order, depth settings,
deduplication, and the six-item combined limit.

## Scope

This change does not alter the `code` trigger, parser routing, command launch
behavior, or other providers.
