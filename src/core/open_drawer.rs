use {
    super::*,
    aes_gcm_siv::{aead::Aead},
};

/// An open drawer, with its content and the pass
/// making it possible to save it on change
///
pub struct OpenDrawer {
    pub depth: usize,
    pub(super) password: String,
    pub content: DrawerContent,
}

impl Identified for OpenDrawer {
    fn get_id(&self) -> &DrawerId {
        self.content.get_id()
    }
}

impl OpenDrawer {

    pub(crate) fn new(
        depth: usize,
        password: String,
        content: DrawerContent,
    ) -> Self {
        Self { depth, password, content }
    }

    /// change the drawer_content into a closed_drawer
    pub(crate) fn close(
        &self,
        closet: &Closet,
    ) -> Result<ClosedDrawer, CoreError> {
        let cipher = closet.cipher(&self.password)?;
        let serialized_content = rmp_serde::encode::to_vec_named(&self.content)?;
        let nonce = random_nonce();
        let crypted_content = cipher
            .encrypt(&nonce, &*serialized_content)
            .map_err(|_| CoreError::Aead)?;
        let nonce = nonce.as_slice().into();
        let id = self.content.id.clone();
        Ok(ClosedDrawer::new(
            id,
            nonce,
            crypted_content.into_boxed_slice(),
        ))

    }
}

