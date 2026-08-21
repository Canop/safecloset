//! Tests ensuring closet files written by previous versions of SafeCloset
//! stay readable, and stay intact when saved by the current version.
//!
//! The reference closets live in the `test-closets` directory. They must be
//! considered read-only: saving a closet renames its file to `*.old` and
//! rewrites it in the current format, which would silently turn a reference
//! closet into a current one and leave those tests green forever. So every
//! test here works on a copy in a temporary directory.
//!
//! Reference closets hold no secret: their passphrases are right below.
//!
//! See `test-closets/README.md` for how to add one.

use {
    super::*,
    std::{
        fs,
        path::{
            Path,
            PathBuf,
        },
    },
    tempfile::TempDir,
};

/// Passphrase of the main drawer, at the root of the reference closets
const ROOT_PWD: &str = "test-closet-root-drawer";

/// Passphrase of the root drawer holding non default settings
const SETTINGS_PWD: &str = "test-closet-settings-drawer";

/// Passphrase of the drawer inside the main drawer
const DEEP_PWD: &str = "test-closet-deep-drawer";

/// Passphrase of the drawer inside the deep drawer
const DEEPER_PWD: &str = "test-closet-deeper-drawer";

/// A passphrase which must open no drawer of the reference closets
const UNUSED_PWD: &str = "test-closet-no-such-drawer";

/// The clear comments of the reference closets
const COMMENTS: &str = "reference closet for SafeCloset compatibility tests";

fn root_entries() -> Vec<Entry> {
    vec![
        Entry::new("plain", "simple value"),
        Entry::new("unicode", "Ünïcödé — 请教别人一次是五分钟的傻子 — 🔐"),
        Entry::new("multiline", "first line\nsecond line\n\nfourth line"),
        Entry::new(
            "markup",
            "a *value* with **bold**, `code` and a \\ backslash",
        ),
        Entry::new("empty value", ""),
    ]
}

fn settings_entries() -> Vec<Entry> {
    vec![Entry::new(
        "settings",
        "this drawer has non default settings",
    )]
}

fn deep_entries() -> Vec<Entry> {
    vec![Entry::new(
        "deep",
        "entry of the drawer inside the main drawer",
    )]
}

fn deeper_entries() -> Vec<Entry> {
    vec![Entry::new(
        "deeper",
        "entry of the drawer inside the deep drawer",
    )]
}

/// Number of drawers at each level of a reference closet, the ones we
/// can't open included.
///
/// Compared before and after a save so that no drawer can be lost or
/// duplicated by the writing code.
#[derive(Debug, PartialEq, Eq)]
struct DrawerCounts {
    root: usize,
    in_root_drawer: usize,
    in_deep_drawer: usize,
}

fn reference_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test-closets")
}

/// Copy a reference closet into a new temporary directory and open it.
///
/// The returned TempDir must be kept alive as long as the closet is used.
fn open_copy(file_name: &str) -> (TempDir, OpenCloset) {
    let reference_path = reference_dir().join(file_name);
    assert!(
        reference_path.exists(),
        "missing reference closet: {}",
        reference_path.display(),
    );
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join(file_name);
    fs::copy(&reference_path, &path).unwrap();
    let open_closet = OpenCloset::open(path).unwrap();
    (temp_dir, open_closet)
}

/// Check the whole expected content of a reference closet, and return the
/// number of drawers found at each level.
///
/// Called both on the fresh copy and after a save, so that the writing
/// code can't silently alter anything.
fn check_content(open_closet: &mut OpenCloset) -> DrawerCounts {
    // an unused passphrase must open nothing
    assert!(open_closet.open_drawer(UNUSED_PWD).is_none());

    // deeper drawers must not be reachable from the root
    assert!(open_closet.open_drawer(DEEP_PWD).is_none());
    assert!(open_closet.open_drawer(DEEPER_PWD).is_none());

    let root = open_closet.root_drawers_count();

    // the root drawer holding non default settings
    let drawer = open_closet.open_drawer(SETTINGS_PWD).unwrap();
    assert_eq!(drawer.depth, 0);
    assert_eq!(drawer.content.entries, settings_entries());
    assert!(drawer.content.settings.hide_values);
    assert!(drawer.content.settings.open_all_values);
    assert!(drawer.content.settings.values_as_markdown);

    // the main drawer
    let drawer = open_closet.open_drawer(ROOT_PWD).unwrap();
    assert_eq!(drawer.depth, 0);
    assert_eq!(drawer.content.entries, root_entries());
    // default settings must be read as such, even from a closet written
    // before those fields existed
    assert!(!drawer.content.settings.hide_values);
    assert!(!drawer.content.settings.open_all_values);
    assert!(!drawer.content.settings.values_as_markdown);
    let in_root_drawer = drawer.content.closet.drawers.len();

    // the drawer inside the main drawer
    let drawer = open_closet.open_drawer(DEEP_PWD).unwrap();
    assert_eq!(drawer.depth, 1);
    assert_eq!(drawer.content.entries, deep_entries());
    let in_deep_drawer = drawer.content.closet.drawers.len();

    // the drawer inside the deep drawer
    let drawer = open_closet.open_drawer(DEEPER_PWD).unwrap();
    assert_eq!(drawer.depth, 2);
    assert_eq!(drawer.content.entries, deeper_entries());

    assert_eq!(open_closet.root_closet().comments, COMMENTS);

    DrawerCounts {
        root,
        in_root_drawer,
        in_deep_drawer,
    }
}

