use crate::app::query_parser::{QueryParserItem, QueryProvider};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) struct ApplicationQueryProvider;

impl QueryProvider for ApplicationQueryProvider {
    fn trigger(&self) -> &'static str {
        "application"
    }

    fn parse(&self, args: &[String]) -> Vec<QueryParserItem> {
        let arg = args.join(" ");
        let mut items = vec![];
        let mut seen_ids = HashSet::new();
        let locales = get_languages_from_env();

        let mut paths: Vec<PathBuf> = default_paths().collect();
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(format!(
                "{}/.nix-profile/share/applications",
                home
            )));
        }

        let entries = Iter::new(paths.into_iter())
            .entries(Some(&locales))
            .collect::<Vec<_>>();
        for entry in entries {
            if entry.no_display() {
                continue;
            }

            let desktop_id = entry
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !seen_ids.insert(desktop_id.to_string()) {
                continue;
            }

            let name = match entry.name(&locales) {
                Some(name) => name.to_string(),
                None => continue,
            };
            if !name.to_lowercase().contains(&arg.to_lowercase()) {
                continue;
            }

            let comment = entry
                .comment(&locales)
                .map(|comment| comment.to_string())
                .unwrap_or_default();
            let action = match entry.exec() {
                Some(exec) => exec.to_string(),
                None => continue,
            };

            let mut icon = PathBuf::new();
            if let Some(icon_path) = entry.icon() {
                if let Some(found_icon) = freedesktop_icons::lookup(&icon_path)
                    .with_cache()
                    .with_size(48)
                    .find()
                {
                    icon = found_icon;
                }
            }

            items.push(QueryParserItem {
                title: name,
                subtitle: comment,
                action,
                icon,
            });
        }

        items
    }
}
