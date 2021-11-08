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
        let faded = state.menu.is_some();
        if let Some(help_state) = &mut state.help {
            help_state.set_area(self.area.clone());
            help_state.draw(w)?;
        } else {
            match &mut state.drawer_state {
                DrawerState::NoneOpen => {
                    let styles = self.skin.styles(false, faded);
                    if state.open_closet.just_created() && state.created_drawers == 0 {
                        styles.md.write_in_area_on(w, MD_NEW_CLOSET, &self.area)?;
                    } else {
                        styles.md.write_in_area_on(w, MD_NO_DRAWER_OPEN, &self.area)?;
                    }
                }
                DrawerState::DrawerCreation(PasswordInputState { input }) => {
                    if state.open_closet.depth() > 0 {
                        self.draw_password_input( w, input, MD_CREATE_DEEP_DRAWER, faded)?;
                    } else {
                        self.draw_password_input( w, input, MD_CREATE_TOP_DRAWER, faded)?;
                    }
                }
                DrawerState::DrawerOpening(PasswordInputState { input }) => {
                    self.draw_password_input(w, input, MD_OPEN_DRAWER, faded)?;
                }
                DrawerState::DrawerEdit(des) => {
                    self.draw_drawer(w, des, faded)?;
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
        faded: bool,
    ) -> Result<(), SafeClosetError> {
        let styles = self.skin.styles(false, faded);
        self.go_to_line(w, 3)?;
        styles.md.write_inline_on(w, introduction)?;
        input.change_area(0, 5, self.area.width);
        input.display_on(w)?;
        self.go_to_line(w, 7)?;
        let s = if input.password_mode {
            MD_HIDDEN_CHARS
        } else {
            MD_VISIBLE_CHARS
        };
        styles.md.write_inline_on(w, s)?;
        Ok(())
    }
    fn draw_drawer(
        &mut self,
        w: &mut W,
        des: &mut DrawerEditState,
        faded: bool,
    ) -> Result<(), SafeClosetError> {
        if des.drawer.content.entries.is_empty() {
            self.skin.styles(false, faded)
                .md.write_in_area_on(w, MD_EMPTY_DRAWER, &self.area)?;
            return Ok(());
        }
        if self.area.height < 5 || self.area.width < 20 {
            warn!("Terminal too small to render drawer content");
            self.skin.styles(false, faded)
                .md.write_in_area_on(w, "*too small*", &self.area)?;
            return Ok(());
        }
        // entries area
        des.update_drawing_layout(&self.area);
        let layout = des.layout();
        let scrollbar = des.scrollbar();
        let name_width = layout.name_width as usize;
        let value_left = name_width + 2; // 1 for selection mark, one for '|'
        let value_width = layout.value_width();
        let tbl_style = self.skin.tbl_style(false, faded);
        let txt_style = self.skin.txt_style(false, faded);
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
            txt_style.queue_str(w, "/")?;
            des.search.input.change_area(1, 2, layout.name_width);
            des.search.input.display_on(w)?;
        } else if des.search.has_content() {
            txt_style.queue_str(w, "/")?;
            let (fitted, width) = StrFit::make_string(
                &des.search.input.get_content(),
                name_width,
            );
            txt_style.queue_str(w, fitted)?;
            if width < name_width {
                tbl_style.queue_str(w, &" ".repeat(name_width - width))?;
            }
        } else {
            self.skin.styles(false, faded).md.write_composite_fill(
                w,
                Composite::from_inline("**name**"),
                name_width + 1,
                Alignment::Center,
            )?;
        }
        tbl_style.queue_str(w, "│")?;
        self.skin.styles(false, faded).md.write_composite_fill(
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
        let global_scrollbar_style = self.skin
            .scrollbar_style(false, faded || des.focus.is_entry_edit());
        let mut line = des.scroll;
        let mut empty_lines = 0; // number of names to skip
        let area = &layout.lines_area;
        let unsel_styles = self.skin.styles(false, faded);
        for y in area.top..=area.bottom() {
            self.go_to_line(w, y)?;
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
                    let field_txt_style = self.skin.txt_style(selected, faded);
                    let ms = MatchedString::new(
                        name_match,
                        &entry.name,
                        field_txt_style,
                        self.skin.match_style(selected, faded),
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
                        self.skin.txt_style(false, true)
                            .queue_str(w, &"▦".repeat(value_width as usize))?;
                    } else if open {
                        let styles = self.skin.styles(selected, faded);
                        let text = styles.md.area_text(&entry.value, &value_area);
                        let mut text_view = TextView::from(&value_area, &text);
                        text_view.show_scrollbar = true;
                        text_view.write_on(w)?;
                    } else {
                        let styles = self.skin.styles(selected, faded);
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
                self.go_to(w, area.width, y)?;
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
