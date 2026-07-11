use crate::app::query_parser::{QueryParserItem, QueryProvider};
use chrono::Local;
use std::path::PathBuf;

pub(super) struct TimeQueryProvider;

impl QueryProvider for TimeQueryProvider {
    fn trigger(&self) -> &'static str {
        "time"
    }

    fn parse(&self, _args: &[String]) -> Vec<QueryParserItem> {
        vec![QueryParserItem {
            title: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            subtitle: String::new(),
            action: String::new(),
            icon: PathBuf::new(),
        }]
    }
}
