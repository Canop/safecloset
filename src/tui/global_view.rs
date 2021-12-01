use {
    super::*,
    crate::error::SafeClosetError,
    termimad::Area,
};

/// The view covering the whole terminal
#[derive(Default)]
pub struct GlobalView {
    area: Area,
    title: TitleView,
    content: ContentView,
    status: StatusView,
}

impl View for GlobalView {
    type State = AppState;

    fn set_available_area(&mut self, area: Area) {
        self.area = area;
        self.title.set_available_area(Area::new(0, 0, self.area.width, 1));
        self.content.set_available_area(Area::new(0, 1, self.area.width, self.area.height - 2));
        self.status.set_available_area(Area::new(0, self.area.height - 1, self.area.width, 1));
    }
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut AppState,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.title.draw(w, state, app_skin)?;
        self.content.draw(w, state, app_skin)?;
        self.status.draw(w, state, app_skin)?;
        w.flush()?;
        Ok(())
    }
}
