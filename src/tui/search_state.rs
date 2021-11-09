use {
    super::ContentSkin,
    crate::{
        core::*,
        search::*,
    },
    termimad::InputField,
};

/// State of the search in a drawer
pub struct SearchState {
    pub input: InputField,
    pub result: Option<SearchResult>,
}

pub struct SearchResult {
    /// filtered entries
    pub entries: Vec<MatchingEntry>,
    /// index among filtered entries of the one with the best score
    pub best_line: Option<usize>,
}

pub struct MatchingEntry {
    pub idx: usize,
    pub name_match: NameMatch,
}

impl Default for SearchState {
    fn default() -> Self {
        let input = ContentSkin::make_input();
        Self {
            input,
            result: None,
        }
    }
}

impl SearchState {
    pub fn set_best_line(&mut self, best_line: usize) {
        if let Some(result) = self.result.as_mut() {
            result.best_line = Some(best_line);
        }
    }
    /// recompute the result from the input content
    pub fn update(&mut self, drawer: &OpenDrawer){
        if self.input.is_empty() {
            self.result = None;
        } else {
            let pattern = FuzzyPattern::from(&self.input.get_content());
            let mut entries: Vec<MatchingEntry> = Vec::new();
            let mut best_line: Option<usize> = None;
            for (idx, entry) in drawer.content.entries.iter().enumerate() {
                if let Some(name_match) = pattern.find(&entry.name) {
                    if let Some(bl) = best_line {
                        if entries[bl].name_match.score < name_match.score {
                            best_line = Some(entries.len());
                        }
                    } else {
                        best_line = Some(entries.len());
                    }
                    entries.push(MatchingEntry { idx, name_match });
                }
            }
            debug!("{} matching entries", entries.len());
            self.result = Some(SearchResult { entries, best_line });
        }
    }
    pub fn has_content(&self) -> bool {
        !self.input.is_empty()
    }
    /// clear the search box
    pub fn clear(&mut self) {
        self.input.clear();
        self.result = None;
    }
}
