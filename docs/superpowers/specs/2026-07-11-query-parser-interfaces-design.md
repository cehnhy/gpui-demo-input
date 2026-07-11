# Query Parser Interfaces Design

## Goal

Separate query parsing and trigger routing from the implementations that produce
results. Consumers should depend on a parser interface, while each supported
trigger is implemented by an independently testable provider.

The refactor must preserve the current application, code, floating-window,
time, and clipboard behavior.

## Public Interfaces

`QueryParser` is the consumer-facing interface:

```rust
pub trait QueryParser {
    fn parse(&self, query: &str) -> Vec<QueryParserItem>;
}
```

`QueryProvider` is the extension interface for one trigger:

```rust
pub trait QueryProvider {
    fn trigger(&self) -> &'static str;
    fn parse(&self, args: &[String]) -> Vec<QueryParserItem>;
}
```

Both traits remain synchronous because all existing providers perform
synchronous work and the current UI call site expects immediate results.

## Default Implementation

`DefaultQueryParser` owns an ordered collection of boxed providers. Its default
constructor registers providers for these triggers:

- `application`
- `code`
- `float`
- `time`
- `c`

The parser splits the input into a trigger and arguments. If the first token
matches a registered provider, that provider receives the remaining tokens.
Otherwise, the entire input is passed to the `application` provider. This makes
the fallback depend on registered providers instead of a duplicated hard-coded
trigger list.

An injectable constructor accepts a provider collection so routing can be
tested without invoking desktop entry discovery or external commands.

## Providers

Each existing query implementation moves into a dedicated provider type:

- `ApplicationQueryProvider`
- `CodeQueryProvider`
- `FloatQueryProvider`
- `TimeQueryProvider`
- `ClipboardQueryProvider`

Providers own only the logic needed for their trigger. They return an empty
vector when required arguments are absent or an external command fails,
matching existing behavior. `QueryParserItem` remains the shared result model.

The initial refactor keeps these types in `query_parser.rs` to avoid unnecessary
module churn. They can move into separate modules later if their implementations
grow.

## Consumer Integration

The input component creates a default parser and calls the `QueryParser` trait
method with the current query. The old `to_query_parser` function and concrete
stateful parser are removed because query text is now method input rather than
parser state.

## Testing

Unit tests use small fake providers to verify:

- explicit triggers route to the matching provider;
- unprefixed text routes to `application` with the full query as arguments;
- arguments are split consistently;
- an empty query is handled without panicking;
- a parser without a matching or fallback provider returns no items.

Provider behavior that depends on the desktop environment or external programs
is left unchanged and is not exercised by routing unit tests.

## Scope

This change does not introduce asynchronous parsing, provider configuration,
dynamic plugin loading, richer error reporting, or changes to command execution.
Those concerns are independent of the interface extraction.
