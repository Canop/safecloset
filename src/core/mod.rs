mod closed_drawer;
mod closet;
mod core_error;
mod drawer_content;
mod drawer_id;
mod drawer_settings;
mod entry;
mod open_closet;
mod open_drawer;
mod random;

pub use {
    closed_drawer::*,
    closet::*,
    core_error::*,
    drawer_content::*,
    drawer_id::*,
    drawer_settings::*,
    entry::*,
    open_closet::*,
    open_drawer::*,
    random::*,
};

pub const MIN_PASSWORD_LENGTH: usize = 2;

/// test most opening, saving, reopening, etc. operations in
/// a complex scenario
#[test]
fn test_create_write_read() {
    let pwd1 = "some test password (not a hard one but it's a test)";
    let pwd2 = "请教别人一次是五分钟的傻子，从不请教别人是一辈子的傻子。";
    let pwd3 = "les sanglots lents et violents de l'autre âne";
    let entry1 = Entry::new("some key", "some value");
    let entry2 = Entry::new("key2", "some value");
    let entry2b = Entry::new("another key", "v2.2");
    let entry2c = Entry::new("key 2c", "v2.3");
    let entry3 = Entry::new("key3", "some other value");
    let entry3b = Entry::new("hey", "what's here?");

    // create a temp directory in which to run our tests
    let temp_dir = tempfile::tempdir().unwrap();

    // define a path for our closet
    let path = temp_dir.path().join("test-create-write-read.safe-closet");

    // create a closet on this path
    let mut open_closet = OpenCloset::create(path.to_path_buf()).unwrap();

    // check that there are already several drawers
    assert!(open_closet.root_drawers_count() >= 3);

    // create 2 drawers at the same level
    open_closet.create_drawer(pwd1).unwrap();
    open_closet.close_deepest_drawer().unwrap();
    let drawer2 = open_closet.create_drawer(pwd2).unwrap();
    drawer2.content.entries.push(entry2.clone());

    // check drawer2 already contains at least a decoy drawer
    assert!(!drawer2.content.closet.drawers.is_empty());

    // close the drawer2
    open_closet.close_deepest_drawer().unwrap();

    // check we can't reuse a password (from a given node)
    assert!(matches!(
        open_closet.create_drawer(pwd1),
        Err(CoreError::PasswordAlreadyUsed),
    ));

    // reopen the first drawer and add an entry
    let open_drawer = open_closet.open_drawer(pwd1).unwrap();
    open_drawer.content.entries.push(entry1.clone());

    // save the closet
    open_closet.close_and_save().unwrap();

    // reopen the closet
    let mut open_closet = OpenCloset::open(path.to_path_buf()).unwrap();

    // open the first drawer, check our entry is here
    let open_drawer = open_closet.open_drawer(pwd1).unwrap();
    assert_eq!(open_drawer.content.entries, vec![entry1.clone()]);

    // open the second drawer, check there's our entry2
    // (we don't close the first one before: we want to let
    // safecloset find it alone)
    let drawer2 = open_closet.open_drawer(pwd2).unwrap();
    assert_eq!(drawer2.content.entries, vec![entry2.clone()]);
    assert_eq!(drawer2.depth, 0);

    // now, let's create a deep drawer inside the second drawer
    let drawer3 = open_closet.create_drawer(pwd3).unwrap();
    drawer3.content.entries.push(entry3.clone());
    assert_eq!(drawer3.depth, 1);

    // let's save and close everything
    open_closet.close_and_save().unwrap();

    // reopen the closet
    let mut open_closet = OpenCloset::open(path.to_path_buf()).unwrap();

    // check we can't open drawer3 from the root level
    assert!(open_closet.open_drawer(pwd3).is_none());

    // open drawer2 then drawer3 from drawer2
    open_closet.open_drawer(pwd2).unwrap();
    let drawer3 = open_closet.open_drawer(pwd3).unwrap();

    // check its content
    assert_eq!(drawer3.depth, 1);
    assert_eq!(drawer3.content.entries, vec![entry3.clone()]);

    // check we can save without closing
    let drawer3 = open_closet.save_then_reopen().unwrap().unwrap();
    assert_eq!(drawer3.depth, 1);
    assert_eq!(drawer3.content.entries, vec![entry3.clone()]);

    // add some content in d3, we'll check again later it's ok
    drawer3.content.entries.push(entry3b.clone());

    // close drawer3, we should be beck in the enclosing drawer, d2
    open_closet.close_deepest_drawer().unwrap();
    let drawer2 = open_closet.deepest_open_drawer().unwrap();
    assert_eq!(drawer2.content.entries, vec![entry2.clone()]);
    drawer2.content.entries.push(entry2b.clone());

    // now let's make a change but by taking the drawer instead of
    // just having a reference
    let mut drawer2 = open_closet.take_deepest_open_drawer().unwrap();
    drawer2.content.entries.push(entry2c.clone());
    open_closet.push_back(drawer2).unwrap();

    // let's save and close everything again to check the content is right
    open_closet.close_and_save().unwrap();
    let mut open_closet = OpenCloset::open(path.to_path_buf()).unwrap();
    let drawer1 = open_closet.open_drawer(pwd1).unwrap();
    assert_eq!(drawer1.content.entries, vec![entry1.clone()]);
    let drawer2 = open_closet.open_drawer(pwd2).unwrap();
    assert_eq!(
        drawer2.content.entries,
        vec![entry2.clone(), entry2b.clone(), entry2c.clone()],
    );
    let drawer3 = open_closet.open_drawer(pwd3).unwrap();
    assert_eq!(
        drawer3.content.entries,
        vec![entry3.clone(), entry3b.clone()]
    );

    // clean the temporary dir
    temp_dir.close().unwrap();
}

