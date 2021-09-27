use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::style::Color,
    minimad::{Alignment, Composite},
    termimad::*,
};

/// Renders on most of the screen:
/// - drawer creation and open dialogs
/// - drawer content
pub struct ContentView {
    area: Area,
    skin: ContentSkin,
}


impl Default for ContentView {
    fn default() -> Self {
        Self {
            area: Area::default(),
            skin: ContentSkin::new(),
        }
    }
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

static MD_CREATE_TOP_DRAWER: &str = r#"Type the passphrase for the new top level drawer:"#;
static MD_CREATE_DEEP_DRAWER: &str = r#"Type the passphrase for this deep drawer (to create a top level drawer, cancel then close the drawer you're in):"#;
static MD_OPEN_DRAWER: &str = r#"Type the passphrase of the drawer you want to open:"#;
static MD_HIDDEN_CHARS: &str = r#"Characters are hidden. Type *^h* to toggle visibility."#;
static MD_VISIBLE_CHARS: &str = r#"Characters are visible. Type *^h* to hide them."#;

impl View for ContentView {
    fn set_area(&mut self, area: Area) {
        self.area = area;
    }
    fn get_area(&self) -> &Area {
        &self.area
    }

    fn bg(&self) -> Color {
        gray(2)
    }

    fn draw(&mut self, w: &mut W, state: &mut AppState) -> Result<(), SafeClosetError> {
        self.clear(w)?;
        if let Some(help_state) = &mut state.help {
            help_state.set_area(self.area.clone());
            help_state.draw(w)?;
        } else {
            match &mut state.drawer_state {
                DrawerState::NoneOpen => {
                    if state.open_closet.just_created() && state.created_drawers == 0 {
                        self.skin.md.write_in_area_on(w, MD_NEW_CLOSET, &self.area)?;
                    } else {
                        self.skin.md.write_in_area_on(w, MD_NO_DRAWER_OPEN, &self.area)?;
                    }
                }
                DrawerState::DrawerCreation(PasswordInputState { input }) => {
                    if state.open_closet.depth() > 0 {
                        self.draw_password_input( w, input, MD_CREATE_DEEP_DRAWER)?;
                    } else {
                        self.draw_password_input( w, input, MD_CREATE_TOP_DRAWER)?;
                    }
                }
                DrawerState::DrawerOpening(PasswordInputState { input }) => {
                    self.draw_password_input(w, input, MD_OPEN_DRAWER)?;
                }
                DrawerState::DrawerEdit(des) => {
                    self.draw_drawer(w, des)?;
                }
            }
        }
        Ok(())
    }
}

impl ContentView {
    fn draw_password_input(
        &mut self,
        w: &mut W,
        input: &mut InputField,
        introduction: &str,
    ) -> Result<(), SafeClosetError> {
        self.go_to_line(w, 3)?;
        self.skin.md.write_inline_on(w, introduction)?;
        input.change_area(0, 5, self.area.width);
        input.display_on(w)?;
        self.go_to_line(w, 7)?;
        let s = if input.password_mode {
            MD_HIDDEN_CHARS
        } else {
            MD_VISIBLE_CHARS
        };
        self.skin.md.write_inline_on(w, s)?;
        Ok(())
    }
    fn draw_drawer(
        &mut self,
        w: &mut W,
        des: &mut DrawerEditState,
    ) -> Result<(), SafeClosetError> {
        if des.drawer.content.entries.is_empty() {
            self.skin.md.write_in_area_on(w, MD_EMPTY_DRAWER, &self.area)?;
            return Ok(());
        }
        if self.area.height < 5 || self.area.width < 20 {
            warn!("Terminal too small to render drawer content");
            self.skin.md.write_in_area_on(w, "*too small*", &self.area)?;
            return Ok(());
        }
        // entries area
        des.update_drawing_layout(&self.area);
        let layout = des.layout();
        let scrollbar = des.scrollbar();
        let name_width = layout.name_width as usize;
        let value_left = name_width + 2; // 1 for selection mark, one for '|'
        let value_width = layout.lines_area.width as usize - value_left;
        let tbl_style = self.skin.tbl_style(false);
        let normal_style = self.skin.txt_style(false);
        // -- header
        self.go_to_line(w, 1)?;
        tbl_style.queue_str(w, &"─".repeat(name_width + 1))?;
        tbl_style.queue_str(w, "┬")?;
        let value_header_width = if scrollbar.is_some() {
            value_width + 1
        } else {
            value_width
        };
        tbl_style.queue_str(w, &"─".repeat(value_header_width))?;
        self.go_to_line(w, 2)?;
        if des.focus.is_search() {
            normal_style.queue_str(w, "/")?;
            des.search.input.change_area(1, 2, layout.name_width);
            des.search.input.display_on(w)?;
        } else if des.search.has_content() {
            normal_style.queue_str(w, "/")?;
            let (fitted, width) = StrFit::make_string(
                &des.search.input.get_content(),
                name_width,
            );
            normal_style.queue_str(w, fitted)?;
            if width < name_width {
                tbl_style.queue_str(w, &" ".repeat(name_width - width))?;
            }
        } else {
            self.skin.md.write_composite_fill(
                w,
                Composite::from_inline("**name**"),
                name_width + 1,
                Alignment::Center,
            )?;
        }
        tbl_style.queue_str(w, "│")?;
        self.skin.md.write_composite_fill(
            w,
            Composite::from_inline("**value**"),
            value_width,
            Alignment::Center,
        )?;
        self.go_to_line(w, 3)?;
        tbl_style.queue_str(w, &"─".repeat(name_width + 1))?;
        tbl_style.queue_str(w, "┼")?;
        tbl_style.queue_str(w, &"─".repeat(value_width + 1))?;
        // -- entries
        let scrollbar_style = match &des.focus {
            DrawerFocus::NameEdit { .. } | DrawerFocus::ValueEdit { .. } => {
                &self.skin.unsel_scrollbar
            }
            _ => &self.skin.md.scrollbar
        };
        let mut line = des.scroll;
        let mut empty_lines = 0;
        let area = &layout.lines_area;
        for y in area.top..=area.bottom() {
            self.go_to_line(w, y)?;
            if empty_lines > 0 {
                SPACE_FILLING.queue_styled(w, tbl_style, name_width + 1)?;
                tbl_style.queue_str(w, "│")?;
                empty_lines -= 1;
                // we skip the value area, to not overwrite it
            } else if let Some((idx, name_match)) = des.listed_entry(line) {
                let entry = &des.drawer.content.entries[idx];
                let is_best = des.has_best_search(line);
                let focus = &mut des.focus;
                // - selection mark
                if is_best {
                    self.skin.char_match.queue_str(w, "▶")?;
                } else if focus.line() == Some(line) {
                    self.skin.md.write_inline_on(w, "▶")?;
                } else {
                    self.skin.md.write_inline_on(w, " ")?;
                }
                // - name field
                if let Some(input) = focus.name_input(line) {
                    input.change_area(1, y, layout.name_width);
                    input.display_on(w)?;
                } else {
                    let mut cw = CropWriter::new(w, name_width);
                    let selected = is_best || focus.is_name_selected(line);
                    let txt_style = self.skin.txt_style(selected);
                    let ms = MatchedString::new(
                        name_match,
                        &entry.name,
                        txt_style,
                        self.skin.match_style(selected),
                    );
                    ms.queue_on(&mut cw)?;
                    cw.fill_with_space(txt_style)?;
                }
                // - separator
                tbl_style.queue_str(w, "│")?;
                // - value field
                if let Some(input) = focus.value_input(line) {
                    let h = layout.value_height_addition as u16 + 1;
                    empty_lines = layout.value_height_addition;
                    let value_area = Area::new(value_left as u16, y, value_width as u16, h);
                    input.set_area(value_area);
                    input.display_on(w)?;
                } else if focus.is_value_selected(line) {
                    if layout.value_height_addition > 0 {
                        // if there are several lines, we adopt the wrapping mode of termimad for
                        // a prettier result
                        let h = layout.value_height_addition as u16 + 1;
                        empty_lines = layout.value_height_addition;
                        let value_area = Area::new(value_left as u16, y, value_width as u16, h);
                        let text = self.skin.sel_md.area_text(&entry.value, &value_area);
                        let mut text_view = TextView::from(&value_area, &text);
                        text_view.show_scrollbar = true;
                        text_view.write_on(w)?;
                    } else {
                        let first_line = entry.value.split('\n').next().unwrap();
                        self.skin.sel_md.write_composite_fill(
                            w,
                            Composite::from_inline(first_line),
                            value_width,
                            Alignment::Left,
                        )?;
                    }
                } else if des.drawer.content.settings.hide_values {
                    tbl_style.queue_str(w, &"▦".repeat(value_width as usize))?;
                } else {
                    let first_line = entry.value.split('\n').next().unwrap();
                    self.skin.md.write_composite_fill(
                        w,
                        Composite::from_inline(first_line),
                        value_width,
                        Alignment::Left,
                    )?;
                }
                line += 1;
            }
            // - scrollbar
            if let Some((stop, sbottom)) = scrollbar {
                self.go_to(w, area.width, y)?;
                if stop <= y && y <= sbottom {
                    scrollbar_style.thumb.queue(w)?;
                } else {
                    scrollbar_style.track.queue(w)?;
                }
            }
        }
        Ok(())
    }
}
