use {
    super::*,
    aes_gcm_siv::{
        Aes256GcmSiv,
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

    /// Try to decrypt the content with the cipher derived from the
    /// password, then return the open drawer with clear data and the
    /// password to allow reencrypting.
    ///
    /// This function can also be used to check drawer existence.
    pub fn open(
        &self,
        depth: usize,
        password: &str,
        cipher: &Aes256GcmSiv,
    ) -> Result<OpenDrawer, CoreError> {
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
                password: password.to_string(),
                content,
            })
        }
    }
}
