#[cfg(any(feature = "tui", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BrowserPane {
    Items,
    Detail,
}

#[cfg(any(feature = "tui", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchDirection {
    Forward,
    Backward,
}

#[cfg(any(feature = "tui", test))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SearchState {
    pub(crate) direction: SearchDirection,
    pub(crate) query: String,
    pub(crate) filter_kind: String,
    pub(crate) match_ordinal: Option<usize>,
    pub(crate) match_count: usize,
}

#[cfg_attr(not(feature = "tui"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BrowserItem {
    pub(crate) kind: String,
    pub(crate) title: String,
    pub(crate) meta: String,
    pub(crate) details: Vec<String>,
}

#[cfg(any(feature = "tui", test))]
impl BrowserItem {
    pub(crate) fn matches_query(&self, query: &str) -> bool {
        let needle = query.trim().to_ascii_lowercase();
        if needle.is_empty() {
            return false;
        }
        self.kind.to_ascii_lowercase().contains(&needle)
            || self.title.to_ascii_lowercase().contains(&needle)
            || self.meta.to_ascii_lowercase().contains(&needle)
            || self
                .details
                .iter()
                .any(|line| line.to_ascii_lowercase().contains(&needle))
    }
}
