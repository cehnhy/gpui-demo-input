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
