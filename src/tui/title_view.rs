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
    fn get_area(&self) -> &Area {
        &self.area
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
        self.go_to_line(w, 0)?;
        let width = self.area.width as usize;
        app_skin.help
            .write_composite_fill(w, composite, width, Alignment::Unspecified)?;
        Ok(())
    }
}

fn state_info(state: &AppState) -> &'static str {
    match &state.drawer_state {
        DrawerState::NoneOpen => {
            if state.open_closet.just_created() && state.created_drawers == 0 {
                "new closet"
            } else {
                "no open drawer"
            }
        }
        DrawerState::DrawerEdit(des) => {
            if des.touched() {
                "*unsaved changes*"
            } else {
                ""
            }
        }
    }
}

