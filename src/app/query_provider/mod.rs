mod application;
mod clipboard;
mod code;
mod float;
mod time;

use super::query_parser::QueryProvider;
use application::ApplicationQueryProvider;
use clipboard::ClipboardQueryProvider;
use code::CodeQueryProvider;
use float::FloatQueryProvider;
use time::TimeQueryProvider;

pub(super) fn default_providers() -> Vec<Box<dyn QueryProvider>> {
    vec![
        Box::new(ApplicationQueryProvider),
        Box::new(CodeQueryProvider),
        Box::new(FloatQueryProvider),
        Box::new(TimeQueryProvider),
        Box::new(ClipboardQueryProvider),
    ]
}
