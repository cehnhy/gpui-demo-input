use crate::app::query_parser::{QueryParserItem, QueryProvider};
use std::path::PathBuf;

pub(super) struct ClipboardQueryProvider;

impl QueryProvider for ClipboardQueryProvider {
    fn trigger(&self) -> &'static str {
        "c"
    }

    fn parse(&self, args: &[String]) -> Vec<QueryParserItem> {
        let Some(arg) = args.first() else {
            return vec![];
        };

        let output = if arg.is_empty() {
            std::process::Command::new("cliphist").arg("list").output()
        } else {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("cliphist list | grep -iF -- '{}'", arg))
                .output()
        };

        let mut items = vec![];
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().filter(|line| !line.is_empty()) {
                    let content = line.split_once('\t').map_or(line, |(_, content)| content);
                    let title = if content.chars().count() > 60 {
                        format!("{}...", content.chars().take(60).collect::<String>())
                    } else {
                        content.to_string()
                    };

                    items.push(QueryParserItem {
                        title,
                        subtitle: String::new(),
                        action: format!("wl-copy {}", content),
                        icon: PathBuf::new(),
                    });

                    if items.len() >= 100 {
                        break;
                    }
                }
            }
        }

        items
    }
}
