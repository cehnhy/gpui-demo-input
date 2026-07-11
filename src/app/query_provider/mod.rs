mod application;
mod clipboard;
mod code;
mod float;

use super::query_parser::QueryProvider;
use application::ApplicationQueryProvider;
use clipboard::ClipboardQueryProvider;
use code::CodeQueryProvider;
use float::FloatQueryProvider;

fn query_arg(args: &[String]) -> &str {
    args.first().map(String::as_str).unwrap_or_default()
}

pub(super) fn default_providers() -> Vec<Box<dyn QueryProvider>> {
    vec![
        Box::new(ApplicationQueryProvider),
        Box::new(CodeQueryProvider),
        Box::new(FloatQueryProvider),
        Box::new(ClipboardQueryProvider),
    ]
}

#[cfg(test)]
mod tests {
    use super::query_arg;

    #[test]
    fn missing_provider_argument_is_an_empty_query() {
        assert_eq!(query_arg(&[]), "");
    }

    #[test]
    fn provider_argument_uses_the_first_value() {
        assert_eq!(query_arg(&["gpui".to_string(), "demo".to_string()]), "gpui");
    }
}
