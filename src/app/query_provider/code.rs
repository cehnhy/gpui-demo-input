use super::query_arg;
use crate::app::query_parser::{QueryParserItem, QueryProvider};
use std::path::PathBuf;

pub(super) struct CodeQueryProvider;

impl QueryProvider for CodeQueryProvider {
    fn trigger(&self) -> &'static str {
        "code"
    }

    fn parse(&self, args: &[String]) -> Vec<QueryParserItem> {
        let arg = query_arg(args);

        let output = std::process::Command::new("fd")
            .arg("--max-results")
            .arg("6")
            .arg("-d")
            .arg("2")
            .arg("-t")
            .arg("d")
            .arg(arg)
            .arg(format!(
                "{}/repo",
                std::env::var("HOME").unwrap_or_default()
            ))
            .output();

        let mut items = vec![];
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().filter(|line| !line.is_empty()) {
                    let path = PathBuf::from(line);
                    let title = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(line)
                        .to_string();
                    let subtitle = path
                        .parent()
                        .and_then(|parent| parent.to_str())
                        .unwrap_or("")
                        .to_string();

                    items.push(QueryParserItem {
                        title,
                        subtitle,
                        action: format!("code {}", line),
                        icon: PathBuf::new(),
                    });
                }
            }
        }

        items
    }
}
