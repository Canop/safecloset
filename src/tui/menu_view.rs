use {
    super::*,
    crossterm::style::Color,
    termimad::{
        *,
        minimad::*,
    },
};

/// The drawer of the menu, when there's one
///
/// (the view is kept between menus)
pub struct MenuView {
    pub area: Area, // the menu only covers part of this area if there aren't many actions
    skin: MadSkin,
    sel_skin: MadSkin,
}

impl Default for MenuView {
    fn default() -> Self {
        let skin = MadSkin {
            italic: CompoundStyle::with_fg(Color::AnsiValue(222)),
            ..Default::default()
        };
        let mut sel_skin = skin.clone();
        sel_skin.set_bg(gray(8));
        Self {
            area: Area::default(),
            skin,
            sel_skin,
        }
    }
}

impl View for MenuView {
    fn set_area(&mut self, area: Area) {
        self.area = area;
    }

    fn get_area(&self) -> &Area {
        &self.area
    }

    fn width(&self) -> usize {
        self.get_area().width as usize
    }

    fn bg(&self) -> Color {
        gray(4)
    }

    fn draw(
        &mut self,
        w: &mut W,
        state: &mut AppState,
    ) -> Result<(), SafeClosetError> {
        if let Some(menu) = state.menu.as_mut() {
            self.draw_menu(w, menu)?;
        }
        Ok(())
    }
}

impl MenuView {
    pub fn best_area_in(screen: &Area) -> Area {
        let sw2 = screen.width / 2;
        let w2 = 19.min(sw2-3); // menu half width
        let left = sw2 - w2;
        let h = (screen.height * 3 / 4).max(5).min(screen.height -4);
        let top = ((screen.height - h) / 3).max(1);
        Area::new(left, top, w2*2, h)
    }

    fn draw_menu(
        &mut self,
        w: &mut W,
        menu: &mut MenuState,
    ) -> Result<(), SafeClosetError> {
        let border_style = &self.skin.table.compound_style;
        let area = &self.area;
        let key_width = 3;
        let label_width = area.width as usize - key_width - 2;
        let mut y = area.top;
        self.go_to(w, area.left, y)?;
        border_style.queue_str(w, "┌")?;
        border_style.queue_str(w, &"─".repeat(area.width as usize - 2))?;
        border_style.queue_str(w, "┐")?;
        let h = (area.height - 2).min(menu.actions.len() as u16);
        let mut actions = menu.actions.iter().skip(menu.scroll);
        for i in 0..h {
            y += 1;
            if let Some(action) = actions.next() {
                let skin = if menu.selection == i as usize + menu.scroll {
                    &self.sel_skin
                } else {
                    &self.skin
                };
                self.go_to(w, area.left, y)?;
                border_style.queue_str(w, "│")?;
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
                border_style.queue_str(w, "│")?;
            } else {
                break;
            }
        }
        y += 1;
        self.go_to(w, area.left, y)?;
        border_style.queue_str(w, "└")?;
        border_style.queue_str(w, &"─".repeat(area.width as usize - 2))?;
        border_style.queue_str(w, "┘")?;
        Ok(())
    }
}

