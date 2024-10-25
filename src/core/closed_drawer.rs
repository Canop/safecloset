use {
    super::*,
    aes_gcm_siv::{
        Nonce,
        aead::Aead,
    },
    serde::{
        Deserialize,
        Serialize,
    },
};

/// a closed, crypted, drawer
#[derive(Serialize, Deserialize)]
pub struct ClosedDrawer {
    id: DrawerId,

    nonce: Box<[u8]>,

    /// crypted serialized DrawerContent
    content: Box<[u8]>,
}

impl Identified for ClosedDrawer {
    fn get_id(&self) -> &DrawerId {
        &self.id
    }
}

impl ClosedDrawer {
    pub fn new(
        id: DrawerId,
        nonce: Box<[u8]>,
        content: Box<[u8]>,
    ) -> Self {
        Self { id, nonce, content }
    }

    /// Try to decrypt the content with the provided password
    /// and the closet's salt, then return the open drawer with
    /// clear data and the password to allow reencrypting.
    ///
    /// This function can also be used to check drawer existence.
    pub fn open(
        &self,
        depth: usize,
        password: String,
        closet: &Closet,
    ) -> Result<OpenDrawer, CoreError> {
        let cipher = closet.cipher(&password)?;
        let nonce = Nonce::from_slice(&self.nonce);
        let clear_content = cipher
            .decrypt(nonce, self.content.as_ref())
            .map_err(|_| CoreError::Aead)?;
        let content: DrawerContent = rmp_serde::from_read(&*clear_content)?;
        if content.id != self.id {
            Err(CoreError::UnconsistentData)
        } else {
            Ok(OpenDrawer {
                depth,
                password,
                content,
            })
        }
    }
}