/// Check a reference closet is still readable
fn check_readable(file_name: &str) {
    let (temp_dir, mut open_closet) = open_copy(file_name);
    check_content(&mut open_closet);
    temp_dir.close().unwrap();
}

/// Check saving a reference closet with the current version keeps the whole
/// content readable, drawers we can't open included
fn check_save_preserves_content(file_name: &str) {
    let (temp_dir, mut open_closet) = open_copy(file_name);
    let counts_before = check_content(&mut open_closet);
    let path = open_closet.path().to_path_buf();

    // this closes and reencrypts the drawers left open by check_content
    open_closet.close_and_save().unwrap();

    let mut open_closet = OpenCloset::open(path).unwrap();
    let counts_after = check_content(&mut open_closet);
    assert_eq!(counts_before, counts_after);
    temp_dir.close().unwrap();
}

/// All the reference closets, sorted, so that adding one is just a matter
/// of dropping its file in the directory
fn reference_closets() -> Vec<String> {
    let dir = reference_dir();
    let dir_entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("can't read {}: {e}", dir.display()));
    let mut file_names: Vec<String> = dir_entries
        .map(|dir_entry| dir_entry.unwrap().file_name())
        .filter_map(|file_name| file_name.into_string().ok())
        .filter(|file_name| file_name.ends_with(".closet"))
        .collect();
    file_names.sort();
    // a discovery finding nothing would make the tests below pass without
    // checking anything
    assert!(
        !file_names.is_empty(),
        "no reference closet in {}",
        dir.display(),
    );
    file_names
}

/// The name of the closet being checked is printed, and the test harness
/// shows the output of failing tests, so a failure tells which reference
/// closet broke.
#[test]
fn test_read_reference_closets() {
    for file_name in reference_closets() {
        println!("reading reference closet {file_name}");
        check_readable(&file_name);
    }
}

#[test]
fn test_save_reference_closets() {
    for file_name in reference_closets() {
        println!("saving reference closet {file_name}");
        check_save_preserves_content(&file_name);
    }
}

/// Write the reference closet of the current version, to be committed in
/// the `test-closets` directory.
///
/// Not run by default. To generate it:
///
/// ```not_rust
/// cargo test generate_reference_closet -- --ignored --nocapture
/// ```
///
/// Older reference closets can't be generated this way, as they must be
/// written by the version they're testing: see `test-closets/README.md`.
#[test]
#[ignore = "writes a reference closet, must be run explicitly"]
fn generate_reference_closet() {
    let dir = reference_dir();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}.closet", env!("CARGO_PKG_VERSION")));
    assert!(
        !path.exists(),
        "{} already exists, remove it first",
        path.display(),
    );

    let mut open_closet = OpenCloset::create(path.clone()).unwrap();
    open_closet.root_closet().comments = COMMENTS.to_string();

    // the root drawer with non default settings
    let drawer = open_closet.create_drawer(SETTINGS_PWD).unwrap();
    drawer.content.entries = settings_entries();
    drawer.content.settings.hide_values = true;
    drawer.content.settings.open_all_values = true;
    drawer.content.settings.values_as_markdown = true;
    open_closet.close_deepest_drawer().unwrap();

    // the main drawer, then a drawer in it, then a drawer in that one
    let drawer = open_closet.create_drawer(ROOT_PWD).unwrap();
    drawer.content.entries = root_entries();
    let drawer = open_closet.create_drawer(DEEP_PWD).unwrap();
    drawer.content.entries = deep_entries();
    let drawer = open_closet.create_drawer(DEEPER_PWD).unwrap();
    drawer.content.entries = deeper_entries();

    open_closet.close_and_save().unwrap();

    // check what we just wrote answers the same expectations as the
    // already committed reference closets
    let mut open_closet = OpenCloset::open(path.clone()).unwrap();
    check_content(&mut open_closet);

    println!("reference closet written: {}", path.display());
}
