# Query Provider Modules Design

## Goal

Move each built-in `QueryProvider` implementation out of `query_parser.rs` so
the parser focuses on interfaces and routing while provider-specific system
integration remains independently maintainable.

## Module Structure

```text
src/app/
├── query_parser.rs
└── query_provider/
    ├── mod.rs
    ├── application.rs
    ├── clipboard.rs
    ├── code.rs
    ├── float.rs
    └── time.rs
```

`app::query_provider` is a private module. Its concrete provider types use
`pub(crate)` visibility only where required for default parser construction.
This keeps `QueryParser`, `QueryProvider`, `QueryParserItem`, and
`DefaultQueryParser` as the public extension and consumption surface.

## Responsibilities

`query_parser.rs` retains:

- the shared query result model;
- the consumer-facing `QueryParser` trait;
- the provider-facing `QueryProvider` trait;
- `DefaultQueryParser` routing;
- routing and default-registration tests.

Each provider file contains one provider type and its `QueryProvider`
implementation. Provider behavior, external commands, trigger values, and
result formatting remain unchanged.

`query_provider/mod.rs` declares the provider modules and exposes a
`default_providers()` function returning the providers in the existing order:
`application`, `code`, `float`, `time`, and `c`.

## Dependency Direction

Provider modules import `QueryProvider` and `QueryParserItem` from
`app::query_parser`. `query_parser.rs` imports only the provider module's
`default_providers()` factory. This creates module references in both
directions, which Rust permits because the items do not form recursive types or
initialization cycles.

## Testing

Existing fake-provider routing tests stay in `query_parser.rs`. The default
registration test continues to verify the complete trigger order through the
parser's provider collection. Full-target tests and compilation confirm that
all extracted modules remain wired correctly.

No environment-dependent provider integration tests are added as part of this
file-organization refactor.

## Scope

This change does not alter provider behavior, public trait signatures, trigger
names, error handling, synchronous execution, or query result data.