/// test changing the password
#[test]
fn test_password_change() {
    let pwd1 = "*p*w*d*1";
    let pwd2 = "PWD2";
    let pwd3 = "p-w-d-3";
    let entry1 = Entry::new("some key", "some value");
    let entry2 = Entry::new("key2", "some value");

    // create a temp directory in which to run our tests
    let temp_dir = tempfile::tempdir().unwrap();

    // define a path for our closet
    let path = temp_dir.path().join("test-pwd-change.closet");

    // create a closet on this path
    let mut open_closet = OpenCloset::create(path.to_path_buf()).unwrap();

    // check that there are already several drawers
    assert!(open_closet.root_drawers_count() >= 3);

    // create 2 drawers at the same level, with some content
    let drawer1 = open_closet.create_drawer(pwd1).unwrap();
    drawer1.content.entries.push(entry1.clone());
    open_closet.close_deepest_drawer().unwrap();
    let drawer2 = open_closet.create_drawer(pwd2).unwrap();
    drawer2.content.entries.push(entry2.clone());

    // save the closet
    open_closet.close_and_save().unwrap();

    // reopen the closet
    let mut open_closet = OpenCloset::open(path.to_path_buf()).unwrap();

    // open the first drawer, check our entry is here
    open_closet.open_drawer(pwd1).unwrap();
    let mut open_drawer = open_closet.take_deepest_open_drawer().unwrap();
    assert_eq!(open_drawer.content.entries, vec![entry1.clone()]);

    // check we can't change the password to an already used one
    assert!(open_closet.change_password(&mut open_drawer, pwd1).is_err());

    // change the password
    open_closet.change_password(&mut open_drawer, pwd3).unwrap();

    // push back the drawer so that it can be saved
    open_closet.push_back(open_drawer).unwrap();

    // save the closet
    open_closet.close_and_save().unwrap();

    // reopen the closet
    let mut open_closet = OpenCloset::open(path.to_path_buf()).unwrap();

    // check we can't open with the old password
    assert!(open_closet.open_drawer(pwd1).is_none());

    // open with the new password
    open_closet.open_drawer(pwd3).unwrap();
    let drawer1 = open_closet.take_deepest_open_drawer().unwrap();

    // check its content
    assert_eq!(drawer1.content.entries, vec![entry1.clone()]);

    // save the closet
    open_closet.close_and_save().unwrap();

    // clean the temporary dir
    temp_dir.close().unwrap();
}
