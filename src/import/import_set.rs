use crate::core::*;

/// Separator between the successive values of an entry, rendered as a
/// horizontal rule when the values are displayed as markdown
const VALUES_SEPARATOR: &str = "\n---\n";

/// The successive values held by an entry
fn values(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(VALUES_SEPARATOR)
        .filter(|value| !value.is_empty())
}

/// Whether the destination already holds every value of this entry.
///
/// Comparing values and not whole strings is what makes an import both
/// idempotent and lossless: a value which is only a substring of another
/// one, like a password which was later extended, must still be imported.
fn holds_values(
    dst_value: &str,
    value: &str,
) -> bool {
    values(value).all(|value| values(dst_value).any(|dst_value| dst_value == value))
}

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
                if !holds_values(&dst_entry.value, &src_entry.value) {
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
                    let missing: Vec<&str> = values(&src_entry.value)
                        .filter(|value| !values(&dst_entry.value).any(|dst| dst == *value))
                        .collect();
                    for value in missing {
                        dst_entry.value.push_str(VALUES_SEPARATOR);
                        dst_entry.value.push_str(value);
                    }
                    break;
                }
            }
        }
        dst_entries.append(&mut self.new_keys);
        report
    }
}

#[cfg(test)]
mod import_set_tests {

    use super::*;

    /// A drawer holding the given entries, deep so that it comes with few
    /// decoy drawers
    fn test_drawer(entries: Vec<Entry>) -> OpenDrawer {
        let mut closet = Closet::new(3).unwrap();
        let mut drawer = closet
            .create_drawer(3, "test-import-passphrase".to_string())
            .unwrap();
        drawer.content.entries = entries;
        drawer
    }

    fn value_of(
        drawer: &OpenDrawer,
        name: &str,
    ) -> String {
        drawer
            .content
            .entries
            .iter()
            .find(|entry| entry.name == name)
            .unwrap()
            .value
            .clone()
    }

    /// check a value which is only a substring of the destination one is
    /// imported instead of being silently dropped
    #[test]
    fn test_import_shorter_value() {
        let mut drawer = test_drawer(vec![Entry::new("key", "hunter22")]);
        let import_set = ImportSet::new(vec![Entry::new("key", "hunter2")], &drawer);
        assert!(!import_set.is_empty());
        import_set.import_into(&mut drawer);
        assert_eq!(value_of(&drawer, "key"), "hunter22\n---\nhunter2");
    }

    /// check importing the same source twice doesn't add its values twice
    #[test]
    fn test_import_is_idempotent() {
        let src = vec![Entry::new("key", "new")];
        let mut drawer = test_drawer(vec![Entry::new("key", "old")]);
        let import_set = ImportSet::new(src.clone(), &drawer);
        import_set.import_into(&mut drawer);
        assert_eq!(value_of(&drawer, "key"), "old\n---\nnew");
        let import_set = ImportSet::new(src, &drawer);
        assert!(import_set.is_empty());
    }

    /// check an already present value isn't imported again
    #[test]
    fn test_import_same_value() {
        let drawer = test_drawer(vec![Entry::new("key", "value")]);
        let import_set = ImportSet::new(vec![Entry::new("key", "value")], &drawer);
        assert!(import_set.is_empty());
    }

    /// check a source value which itself holds several values is merged
    /// without duplicating the ones already there
    #[test]
    fn test_import_source_holding_several_values() {
        let src = vec![Entry::new("key", "b\n---\nc")];
        let mut drawer = test_drawer(vec![Entry::new("key", "a\n---\nb")]);
        let import_set = ImportSet::new(src.clone(), &drawer);
        import_set.import_into(&mut drawer);
        assert_eq!(value_of(&drawer, "key"), "a\n---\nb\n---\nc");
        let import_set = ImportSet::new(src, &drawer);
        assert!(import_set.is_empty());
    }

    /// check an empty value doesn't add a dangling separator
    #[test]
    fn test_import_empty_value() {
        let drawer = test_drawer(vec![Entry::new("key", "value")]);
        let import_set = ImportSet::new(vec![Entry::new("key", "")], &drawer);
        assert!(import_set.is_empty());
    }
}
