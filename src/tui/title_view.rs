use {
    super::*,
    crate::error::SafeClosetError,
    termimad::{
        minimad::{Alignment, Composite},
        Area,
    },
};

#[derive(Default)]
pub struct TitleView {
    area: Area,
}

impl View for TitleView {
    type State = AppState;

    fn set_available_area(&mut self, area: Area) {
        self.area = area;
    }
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut AppState,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        let path = state.open_closet.path().to_string_lossy();
        let md = format!(" **SafeCloset** ` ` {} ` ` {} ", &path, state_info(state));
        let composite = Composite::from_inline(&md);
        self.go_to(w, self.area.left, self.area.top)?;
        let width = self.area.width as usize;
        app_skin.title
            .write_composite_fill(w, composite, width, Alignment::Unspecified)?;
        Ok(())
    }
}

fn state_info(state: &AppState) -> &'static str {
    match &state.drawer_state {
        None => {
            if state.open_closet.just_created() && state.created_drawers == 0 {
                "new closet"
            } else {
                "no open drawer"
            }
        }
        Some(ds) => {
            if ds.touched() {
                "*unsaved changes*"
            } else {
                ""
            }
        }
    }
}

