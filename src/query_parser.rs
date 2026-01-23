use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use std::path::PathBuf;

pub fn to_query_parser(query: String) -> QueryParser {
    let mut query_parser = QueryParser {
        trigger: "application".to_string(),
        args: vec!["".to_string()],
    };

    if query.is_empty() {
        return query_parser;
    }

    let exist_triggers = vec!["code", "float", "time"];
    let mut args: Vec<String> = query.split(' ').map(|s| s.to_string()).collect();
    if !exist_triggers.contains(&args[0].as_str()) {
        args.insert(0, "application".to_string());
    }
    query_parser.trigger = args[0].clone();
    query_parser.args = args[1..].to_vec();

    return query_parser;
}

pub struct QueryParser {
    trigger: String, // application, code, float, time
    args: Vec<String>,
}

pub struct QueryParserItem {
    pub title: String,
    pub subtitle: String,
    pub action: String,
    pub icon: PathBuf,
}

impl QueryParser {
    pub fn parse(&self) -> Vec<QueryParserItem> {
        match self.trigger.as_str() {
            "application" => self.parse_application(),
            "code" => {
                vec![]
            }
            "float" => {
                vec![]
            }
            "time" => self.parse_time(),
            _ => {
                vec![]
            }
        }
    }

    fn parse_application(&self) -> Vec<QueryParserItem> {
        let arg = self.args[0].clone();
        let mut items = vec![];

        let locales = get_languages_from_env();
        let entries = Iter::new(default_paths())
            .entries(Some(&locales))
            .collect::<Vec<_>>();
        for entry in entries {
            if entry.no_display() {
                continue;
            }

            let name = match entry.name(&locales) {
                Some(name) => name.to_string(),
                None => continue,
            };

            if !name.to_lowercase().contains(&arg.to_lowercase()) {
                continue;
            }

            let comment = match entry.comment(&locales) {
                Some(comment) => comment.to_string(),
                None => continue,
            };

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

    fn parse_time(&self) -> Vec<QueryParserItem> {
        return vec![QueryParserItem {
            title: "1".to_string(),
            subtitle: "".to_string(),
            action: "".to_string(),
            icon: PathBuf::from(""),
        }];
    }
}
