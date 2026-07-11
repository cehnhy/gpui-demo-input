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
        Self::new(super::query_provider::default_providers())
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

        assert_eq!(triggers, vec!["application", "code", "float", "c"]);
    }
}
