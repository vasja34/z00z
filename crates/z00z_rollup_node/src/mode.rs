#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeMode {
    Aggregator,
    Validator,
    Watcher,
    Combined,
    ApiOnly,
}
