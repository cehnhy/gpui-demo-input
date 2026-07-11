use chrono::Local;
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::collections::HashSet;
use std::path::PathBuf;

pub fn to_query_parser(query: String) -> LegacyQueryParser {
    let mut query = query;

    let exist_triggers = vec!["code", "float", "time", "c"];
    if !exist_triggers
        .iter()
        .any(|&p| query.starts_with(&format!("{} ", p)))
    {
        query = format!("application {}", query);
    }

    let mut args: Vec<String> = query.split(' ').map(|s| s.to_string()).collect();
    return LegacyQueryParser {
        trigger: args.remove(0),
        args,
    };
}

pub struct LegacyQueryParser {
    trigger: String, // application, code, float, time, clip
    args: Vec<String>,
}

pub struct QueryParserItem {
    pub title: String,
    pub subtitle: String,
    pub action: String,
    pub icon: PathBuf,
}

pub trait QueryParser {
    fn parse(&self, query: &str) -> Vec<QueryParserItem>;
}

pub trait QueryProvider {
    fn trigger(&self) -> &'static str;
    fn parse(&self, args: &[String]) -> Vec<QueryParserItem>;
}

pub struct DefaultQueryParser {
    providers: Vec<Box<dyn QueryProvider>>,
}

impl DefaultQueryParser {
    pub fn new(providers: Vec<Box<dyn QueryProvider>>) -> Self {
        Self { providers }
    }
}

impl QueryParser for DefaultQueryParser {
    fn parse(&self, query: &str) -> Vec<QueryParserItem> {
        let tokens = query
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let explicit = tokens
            .first()
            .and_then(|trigger| self.providers.iter().find(|p| p.trigger() == trigger));

        if let Some(provider) = explicit {
            return provider.parse(&tokens[1..]);
        }

        self.providers
            .iter()
            .find(|provider| provider.trigger() == "application")
            .map(|provider| provider.parse(&tokens))
            .unwrap_or_default()
    }
}

impl LegacyQueryParser {
    pub fn parse(&self) -> Vec<QueryParserItem> {
        match self.trigger.as_str() {
            "application" => self.parse_application(),
            "code" => self.parse_code(),
            "float" => self.parse_float(),
            "time" => self.parse_time(),
            "c" => self.parse_clip(),
            _ => vec![],
        }
    }

    fn parse_application(&self) -> Vec<QueryParserItem> {
        let arg = self.args[0].clone();
        let mut items = vec![];
        let mut seen_ids = HashSet::new();

        let locales = get_languages_from_env();

        // Add NixOS-specific paths to the default paths
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

            // Get desktop file ID to deduplicate
            let desktop_id = entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if !seen_ids.insert(desktop_id.to_string()) {
                // Already seen this desktop file, skip it
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
                .map(|c| c.to_string())
                .unwrap_or_else(|| "".to_string());

            let action = match entry.exec() {
                Some(exec) => exec.to_string(),
                None => continue,
            };

            let mut icon = PathBuf::from("");
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

    fn parse_code(&self) -> Vec<QueryParserItem> {
        if self.args.is_empty() {
            return vec![];
        }
        let arg = self.args[0].clone();

        let output = std::process::Command::new("fd")
            .arg("--max-results")
            .arg("6")
            .arg("-d")
            .arg("2")
            .arg("-t")
            .arg("d")
            .arg(&arg)
            .arg(format!(
                "{}/repo",
                std::env::var("HOME").unwrap_or_default()
            ))
            .output();

        let mut items = vec![];

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    let path = PathBuf::from(line);
                    let title = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(line)
                        .to_string();

                    let subtitle = path
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or("")
                        .to_string();

                    items.push(QueryParserItem {
                        title,
                        subtitle,
                        action: format!("code {}", line),
                        icon: PathBuf::from(""),
                    });
                }
            }
        }

        items
    }

    fn parse_float(&self) -> Vec<QueryParserItem> {
        return vec![QueryParserItem {
            title: "toggle floating".to_string(),
            subtitle: "".to_string(),
            action: "niri msg action toggle-window-floating".to_string(),
            icon: PathBuf::from(""),
        }];
    }

    fn parse_time(&self) -> Vec<QueryParserItem> {
        return vec![QueryParserItem {
            title: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            subtitle: "".to_string(),
            action: "".to_string(),
            icon: PathBuf::from(""),
        }];
    }

    fn parse_clip(&self) -> Vec<QueryParserItem> {
        if self.args.is_empty() {
            return vec![];
        }
        let arg = self.args[0].clone();

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
                for line in stdout.lines() {
                    if line.is_empty() {
                        continue;
                    }

                    let content = line.splitn(2, '\t').nth(1).unwrap_or(line);

                    let title = if content.chars().count() > 60 {
                        let truncated: String = content.chars().take(60).collect();
                        format!("{}...", truncated)
                    } else {
                        content.to_string()
                    };

                    items.push(QueryParserItem {
                        title,
                        subtitle: "".to_string(),
                        action: format!("wl-copy {}", content),
                        icon: PathBuf::from(""),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct FakeProvider {
        trigger: &'static str,
        calls: Rc<RefCell<Vec<Vec<String>>>>,
    }

    impl FakeProvider {
        fn new(trigger: &'static str, calls: Rc<RefCell<Vec<Vec<String>>>>) -> Self {
            Self { trigger, calls }
        }
    }

    impl QueryProvider for FakeProvider {
        fn trigger(&self) -> &'static str {
            self.trigger
        }

        fn parse(&self, args: &[String]) -> Vec<QueryParserItem> {
            self.calls.borrow_mut().push(args.to_vec());
            vec![QueryParserItem {
                title: self.trigger.to_string(),
                subtitle: String::new(),
                action: String::new(),
                icon: PathBuf::new(),
            }]
        }
    }

    #[test]
    fn routes_explicit_trigger_to_matching_provider() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let parser = DefaultQueryParser::new(vec![Box::new(FakeProvider::new(
            "code",
            Rc::clone(&calls),
        ))]);

        let items = parser.parse("code gpui demo");

        assert_eq!(items[0].title, "code");
        assert_eq!(
            calls.borrow().as_slice(),
            &[vec!["gpui".to_string(), "demo".to_string()]]
        );
    }

    #[test]
    fn routes_unprefixed_query_to_application_provider() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let parser = DefaultQueryParser::new(vec![Box::new(FakeProvider::new(
            "application",
            Rc::clone(&calls),
        ))]);

        parser.parse("visual studio code");

        assert_eq!(
            calls.borrow().as_slice(),
            &[vec![
                "visual".to_string(),
                "studio".to_string(),
                "code".to_string()
            ]]
        );
    }

    #[test]
    fn handles_empty_query_without_panicking() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let parser = DefaultQueryParser::new(vec![Box::new(FakeProvider::new(
            "application",
            Rc::clone(&calls),
        ))]);

        parser.parse("");

        assert_eq!(calls.borrow().as_slice(), &[Vec::<String>::new()]);
    }

    #[test]
    fn returns_no_items_without_matching_or_fallback_provider() {
        let parser = DefaultQueryParser::new(Vec::new());

        assert!(parser.parse("unknown query").is_empty());
    }
}
