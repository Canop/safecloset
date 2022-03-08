use {
    crate::{
        tui::*,
    },
    termimad::{
        *,
        minimad::*,
    },
};

/// The drawer of the menu
#[derive(Default)]
pub struct MenuView {
    available_area: Area,
}

impl View for MenuView {
    type State = MenuState;

    fn set_available_area(&mut self, available_area: Area) {
        if available_area != self.available_area {
            self.available_area = available_area;
        }
    }

    /// Draw the menu and set the area of all visible items in the state
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut MenuState,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        state.clear_item_areas();
        let skin = &app_skin.dialog;
        let border_colors = skin.md.table.compound_style.clone();
        let area = self.compute_area(state.items.len());
        let h = area.height as usize - 2; // internal height
        let scrollbar = compute_scrollbar(
            state.scroll,
            state.items.len(),
            h,
            area.top + 1,
        );
        state.fix_scroll(h);
        let mut rect = Rect::new(area.clone(), border_colors);
        rect.set_border_style(BORDER_STYLE_BLAND);
        rect.draw(w)?;
        let key_width = 3;
        let mut label_width = area.width as usize - key_width - 2;
        if scrollbar.is_some() {
            label_width -= 1;
        }
        let mut y = area.top;
        let mut items = state.items.iter_mut().skip(state.scroll);
        for i in 0..h {
            y += 1;
            if let Some(item) = items.next() {
                let item_area = Area::new(area.left + 1, y, area.width - 2, 1);
                let skin = if state.selection == i as usize + state.scroll {
                    &skin.sel_md
                } else {
                    &skin.md
                };
                w.go_to(item_area.left, y)?;
                skin.write_composite_fill(
                    w,
                    Composite::from_inline(item.action.label()),
                    label_width,
                    Alignment::Left,
                )?;
                let key_desc = item.action.key()
                    .map_or("".to_string(), |key| KEY_FORMAT.to_string(key));
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
    fn compute_area(&self, items_count: usize) -> Area {
        let screen = &self.available_area;
        let ideal_height = items_count as u16 + 2; // margin of 1
        let sw2 = screen.width / 2;
        let w2 = 19.min(sw2-3); // menu half width
        let left = sw2 - w2;
        let h = screen.height.min(ideal_height);
        let top = ((screen.height - h) * 3 / 5).max(1);
        Area::new(left, top, w2*2, h)
    }
}

