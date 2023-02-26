use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum OriginKind {
    LocalFile,
    OtherFile,
}
impl fmt::Display for OriginKind {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::LocalFile => write!(f, "Import from *s*ame file"),
            Self::OtherFile => write!(f, "Import from *a*nother file"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConfirmOrGoDeeper {
    Confirm,
    GoDeeper,
    Cancel,
}
impl fmt::Display for ConfirmOrGoDeeper {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Confirm => write!(f, "Import this drawer"),
            Self::GoDeeper => write!(f, "Open a deeper drawer"),
            Self::Cancel => write!(f, "Cancel"),
        }
    }
}
