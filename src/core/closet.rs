use {
    super::*,
    rand::{thread_rng, Rng},
    std::{
        fs,
        ops::Range,
        path::{Path, PathBuf},
    },
};

/// The closet containing all the drawers, crypted.
///
/// Only one drawer can be opened at a time.
pub struct Closet {

    /// the path to the file in which the closet is persisted
    path: PathBuf,

    /// contains all drawers, closed
    ser_closet: SerCloset,

    /// an id attached to the currently open drawer, if any.
    ///
    /// This is necessary to ensure the consistency of the
    /// reinsertion idx which is in the open drawer, as the
    /// drawers are scrambled when one is closed.
    open_id: Option<usize>,
}

const MIN_PASSWORD_LENGTH: usize = 10;
const DECOY_DRAWERS_COUNT: Range<usize> = 7..20;

impl Closet {

    /// Either create a new closet, or open an existing one, depending
    /// on whether the file exists
    pub fn open_or_create<P: Into<PathBuf>>(path: P) -> Result<Self, CoreError> {
        let path = path.into();
        if path.exists() {
            Self::open(path)
        } else {
            Self::create(path)
        }
    }

    /// Create a new closet, with a random number of drawers
    /// (which won't be openable as you won't have their password)
    pub fn create(path: PathBuf) -> Result<Self, CoreError> {
        if path.exists() {
            return Err(CoreError::FileExists(path));
        }
        let mut closet = Closet {
            path,
            ser_closet: SerCloset::new(),
            open_id: None,
        };
        let decoy_drawer_count = thread_rng().gen_range(DECOY_DRAWERS_COUNT);
        for _ in 0..decoy_drawer_count {
            let password = random_password();
            closet.create_drawer_unchecked(&password)?;
        }
        Ok(closet)
    }

    /// Open a closet from a closet file
    pub fn open(path: PathBuf) -> Result<Self, CoreError> {
        let ser_closet = SerCloset::from_file(&path)?;
        let closet = Closet {
            path,
            ser_closet,
            open_id: None,
        };
        Ok(closet)
    }

    /// Create a drawer without checking first the password isn't used by
    /// another drawer, or that the password meets minimal requirements.
    ///
    /// This is fast but dangerous, and should not be used on user action.
    fn create_drawer_unchecked(&mut self, password: &str) -> Result<(), CoreError> {
        let drawer_idx = self.ser_closet.drawers.len();
        let drawer = OpenDrawer {
            password: password.to_string(),
            drawer_idx,
            entries: Vec::new(),
            settings: DrawerSettings::default(),
            open_id: 0, // it's not really open
        };
        // the crypted drawer is pushed at the end, so if there's an open
        // driver its open_id is still valid
        let crypted_drawer = ClosedDrawer::from_open_drawer(drawer, &self.ser_closet)?;
        self.ser_closet.drawers.push(crypted_drawer);
        Ok(())
    }

    /// Create a drawer, don't open it, add it to the closet.
    ///
    /// Return an error if the password is already used by
    /// another drawer (which probably means the user wanted
    /// to open a drawer and not create one).
    pub fn create_drawer(&mut self, password: &str) -> Result<(), CoreError> {
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(CoreError::PasswordTooShort);
        }
        // check no existing drawer already has this password
        for (drawer_idx, closed_drawer) in self.ser_closet.drawers.iter().enumerate() {
            if closed_drawer.open(drawer_idx, password, &self.ser_closet, 0).is_ok() {
                return Err(CoreError::PasswordAlreadyUsed);
            }
        }
        self.create_drawer_unchecked(password)
    }

    /// Open the drawer responding to this password and return it.
    ///
    /// Return None when no drawer can be opened with this password.
    pub fn open_drawer(&mut self, password: &str) -> Option<OpenDrawer> {
        let open_id = rand::thread_rng().gen();
        for (drawer_idx, closed_drawer) in self.ser_closet.drawers.iter().enumerate() {
            if let Ok(open_drawer) =  closed_drawer.open(drawer_idx, password, &self.ser_closet, open_id) {
                debug!("open id: {}", open_id);
                self.open_id = Some(open_id);
                return Some(open_drawer);
            }
        }
        None
    }

    /// Close the passed drawer, scramble the closet, forget the
    /// password.
    pub fn close_drawer(&mut self, drawer: OpenDrawer) -> Result<(), CoreError> {
        if self.open_id != Some(drawer.open_id) {
            return Err(CoreError::WrongOpenId);
        }
        let drawer_idx = drawer.drawer_idx;
        let crypted_drawer = ClosedDrawer::from_open_drawer(drawer, &self.ser_closet)?;
        self.ser_closet.push_drawer_back(crypted_drawer, drawer_idx);
        self.open_id = None;
        Ok(())
    }

    /// Close the drawer then reopen it.
    ///
    /// After this operation, the closet contains the content of the given
    /// drawer but the closet isn't saved.
    pub fn close_then_reopen(&mut self, drawer: OpenDrawer) -> Result<OpenDrawer, CoreError> {
        let password = drawer.password.clone();
        self.close_drawer(drawer)?;
        self.open_drawer(&password).ok_or_else(|| {
            // shouldn't happen
            CoreError::InternalError("can't reopen just closed drawer".to_string())
        })
    }

    /// Save the closet. Open drawers must be closed before or
    /// they won't be included in the saving
    pub fn save(&mut self) -> Result<(), CoreError> {
        if self.path.exists() {
            let backup_path = self.path.with_extension("old");
            if backup_path.exists() {
                fs::remove_file(&backup_path)?;
            }
            fs::rename(&self.path, &backup_path)?;
        }
        self.ser_closet.write_to_file(&self.path)?;
        Ok(())
    }

    /// return the path to the closet file
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[test]
fn test_create_write_read() {
    let pwd1 = "some test password (not a hard one but it's a test)";
    let pwd2 = "请教别人一次是五分钟的傻子，从不请教别人是一辈子的傻子。";
    let entry1 = Entry {
        name: "some key".to_string(),
        value: "some value".to_string(),
    };

    // create a temp directory in which to run our tests
    let temp_dir = tempfile::tempdir().unwrap();

    // define a path for our closet
    let path = temp_dir.path().join("test.safe-closet");

    // create a closet on this path
    let mut closet = Closet::create(path.to_path_buf()).unwrap();

    // check that there are already several drawers
    assert!(closet.ser_closet.drawers.len() >= DECOY_DRAWERS_COUNT.min().unwrap());

    // create 2 drawers
    closet.create_drawer(pwd1).unwrap();
    closet.create_drawer(pwd2).unwrap();

    // check we can't reuse a password
    assert!(matches!(
        closet.create_drawer(pwd1),
        Err(CoreError::PasswordAlreadyUsed),
    ));

    // reopen the first drawer and add an entry
    let mut open_drawer = closet.open_drawer(pwd1).unwrap();
    open_drawer.entries.push(entry1.clone());

    // closing the drawer
    closet.close_drawer(open_drawer).unwrap();

    // saving the closet
    closet.save().unwrap();

    // reopen the closet
    let mut closet = Closet::open(path).unwrap();

    // open the first drawer, check our entry is here
    let open_drawer = closet.open_drawer(pwd1).unwrap();
    assert_eq!(open_drawer.entries, vec![entry1]);

    // open the second drawer, check there's no entry
    let open_drawer = closet.open_drawer(pwd2).unwrap();
    assert!(open_drawer.entries.is_empty());

    // clean the temporary dir
    temp_dir.close().unwrap();
}
