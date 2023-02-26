use {
    super::*,
    crate::tui::*,
    termimad::*,
};

#[derive(Default)]
pub struct ImportView {
    available_area: Area,
}

impl ImportView {
    fn set_view_available_area(
        &self,
        state: &mut ImportState,
    ) {
        match &mut state.step {
            Step::DecideOriginKind(menu) => {
                menu.set_available_area(self.available_area.clone());
            }
            Step::FileSelector(selector) => {
                selector
                    .view
                    .set_available_area(self.available_area.clone());
            }
            Step::TypeDrawerPassword { dialog, .. } => {
                dialog.view.set_available_area(self.available_area.clone());
            }
            Step::ConfirmImportCsv { menu, .. } => {
                menu.set_available_area(self.available_area.clone());
            }
            Step::ConfirmImportDrawer { menu, .. } => {
                menu.set_available_area(self.available_area.clone());
            }
            Step::InformEnd(menu) => {
                menu.set_available_area(self.available_area.clone());
            }
            Step::Finished => {}
        }
    }
}

impl View<ImportState> for ImportView {
    fn set_available_area(
        &mut self,
        area: Area,
    ) {
        self.available_area = area;
    }

    /// Render the view in its area
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut ImportState, // mutable to allow adapt to terminal size changes
        skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.set_view_available_area(state);

        match &mut state.step {
            Step::DecideOriginKind(menu) => {
                menu.draw(w, skin)?;
            }
            Step::FileSelector(selector) => {
                selector.draw(w, skin)?;
            }
            Step::TypeDrawerPassword { dialog, .. } => {
                dialog.draw(w, skin)?;
            }
            Step::ConfirmImportCsv { menu, .. } => {
                menu.draw(w, skin)?;
            }
            Step::ConfirmImportDrawer { menu, .. } => {
                menu.draw(w, skin)?;
            }
            Step::InformEnd(menu) => {
                menu.draw(w, skin)?;
            }
            Step::Finished => {}
        }
        Ok(())
    }
}
