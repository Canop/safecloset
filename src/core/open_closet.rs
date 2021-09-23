use {
    super::*,
    std::{
        path::{Path, PathBuf},
    },
};


/// only at root
pub struct OpenCloset {

    path: PathBuf,

    root_closet: Closet,

    // drawers, indexed by depth
    open_drawers: Vec<OpenDrawer>,

    // the closet was just created because there no preexisting file
    created: bool,

}


impl OpenCloset {

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

    pub fn just_created(&self) -> bool {
        self.created
    }

    #[cfg(test)]
    pub fn root_drawers_count(&self) -> usize {
        self.root_closet.drawers.len()
    }

    /// Create a new closet, with a random number of drawers
    /// (which won't be openable as you won't have their password)
    pub fn create(path: PathBuf) -> Result<Self, CoreError> {
        if path.exists() {
            return Err(CoreError::FileExists(path));
        }
        let open_closet = OpenCloset {
            path,
            root_closet: Closet::new(0)?,
            open_drawers: Vec::new(),
            created: true,
        };
        Ok(open_closet)
    }

    /// Open a closet from a closet file
    pub fn open(path: PathBuf) -> Result<Self, CoreError> {
        let root_closet = Closet::from_file(&path)?;
        let open_closet = OpenCloset {
            path,
            root_closet,
            open_drawers: Vec::new(),
            created: false,
        };
        Ok(open_closet)
    }

    /// Save all the open drawers, then the closet in its file.
    pub fn close_and_save(&mut self) -> Result<(), CoreError> {
        while !self.open_drawers.is_empty() {
            self.close_deepest_drawer()?;
        }
        self.root_closet.save(&self.path)
    }

    /// Save all the open drawers, then the closet in its file,
    /// then reopen the drawer which was the deepest one before
    /// saving.
    ///
    /// If nothing was open, nothing is reopened.
    pub fn save_then_reopen(&mut self) -> Result<Option<&mut OpenDrawer>, CoreError> {
        let mut passwords = Vec::new();
        while !self.open_drawers.is_empty() {
            passwords.push(self.close_deepest_drawer()?);
        }
        self.root_closet.save(&self.path)?;
        // now we reopen
        while let Some(password) = passwords.pop() {
            if !self.open_drawer_at_depth(self.depth(), &password) {
                return Err(CoreError::InternalError("drawer can't be reopened".to_string()));
            }
        }
        Ok(self.open_drawers.last_mut())
    }

    /// Save all the open drawers, then the closet in its file,
    /// then reopen the drawer which was the deepest one before
    /// saving.
    pub fn push_back_save_retake(
        &mut self,
        open_drawer: OpenDrawer,
    ) -> Result<OpenDrawer, CoreError> {
        self.push_back(open_drawer)?;
        self.save_then_reopen()?;
        Ok(self.take_deepest_open_drawer().unwrap()) // SAFETY: we just pushed back, so there's a drawer
    }

    /// return the path to the closet file
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn depth(&self) -> usize {
        self.open_drawers.len()
    }

    /// does nothing if (depth, password) don't match
    /// an existing drawer
    fn open_drawer_at_depth(
        &mut self,
        depth: usize,
        password: &str,
    ) -> bool {
        if depth > self.open_drawers.len() {
            warn!("invalid depth for drawer opening");
            return false;
        }
        let closet = if depth == 0 {
            &mut self.root_closet
        } else {
            &mut self.open_drawers[depth - 1].content.closet
        };
        if let Some(open_drawer) = closet.open_drawer(depth, password) {
            self.open_drawers.truncate(depth);
            self.open_drawers.push(open_drawer);
            true
        } else {
            false
        }
    }

