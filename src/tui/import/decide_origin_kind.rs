

pub struct DecideOriginKind {
    selection: OriginKind,
}

impl Default for DecideOriginKind {
    fn default() -> Self {
        Self {
            selection: OriginKind::LocalFile,
        }
    }
}


#[derive(Default)]
pub struct DecideOriginKindView {
    available_area: Area,
}

impl View for DecideOriginKindView {
    type State = DecideOriginKindState;

    fn set_available_area(&mut self, available_area: Area) {
        if available_area != self.available_area {
            self.available_area = available_area;
        }
    }

    /// Draw the menu and set the area of all visible items in the state
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut DecideOriginKind,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        Ok(())
    }
}

