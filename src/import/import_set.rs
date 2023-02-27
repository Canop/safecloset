use crate::core::*;

/// The subset of the drawer or csv file with the things to import
#[derive(Debug, Default)]
pub struct ImportSet {
    new_keys: Vec<Entry>,
    different_values: Vec<Entry>,
}

impl ImportSet {
    pub fn confirm_string(&self) -> String {
        format!(
            "The source contains {} new keys and {} new values.",
            self.new_keys.len(),
            self.different_values.len(),
        )
    }
}

impl ImportSet {
    pub fn new(
        mut src: Vec<Entry>,
        dst: &OpenDrawer,
    ) -> Self {
        let dst_entries = &dst.content.entries;
        let mut report = Self::default();
        for src_entry in src.drain(..) {
            let dst_entry = dst_entries.iter().find(|&se| se.name == src_entry.name);
            if let Some(dst_entry) = dst_entry {
                if !dst_entry.value.contains(&src_entry.value) {
                    report.different_values.push(src_entry);
                }
            } else {
                report.new_keys.push(src_entry);
            }
        }
        report
    }
    pub fn is_empty(&self) -> bool {
        self.new_keys.is_empty() && self.different_values.is_empty()
    }
    /// Import the set into the destination drawer and
    /// return a displayable report
    pub fn import_into(
        mut self,
        dst: &mut OpenDrawer,
    ) -> String {
        let report = format!(
            "{} added entries and {} enriched entries.\n\
            Nothing is saved on disk until you save.",
            self.new_keys.len(),
            self.different_values.len(),
        );
        let dst_entries = &mut dst.content.entries;
        for src_entry in self.different_values.drain(..) {
            for dst_entry in dst_entries.iter_mut() {
                if dst_entry.name == src_entry.name {
                    dst_entry.value.push_str("\n---\n");
                    dst_entry.value.push_str(&src_entry.value);
                    break;
                }
            }
        }
        dst_entries.append(&mut self.new_keys);
        report
    }
}
