use {
    crate::{
        core::*,
        search::*,
    },
    termimad::{Area, InputField},
};

/// State of the search in a drawer
///
pub struct SearchState {
    pub input: InputField,
    pub result: Option<SearchResult>,
}

pub struct SearchResult {
    pattern: FuzzyPattern,
    pub entries: Vec<MatchingEntry>,
}

pub struct MatchingEntry {
    pub idx: usize,
    pub name_match: NameMatch,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            input: InputField::new(Area::uninitialized()),
            result: None,
        }
    }
}

impl SearchState {
    /// recompute the result from the input content
    pub fn update(&mut self, drawer: &OpenDrawer){
        if self.input.is_empty() {
            self.result = None;
            debug!("no more search");
        } else {
            let pattern = FuzzyPattern::from(&self.input.get_content());
            let mut entries = Vec::new();
            info!("searching on pattern {:?}", &pattern);
            for (idx, entry) in drawer.entries.iter().enumerate() {
                if let Some(name_match) = pattern.find(&entry.name) {
                    entries.push(MatchingEntry { idx, name_match });
                }
            }
            debug!("{} matching entries", entries.len());
            self.result = Some(SearchResult { pattern, entries });
        }
    }
    pub fn has_content(&self) -> bool {
        !self.input.is_empty()
    }
    /// clear the search box
    pub fn clear(&mut self) {
        self.input.set_content("");
        self.result = None;
    }
}
