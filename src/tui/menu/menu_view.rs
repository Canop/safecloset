use {
    crate::tui::*,
    termimad::{
        *,
        minimad::*,
    },
};

/// The drawer of the menu
#[derive(Default)]
pub struct MenuView {
    available_area: Area,
    area: Area, // the menu only covers part of this area if there aren't many actions
}

impl View for MenuView {
    type State = MenuState;

    fn set_available_area(&mut self, available_area: Area) {
        if available_area != self.available_area {
            self.available_area = available_area;
            self.area = MenuView::best_area_in(&self.available_area);
        }
    }

    fn get_area(&self) -> &Area {
        &self.area
    }

    fn draw(
        &mut self,
        w: &mut W,
        state: &mut MenuState,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        debug_assert!(self.area.width > 3);
        let skin = &app_skin.dialog;
        let border_colors = skin.md.table.compound_style.clone();
        let area = &self.area;
        let h = (area.height - 2).min(state.actions.len() as u16); // internal height
        let mut rect = Rect::new(area.clone(), border_colors);
        rect.set_border_style(BORDER_STYLE_BLAND);
        rect.area.height = h + 1;
        rect.draw(w)?;
        let key_width = 3;
        let label_width = area.width as usize - key_width - 2;
        let mut y = area.top;
        let mut actions = state.actions.iter().skip(state.scroll);
        for i in 0..h {
            y += 1;
            if let Some(action) = actions.next() {
                let skin = if state.selection == i as usize + state.scroll {
                    &skin.sel_md
                } else {
                    &skin.md
                };
                self.go_to(w, area.left+1, y)?;
                skin.write_composite_fill(
                    w,
                    Composite::from_inline(action.label()),
                    label_width,
                    Alignment::Left,
                )?;
                let key_desc = action.key()
                    .map_or("".to_string(), |key| key_event_desc(key));
                skin.write_composite_fill(
                    w,
                    mad_inline!("*$0", &key_desc),
                    key_width,
                    Alignment::Right,
                )?;
            } else {
                break;
            }
        }
        Ok(())
    }
}

impl MenuView {
    fn best_area_in(screen: &Area) -> Area {
        let sw2 = screen.width / 2;
        let w2 = 19.min(sw2-3); // menu half width
        let left = sw2 - w2;
        let h = (screen.height * 3 / 4).max(5).min(screen.height -4);
        let top = ((screen.height - h) / 3).max(1);
        Area::new(left, top, w2*2, h)
    }
}