    /// try to open a drawer at any depth (preferably from
    /// one of the deepest open drawers)
    pub fn open_drawer(&mut self, password: &str) -> Option<&mut OpenDrawer> {
        let mut depth = self.open_drawers.len();
        let mut open: bool;
        loop {
            open = self.open_drawer_at_depth(depth, password);
            if open || depth == 0 {
                break;
            }
            depth -= 1;
        }
        if open {
            Some(&mut self.open_drawers[depth])
        } else {
            None
        }
    }

    /// try to open a drawer at any depth (preferably from
    /// one of the deepest open drawers) then take it
    pub fn open_take_drawer(&mut self, password: &str) -> Option<OpenDrawer> {
        if self.open_drawer(password).is_some() {
            self.take_deepest_open_drawer()
        } else {
            None
        }
    }

    /// create a drawer at the deepest possible depth
    /// (to create a less deep drawer, you should close
    /// the deeper one(s) before)
    #[allow(dead_code)]
    pub fn create_drawer<S: Into<String>>(
        &mut self,
        password: S,
    ) -> Result<&mut OpenDrawer, CoreError> {
        let depth = self.open_drawers.len();
        let closet = if self.open_drawers.is_empty() {
            &mut self.root_closet
        } else {
            &mut self.open_drawers[depth - 1].content.closet
        };
        let open_drawer = closet.create_drawer(depth, password.into())?;
        self.open_drawers.push(open_drawer);
        Ok(&mut self.open_drawers[depth])
    }

    /// create a drawer at the deepest possible depth
    /// (to create a less deep drawer, you should close
    /// the deeper one(s) before)
    pub fn create_take_drawer<S: Into<String>>(
        &mut self,
        password: S,
    ) -> Result<OpenDrawer, CoreError> {
        let depth = self.open_drawers.len();
        let closet = if self.open_drawers.is_empty() {
            &mut self.root_closet
        } else {
            &mut self.open_drawers[depth - 1].content.closet
        };
        let open_drawer = closet.create_drawer(depth, password.into())?;
        Ok(open_drawer)
    }

    /// Close the deepest open drawer and return its password
    pub fn close_deepest_drawer(&mut self) -> Result<String, CoreError> {
        match self.open_drawers.pop() {
            Some(open_drawer) => {
                let password = open_drawer.password.clone();
                let closet = if self.open_drawers.is_empty() {
                    &mut self.root_closet
                } else {
                    let idx = self.open_drawers.len() - 1;
                    &mut self.open_drawers[idx].content.closet
                };
                closet.close_drawer(open_drawer)?;
                Ok(password)
            }
            None => {
                Err(CoreError::NoOpenDrawer)
            }
        }
    }

    #[allow(dead_code)]
    pub fn deepest_open_drawer(&mut self) -> Option<&mut OpenDrawer> {
        self.open_drawers.last_mut()
    }

    /// take a drawer to modify it owned. Won't be saved if you
    /// don't push it back
    pub fn take_deepest_open_drawer(&mut self) -> Option<OpenDrawer> {
        self.open_drawers.pop()
    }

    pub fn push_back(&mut self, open_drawer: OpenDrawer) -> Result<(), CoreError> {
        let depth = self.open_drawers.len();
        let closet = if self.open_drawers.is_empty() {
            &mut self.root_closet
        } else {
            &mut self.open_drawers[depth - 1].content.closet
        };
        if closet.drawers.iter().any(|closed_drawer| closed_drawer.has_same_id(&open_drawer)) {
            self.open_drawers.push(open_drawer);
            Ok(())
        } else {
            Err(CoreError::InvalidPushBack)
        }
    }

    //pub fn push_back_and_close(&mut self, open_drawer: OpenDrawer) -> Result<(), CoreError> {
    //    let depth = self.open_drawers.len();
    //    let closet = if self.open_drawers.is_empty() {
    //        &mut self.root_closet
    //    } else {
    //        &mut self.open_drawers[depth - 1].content.closet
    //    };
    //    if closet.close_drawer(open_drawer)? {
    //        Ok(())
    //    } else {
    //        Err(CoreError::InvalidPushBack)
    //    }
    //}
}
