mod engine;
#[cfg(test)]
mod tests;

pub use engine::{
    SearchError, SearchLimit, SearchLimits, SearchReport, SearchStatus, SearchUpdate,
    search_by_name,
};
