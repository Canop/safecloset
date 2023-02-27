use {
    super::*,
    crate::error::SafeClosetError,
    crokey::crossterm::{
        style::{
            Color,
            SetBackgroundColor,
        },
        terminal,
    },
    minimad::{
        Alignment,
        Composite,
    },
    termimad::*,
};

/// Renders on most of the screen:
/// - drawer creation and open dialogs
/// - drawer content
#[derive(Default)]
pub struct ContentView {
    area: Area,
}

static MD_NEW_CLOSET: &str = r#"
This is a new closet.

To store secrets, you must create at least a drawer.

This is done with the *^n* key combination (*control-n*).
"#;

static MD_NO_DRAWER_OPEN: &str = r#"
Hit *^n* to create a new drawer.

Hit *^o* to open an existing one.
"#;

static MD_EMPTY_DRAWER: &str = r#"
This drawer is still empty.

Hit the *n* key to create a new entry.
"#;

impl View for ContentView {
    type State = AppState;

    fn set_available_area(
        &mut self,
        area: Area,
    ) {
        self.area = area;
    }
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut AppState,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.clear(w)?;
        let faded = state.dialog.is_some();
        let skin = &app_skin.content;
        if let Some(des) = state.drawer_state.as_mut() {
            self.draw_drawer(w, des, faded, skin)?;
        } else {
            let styles = skin.styles(false, faded);
            if state.open_closet.just_created() && state.created_drawers == 0 {
                styles.md.write_in_area_on(w, MD_NEW_CLOSET, &self.area)?;
            } else {
                styles
                    .md
                    .write_in_area_on(w, MD_NO_DRAWER_OPEN, &self.area)?;
            }
        }
        match &mut state.dialog {
            Dialog::Help(help) => {
                help.set_available_area(self.area.clone());
                help.draw(w, app_skin)?;
            }
            Dialog::Menu(menu) => {
                menu.set_available_area(self.area.clone());
                menu.draw(w, app_skin)?;
            }
            Dialog::Password(password_dialog) => {
                password_dialog.view.set_available_area(self.area.clone());
                password_dialog.draw(w, app_skin)?;
            }
            Dialog::CommentsEditor(comments_editor) => {
                comments_editor.view.set_available_area(self.area.clone());
                comments_editor.draw(w, app_skin)?;
            }
            Dialog::None => {}
        }
        Ok(())
    }
}

