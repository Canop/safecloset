use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::style::Color,
    termimad::Area,
};

/// The view covering the whole terminal
pub struct GlobalView {
    area: Area,
    title: TitleView,
    content: ContentView,
    status: StatusView,
}

impl Default for GlobalView {
    fn default() -> Self {
        Self {
            area: Area::uninitialized(),
            title: TitleView::default(),
            content: ContentView::default(),
            status: StatusView::default(),
        }
    }
}

impl View for GlobalView {
    fn set_area(&mut self, area: Area) {
        self.area = area;
        self.title.set_area(Area::new(0, 0, self.area.width, 1));
        self.content
            .set_area(Area::new(0, 1, self.area.width, self.area.height - 2));
        self.status
            .set_area(Area::new(0, self.area.height - 1, self.area.width, 1));
    }
    fn get_area(&self) -> &Area {
        &self.area
    }
    fn bg(&self) -> Color {
        self.content.bg()
    }

    fn draw(&mut self, w: &mut W, state: &mut AppState) -> Result<(), SafeClosetError> {
        debug!("global draw");
        self.title.draw(w, state)?;
        self.content.draw(w, state)?;
        self.status.draw(w, state)?;
        w.flush()?;
        Ok(())
    }
}
