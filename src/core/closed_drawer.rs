use {
    super::*,
    aes_gcm_siv::{aead::Aead, Nonce},
    serde::{Deserialize, Serialize},
};

/// a closed, crypted, drawer
#[derive(Serialize, Deserialize)]
pub struct ClosedDrawer {

    nonce: Box<[u8]>,

    /// crypted content
    content: Box<[u8]>,
}

impl ClosedDrawer {

    /// Make back a closed drawer from an open one,
    /// by generating a new random nonce then crypting
    /// the drawer content
    pub fn from_open_drawer(
        open_drawer: OpenDrawer,
        closet: &SerCloset,
    ) -> Result<Self, CoreError> {
        let cipher = closet.cipher(&open_drawer.password)?;
        let ser_drawer = SerDrawer::new(open_drawer, closet.salt.clone());
        let clear_content = rmp_serde::encode::to_vec_named(&ser_drawer)?;
        let nonce = random_nonce();
        let crypted_content = cipher
            .encrypt(&nonce, &*clear_content)
            .map_err(|_| CoreError::Aead)?;
        let nonce = nonce.as_slice().into();
        Ok(Self {
            nonce,
            content: crypted_content.into_boxed_slice(),
        })
    }

    /// Try to decrypt the content with the provided password
    /// and the closet's salt, then return the open drawer with
    /// clear data and the password to allow reencrypting.
    ///
    /// This function can also be used to check drawer existence.
    pub fn open(
        &self,
        drawer_idx: usize,
        password: &str,
        closet: &SerCloset,
        open_id: usize,
    ) -> Result<OpenDrawer, CoreError> {
        let cipher = closet.cipher(password)?;
        let nonce = Nonce::from_slice(&self.nonce);
        let clear_content = cipher
            .decrypt(nonce, self.content.as_ref())
            .map_err(|_| CoreError::Aead)?;
        let ser_drawer: SerDrawer = rmp_serde::from_read(&*clear_content)?;
        ser_drawer.into_open_drawer(
            drawer_idx,
            password.to_string(),
            open_id,
            &closet.salt,
        )
    }
}