impl ContentView {
    fn bg(&self) -> Color {
        gray(2)
    }
    fn get_area(&self) -> &Area {
        &self.area
    }
    /// Clear the whole area (and everything to the right)
    fn clear(
        &self,
        w: &mut W,
    ) -> Result<(), SafeClosetError> {
        let area = self.get_area();
        w.queue(SetBackgroundColor(self.bg()))?;
        let x = area.left;
        for y in area.top..area.top + area.height {
            w.go_to(x, y)?;
            self.clear_line(w)?;
        }
        Ok(())
    }
    /// Clear from the cursor to end of line, whatever the area
    fn clear_line(
        &self,
        w: &mut W,
    ) -> Result<(), SafeClosetError> {
        w.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
        Ok(())
    }
    fn draw_drawer(
        &mut self,
        w: &mut W,
        des: &mut DrawerState,
        faded: bool,
        skin: &ContentSkin,
    ) -> Result<(), SafeClosetError> {
        if des.drawer.content.entries.is_empty() {
            skin.styles(false, faded)
                .md
                .write_in_area_on(w, MD_EMPTY_DRAWER, &self.area)?;
            return Ok(());
        }
        if self.area.height < 7 || self.area.width < 20 {
            warn!("Terminal too small to render drawer content");
            skin.styles(false, faded)
                .md
                .write_in_area_on(w, "*terminal too small*", &self.area)?;
            return Ok(());
        }
        let x = self.area.left;
        // entries area
        des.update_drawing_layout(&self.area);
        let layout = des.layout();
        let scrollbar = des.scrollbar();
        let name_width = layout.name_width as usize;
        let value_left = name_width + 2; // 1 for selection mark, one for '|'
        let value_width = layout.value_width();
        let tbl_style = skin.tbl_style(false, faded);
        let txt_style = skin.txt_style(false, faded);
        // -- header
        w.go_to(x, 1)?;
        tbl_style.queue_str(w, &"─".repeat(name_width + 1))?;
        tbl_style.queue_str(w, "┬")?;
        let value_header_width = if scrollbar.is_some() {
            value_width + 1
        } else {
            value_width
        };
        tbl_style.queue_str(w, &"─".repeat(value_header_width))?;
        w.go_to(x, 2)?;
        if des.focus.is_search() {
            txt_style.queue_str(w, "/")?;
            des.search.input.change_area(1, 2, layout.name_width);
            des.search.input.display_on(w)?;
        } else if des.search.has_content() {
            txt_style.queue_str(w, "/")?;
            let (fitted, width) = StrFit::make_string(&des.search.input.get_content(), name_width);
            txt_style.queue_str(w, fitted)?;
            if width < name_width {
                tbl_style.queue_str(w, &" ".repeat(name_width - width))?;
            }
        } else {
            skin.styles(false, faded).md.write_composite_fill(
                w,
                Composite::from_inline("**name**"),
                name_width + 1,
                Alignment::Center,
            )?;
        }
        tbl_style.queue_str(w, "│")?;
        skin.styles(false, faded).md.write_composite_fill(
            w,
            Composite::from_inline("**value**"),
            value_width,
            Alignment::Center,
        )?;
        w.go_to(x, 3)?;
        tbl_style.queue_str(w, &"─".repeat(name_width + 1))?;
        tbl_style.queue_str(w, "┼")?;
        tbl_style.queue_str(w, &"─".repeat(value_width + 1))?;
        // -- entries
        let global_scrollbar_style =
            skin.scrollbar_style(false, faded || des.focus.is_entry_edit());
        let mut line = des.scroll;
        let mut empty_lines = 0; // number of names to skip
        let area = &layout.lines_area;
        let unsel_styles = skin.styles(false, faded);
        for y in area.top..=area.bottom() {
            w.go_to(x, y)?;
            if empty_lines > 0 {
                SPACE_FILLING.queue_styled(w, tbl_style, name_width + 1)?;
                tbl_style.queue_str(w, "│")?;
                empty_lines -= 1;
                // we skip the value area, to not overwrite it
            } else if let Some((idx, name_match)) = des.listed_entry(line) {
                let entry = &des.drawer.content.entries[idx];
                let value_height = layout.value_heights_by_line[line];
                let is_best = des.has_best_search(line);
                let focus = &mut des.focus;
                let faded = faded && !focus.is_line_pending_removal(line);
                empty_lines = value_height - 1;
                // - selection mark
                if is_best {
                    unsel_styles.char_match.queue_str(w, "▶")?;
                } else if focus.line() == Some(line) {
                    unsel_styles.md.write_inline_on(w, "▶")?;
                } else {
                    unsel_styles.md.write_inline_on(w, " ")?;
                }
                // - name field
                if let Some(input) = focus.name_input(line) {
                    input.change_area(1, y, layout.name_width);
                    input.display_on(w)?;
                } else {
                    let mut cw = CropWriter::new(w, name_width);
                    let selected = is_best || focus.is_name_selected(line);
                    let field_txt_style = skin.txt_style(selected, faded);
                    let ms = MatchedString::new(
                        name_match,
                        &entry.name,
                        field_txt_style,
                        skin.match_style(selected, faded),
                    );
                    ms.queue_on(&mut cw)?;
                    cw.fill_with_space(field_txt_style)?;
                }
                // - separator
                tbl_style.queue_str(w, "│")?;
                // - value field
                let value_area = Area::new(
                    value_left as u16,
                    y,
                    value_width as u16,
                    value_height as u16,
                );
                if let Some(input) = focus.value_input(line) {
                    input.set_area(value_area);
                    input.display_on(w)?;
                } else {
                    let selected = focus.is_value_selected(line);
                    let forced_open = selected || focus.is_line_pending_removal(line);
                    let hide_values = des.drawer.content.settings.hide_values;
                    let open_all_values = des.drawer.content.settings.open_all_values;
                    let (open, hidden) = if forced_open {
                        (true, false)
                    } else if hide_values {
                        (false, true)
                    } else if open_all_values {
                        (true, false)
                    } else {
                        (false, false)
                    };
                    if hidden {
                        skin.txt_style(false, true)
                            .queue_str(w, &"▦".repeat(value_width))?;
                    } else if open {
                        let styles = skin.styles(selected, faded);
                        let text = styles.md.area_text(&entry.value, &value_area);
                        let mut text_view = TextView::from(&value_area, &text);
                        text_view.show_scrollbar = true;
                        text_view.write_on(w)?;
                    } else {
                        let styles = skin.styles(selected, faded);
                        let first_line = entry.value.split('\n').next().unwrap();
                        styles.md.write_composite_fill(
                            w,
                            Composite::from_inline(first_line),
                            value_width,
                            Alignment::Left,
                        )?;
                    }
                }
                line += 1;
            }
            // - scrollbar
            if let Some((stop, sbottom)) = scrollbar {
                w.go_to(area.width, y)?;
                if stop <= y && y <= sbottom {
                    global_scrollbar_style.thumb.queue(w)?;
                } else {
                    global_scrollbar_style.track.queue(w)?;
                }
            }
        }
        Ok(())
    }
}
