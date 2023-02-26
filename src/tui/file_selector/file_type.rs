use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct FileCheck {
    pub ok: bool,
    pub message: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    File,
}

impl FileCheck {
    pub fn new(
        ok: bool,
        message: &'static str,
    ) -> Self {
        Self { ok, message }
    }
}

impl FileType {
    pub fn check(
        self,
        path: &Path,
    ) -> FileCheck {
        if path.components().count() == 0 {
            return FileCheck::new(false, "Type the path to the file to open");
        }
        if !path.exists() || !path.is_file() {
            return FileCheck::new(false, "Type the path of a file");
        }
        // we don't check the extension because people can name files how they want
        FileCheck::new(true, "Type *enter* to select this file")
    }
}
