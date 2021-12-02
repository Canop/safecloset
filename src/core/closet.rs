use {
    super::*,
    aes_gcm_siv::{aead::NewAead, Aes256GcmSiv, Key},
    rand::{thread_rng, Rng, seq::SliceRandom},
    serde::{Deserialize, Serialize},
    std::{
        fs,
        path::Path,
    },
};

/// The closet containing all the crypted drawers
#[derive(Serialize, Deserialize)]
pub struct Closet {

    /// Clear comments, which can be read with a standard binary/hex editor
    #[serde(default = "default_clear_comments")]
    pub comments: String,

    /// The salt used to generate the cipher keys from the passwords
    pub salt: String,

    /// The crypted drawers
    pub drawers: Vec<ClosedDrawer>,
}

/// Return the default comments which are written in the clear
/// part of the file. The rationale for writing the name of the
/// soft is that you may forget it and need to find the software
/// to open your closet file, while attackers are the ones who
/// may the most easily guess it.
fn default_clear_comments() -> String {
    "Closet file written with SafeCloset\nhttps://dystroy.org/safecloset".to_string()
}

/// compute the number of decoy drawers we must create for
/// the given depth
fn random_decoy_drawers_count(depth: usize) -> usize {
    let mut n = match depth {
        0 => thread_rng().gen_range(3..6),
        1 => thread_rng().gen_range(1..3),
        2 => thread_rng().gen_range(0..2),
        _ => 0,
    };
    while thread_rng().gen_bool(0.2) {
        n += 1;
    }
    n
}

impl Closet {

    pub fn new(depth: usize) -> Result<Self, CoreError> {
        let comments = default_clear_comments();
        let salt = random_password();
        let drawers = Vec::new();
        let mut closet = Self { comments, salt, drawers };
        // creating decoy drawers
        for _ in 0..random_decoy_drawers_count(depth) {
            closet.create_drawer_unchecked(depth, random_password())?;
        }
        Ok(closet)
    }

    /// Save the closet to a file
    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        if path.exists() {
            let backup_path = path.with_extension("old");
            if backup_path.exists() {
                fs::remove_file(&backup_path)?;
            }
            fs::rename(path, &backup_path)?;
        }
        self.write_to_file(path)?;
        Ok(())
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), CoreError> {
        if path.exists() {
            return Err(CoreError::FileExists(path.to_path_buf()));
        }
        let mut file = fs::File::create(path)?;
        rmp_serde::encode::write_named(&mut file, &self)?;
        Ok(())
    }


    /// read a closet from a file
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let file = fs::File::open(path)?;
        let closet = rmp_serde::decode::from_read(file)?;
        Ok(closet)
    }

    /// Create a drawer without checking first the password isn't used by
    /// another drawer, or that the password meets minimal requirements,
    /// add it to the closed drawers of the closet.
    ///
    /// This is fast but dangerous, and should not be used on user action.
    ///
    /// TODO add it automatically or not ? Maybe only if saved
    ///     mais alors il faut revérifier password collision
    fn create_drawer_unchecked(
        &mut self,
        depth: usize,
        password: String,
    ) -> Result<OpenDrawer, CoreError> {
        let drawer_content = DrawerContent::new(depth)?;
        let mut open_drawer = OpenDrawer::new(depth, password, drawer_content);
        let closed_drawer = open_drawer.close(self)?;
        self.drawers.push(closed_drawer);
        Ok(open_drawer)
    }

    /// Create a drawer, add it to the closet.
    ///
    /// Return an error if the password is already used by
    /// another drawer (which probably means the user wanted
    /// to open a drawer and not create one).
    pub fn create_drawer(
        &mut self,
        depth: usize,
        password: String,
    ) -> Result<OpenDrawer, CoreError> {
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(CoreError::PasswordTooShort);
        }
        if self.is_password_taken(depth, &password) {
            return Err(CoreError::PasswordAlreadyUsed);
        }
        self.create_drawer_unchecked(depth, password)
    }

    /// Open the drawer responding to this password and return it.
    ///
    /// Return None when no drawer can be opened with this password.
    pub fn open_drawer(
        &self,
        depth: usize,
        password: &str,
    ) -> Option<OpenDrawer> {
        for closed_drawer in &self.drawers {
            let open_drawer = time!(
                "closed_drawer.open",
                closed_drawer.open(
                    depth,
                    password.to_string(),
                    self,
                )
            );
            if let Ok(open_drawer) =  open_drawer {
                return Some(open_drawer);
            }
        }
        None
    }

    pub fn is_password_taken(
        &self,
        depth: usize,
        password: &str,
    ) -> bool {
        self.open_drawer(depth, password).is_some()
    }

    /// Close the passed drawer, put it back among closed ones
    pub fn close_drawer(
        &mut self,
        mut open_drawer: OpenDrawer,
    ) -> Result<bool, CoreError> {
        let closed_drawer = open_drawer.close(self)?;
        Ok(self.push_drawer_back(closed_drawer))
    }

    fn push_drawer_back(&mut self, drawer: ClosedDrawer) -> bool {
        for idx in 0..self.drawers.len() {
            if self.drawers[idx].has_same_id(&drawer) {
                self.drawers[idx] = drawer;
                return true;
            }
        }
        false
    }

    /// Change the order of drawers
    pub fn shuffle_drawers(&mut self) {
        self.drawers.shuffle(&mut thread_rng());
    }

    /// Close the drawer then reopen it.
    ///
    /// After this operation, the closet contains the content of the given
    /// drawer but the closet isn't saved.
    #[allow(dead_code)]
    pub fn close_then_reopen(
        &mut self,
        drawer: OpenDrawer,
    ) -> Result<OpenDrawer, CoreError> {
        let depth = drawer.depth;
        let password = drawer.password.clone();
        self.close_drawer(drawer)?;
        self.open_drawer(depth, &password).ok_or_else(|| {
            // shouldn't happen
            CoreError::InternalError("can't reopen just closed drawer".to_string())
        })
    }

    pub fn cipher(&self, password: &str) -> Result<Aes256GcmSiv, CoreError> {
        // TODO does this config totally determine and freeze the version ?
        let config = argon2::Config {
            hash_length: 32,
            ..Default::default()
        };
        //config.variant = argon2::Variant::Argon2i;
        //config.version = argon2::Version::Version13;
        let hash = argon2::hash_raw(password.as_bytes(), self.salt.as_bytes(), &config)?;
        let key = Key::from_slice(&hash);
        Ok(Aes256GcmSiv::new(key))
    }
}

