use {
    crate::tui::*,
    crokey::crossterm::{
        queue,
        style::Print,
    },
    termimad::{
        minimad::*,
        *,
    },
};

/// The drawer of the menu
#[derive(Default)]
pub struct MenuView {
    available_area: Area,
}

impl<I: ToString + Clone> View<MenuState<I>> for MenuView {
    fn set_available_area(
        &mut self,
        available_area: Area,
    ) {
        if available_area != self.available_area {
            self.available_area = available_area;
        }
    }

    /// Draw the menu and set the area of all visible items in the state
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut MenuState<I>,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        state.clear_item_areas();
        let skin = &app_skin.dialog;
        let border_colors = skin.md.table.compound_style.clone();
        let area_width = self.compute_area_width();
        let mut intro_lines = Vec::new();
        let text_width = area_width - 2;
        let intro = state.intro.clone();
        let mut content_height = state.items.len();
        if let Some(intro) = &intro {
            let text = FmtText::from(&skin.md, intro, Some(text_width as usize));
            intro_lines = text.lines;
            content_height += intro_lines.len() + 1; // 1 for margin
        }
        let area = self.compute_area(content_height, area_width);
        let h = area.height as usize - 2; // internal height
        let scrollbar = compute_scrollbar(state.scroll, content_height, h, area.top + 1);
        state.fix_scroll(h);
        let mut rect = Rect::new(area.clone(), border_colors);
        rect.set_border_style(BORDER_STYLE_BLAND);
        rect.set_fill(true);
        rect.draw(w)?;
        let key_width = 3;
        let mut label_width = area.width as usize - key_width - 2;
        if scrollbar.is_some() {
            label_width -= 1;
        }
        let mut y = area.top;
        let mut items = state.items.iter_mut().enumerate().skip(state.scroll);
        for _ in 0..h {
            y += 1;
            if !intro_lines.is_empty() {
                let intro_line = intro_lines.remove(0);
                w.go_to(area.left + 1, y)?;
                let dl = DisplayableLine::new(&skin.md, &intro_line, Some(text_width as usize));
                queue!(w, Print(&dl))?;
                if intro_lines.is_empty() {
                    y += 1; // skip line for margin
                }
                continue;
            }
            if let Some((idx, item)) = items.next() {
                let item_area = Area::new(area.left + 1, y, area.width - 2, 1);
                let skin = if state.selection == idx {
                    &skin.sel_md
                } else {
                    &skin.md
                };
                w.go_to(item_area.left, y)?;
                skin.write_composite_fill(
                    w,
                    Composite::from_inline(&item.action.to_string()),
                    label_width,
                    Alignment::Left,
                )?;
                let key_desc = item
                    .key
                    .map_or(String::new(), |key| KEY_FORMAT.to_string(key));
                skin.write_composite_fill(
                    w,
                    mad_inline!("*$0", &key_desc),
                    key_width,
                    Alignment::Right,
                )?;
                item.area = Some(item_area);
            } else {
                break;
            }
            if let Some((stop, sbottom)) = scrollbar {
                w.go_to(area.right() - 2, y)?;
                if stop <= y && y <= sbottom {
                    skin.md.scrollbar.thumb.queue(w)?;
                } else {
                    skin.md.scrollbar.track.queue(w)?;
                }
            }
        }
        Ok(())
    }
}

impl MenuView {
    fn compute_area_width(&self) -> u16 {
        let screen = &self.available_area;
        let sw2 = screen.width / 2;
        let w2 = 24.min(sw2 - 3); // menu half width
        w2 * 2
    }
    fn compute_area(
        &self,
        content_height: usize,
        area_width: u16,
    ) -> Area {
        let screen = &self.available_area;
        let ideal_height = content_height as u16 + 2; // margin of 1
        let left = (screen.width - area_width) / 2;
        let h = screen.height.min(ideal_height);
        let top = ((screen.height - h) * 3 / 5).max(1);
        Area::new(left, top, area_width, h)
    }
}
