use {
    super::*,
    aes_gcm_siv::{aead::NewAead, Aes256GcmSiv, Key},
    rand::{seq::SliceRandom, thread_rng},
    serde::{Deserialize, Serialize},
    std::{fs::File, path::Path},
};

/// A closet as it is serialized to a file
#[derive(Serialize, Deserialize)]
pub struct SerCloset {

    /// The salt used to generate the cipher keys from the passwords
    pub salt: String,

    /// The crypted drawers
    pub drawers: Vec<ClosedDrawer>,
}

impl SerCloset {
    pub fn new() -> Self {
        Self {
            salt: random_password(),
            drawers: Vec::new(),
        }
    }

    pub fn push_drawer_back(&mut self, drawer: ClosedDrawer, idx: usize) {
        self.drawers[idx] = drawer;
        self.drawers.shuffle(&mut thread_rng());
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), CoreError> {
        if path.exists() {
            return Err(CoreError::FileExists(path.to_path_buf()));
        }
        let mut file = File::create(path)?;
        rmp_serde::encode::write_named(&mut file, &self)?;
        Ok(())
    }

    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let file = File::open(path)?;
        let sc = rmp_serde::decode::from_read(file)?;
        Ok(sc)
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
