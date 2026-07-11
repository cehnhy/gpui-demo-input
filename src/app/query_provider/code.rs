use super::query_arg;
use crate::app::query_parser::{QueryParserItem, QueryProvider};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
struct SearchSpec {
    root: PathBuf,
    max_depth: Option<usize>,
}

fn search_specs(home: &Path) -> Vec<SearchSpec> {
    vec![
        SearchSpec {
            root: home.join("repo"),
            max_depth: Some(2),
        },
        SearchSpec {
            root: home.to_path_buf(),
            max_depth: Some(1),
        },
    ]
}

fn merge_paths(search_results: impl IntoIterator<Item = Vec<PathBuf>>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();

    for search_result in search_results {
        for path in search_result {
            if seen.insert(path.clone()) {
                paths.push(path);
                if paths.len() == 6 {
                    return paths;
                }
            }
        }
    }

    paths
}

fn run_search(spec: &SearchSpec, query: &str) -> Vec<PathBuf> {
    let mut command = std::process::Command::new("fd");
    command.arg("--max-results").arg("6").arg("--type").arg("d");

    if let Some(max_depth) = spec.max_depth {
        command.arg("--max-depth").arg(max_depth.to_string());
    }

    let Ok(output) = command.arg(query).arg(&spec.root).output() else {
        return vec![];
    };
    if !output.status.success() {
        return vec![];
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

pub(super) struct CodeQueryProvider;

impl QueryProvider for CodeQueryProvider {
    fn trigger(&self) -> &'static str {
        "code"
    }

    fn parse(&self, args: &[String]) -> Vec<QueryParserItem> {
        let arg = query_arg(args);
        let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) else {
            return vec![];
        };

        merge_paths(
            search_specs(Path::new(&home))
                .iter()
                .map(|spec| run_search(spec, arg)),
        )
        .into_iter()
        .map(|path| {
            let path_text = path.to_string_lossy();
            let title = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&path_text)
                .to_string();
            let subtitle = path
                .parent()
                .and_then(|parent| parent.to_str())
                .unwrap_or("")
                .to_string();

            QueryParserItem {
                title,
                subtitle,
                action: format!("code {}", path_text),
                icon: PathBuf::new(),
            }
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn builds_repo_and_home_search_specs() {
        assert_eq!(
            search_specs(Path::new("/home/test")),
            vec![
                SearchSpec {
                    root: PathBuf::from("/home/test/repo"),
                    max_depth: Some(2),
                },
                SearchSpec {
                    root: PathBuf::from("/home/test"),
                    max_depth: Some(1),
                },
            ]
        );
    }

    #[test]
    fn merges_paths_in_order_without_duplicates() {
        let paths = merge_paths([
            vec![
                PathBuf::from("/home/test/repo/a"),
                PathBuf::from("/home/test/repo/b"),
            ],
            vec![
                PathBuf::from("/home/test/repo/a"),
                PathBuf::from("/home/test/nixos-config"),
            ],
        ]);

        assert_eq!(
            paths,
            vec![
                PathBuf::from("/home/test/repo/a"),
                PathBuf::from("/home/test/repo/b"),
                PathBuf::from("/home/test/nixos-config"),
            ]
        );
    }

    #[test]
    fn limits_merged_paths_to_six() {
        let paths = merge_paths([(0..8)
            .map(|index| PathBuf::from(index.to_string()))
            .collect::<Vec<_>>()]);

        assert_eq!(paths.len(), 6);
    }
}
