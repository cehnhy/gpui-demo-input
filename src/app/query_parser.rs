use chrono::Local;
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::collections::HashSet;
use std::path::PathBuf;

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

impl Default for DefaultQueryParser {
    fn default() -> Self {
        Self::new(vec![
            Box::new(ApplicationQueryProvider),
            Box::new(CodeQueryProvider),
            Box::new(FloatQueryProvider),
            Box::new(TimeQueryProvider),
            Box::new(ClipboardQueryProvider),
        ])
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

struct ApplicationQueryProvider;

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

struct CodeQueryProvider;

impl QueryProvider for CodeQueryProvider {
    fn trigger(&self) -> &'static str {
        "code"
    }

    fn parse(&self, args: &[String]) -> Vec<QueryParserItem> {
        let Some(arg) = args.first() else {
            return vec![];
        };

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

struct FloatQueryProvider;

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

struct TimeQueryProvider;

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

struct ClipboardQueryProvider;

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
        let parser =
            DefaultQueryParser::new(vec![Box::new(FakeProvider::new("code", Rc::clone(&calls)))]);

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

    #[test]
    fn default_parser_registers_existing_triggers() {
        let parser = DefaultQueryParser::default();
        let triggers = parser
            .providers
            .iter()
            .map(|provider| provider.trigger())
            .collect::<Vec<_>>();

        assert_eq!(triggers, vec!["application", "code", "float", "time", "c"]);
    }
}
