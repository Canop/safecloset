use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::style::{Color, Color::*, SetBackgroundColor},
    minimad::{Alignment, Composite},
    termimad::{gray, Area, InputField, MadSkin, StrFit},
};

/// Renders on most of the screen:
/// - drawer creation and open dialogs
/// - drawer content
pub struct ContentView {
    area: Area,
    skins: ContentSkins,
}

pub struct ContentSkins {
    /// skin for the selected cell
    sel_cell: MadSkin,

    /// skin for non selected cell
    cell: MadSkin,

    /// skin for the structural parts
    normal: MadSkin,
}

impl ContentSkins {
    pub fn new() -> Self {
        let mut normal = MadSkin::default();
        normal.paragraph.set_fgbg(AnsiValue(230), gray(2));
        normal.table.set_fgbg(gray(8), gray(2));
        let mut cell = normal.clone();
        cell.table.set_fgbg(AnsiValue(230), gray(2));
        let mut sel_cell = cell.clone();
        sel_cell.paragraph.set_bg(gray(6));
        Self {
            sel_cell,
            cell,
            normal,
        }
    }
}

impl Default for ContentView {
    fn default() -> Self {
        Self {
            area: Area::uninitialized(),
            skins: ContentSkins::new(),
        }
    }
}

static MD_NO_DRAWER_OPEN: &str = r#"
Hit *n* to create a new drawer.

Hit *o* to open an existing one.
"#;

static MD_EMPTY_DRAWER: &str = r#"
This drawer is still empty.

Hit *n* to create a new entry.
"#;

static MD_CREATE_DRAWER: &str = r#"Type the passphrase for the new drawer:"#;
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
        match &mut state.drawer_state {
            DrawerState::NoneOpen => {
                self.skins
                    .normal
                    .write_in_area_on(w, MD_NO_DRAWER_OPEN, &self.area)?;
            }
            DrawerState::DrawerCreation(PasswordInputState { input }) => {
                self.draw_password_input(w, input, MD_CREATE_DRAWER)?;
            }
            DrawerState::DrawerOpening(PasswordInputState { input }) => {
                self.draw_password_input(w, input, MD_OPEN_DRAWER)?;
            }
            DrawerState::DrawerEdit(des) => {
                self.draw_drawer(w, des)?;
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
        self.skins.normal.write_inline_on(w, introduction)?;
        input.change_area(0, 5, self.area.width);
        input.display_on(w)?;
        self.go_to_line(w, 7)?;
        let s = if input.password_mode {
            MD_HIDDEN_CHARS
        } else {
            MD_VISIBLE_CHARS
        };
        self.skins.normal.write_inline_on(w, s)?;
        Ok(())
    }
    fn draw_drawer(
        &mut self,
        w: &mut W,
        des: &mut DrawerEditState,
    ) -> Result<(), SafeClosetError> {
        if des.drawer.entries.is_empty() {
            self.skins
                .normal
                .write_in_area_on(w, MD_EMPTY_DRAWER, &self.area)?;
            return Ok(());
        }
        if self.area.height < 5 || self.area.width < 20 {
            warn!("Terminal too small to render drawer content");
            self.skins
                .normal
                .write_in_area_on(w, "*too small*", &self.area)?;
            return Ok(());
        }
        let normal_style = &self.skins.normal.paragraph.compound_style;
        // entries area
        let mut area = Area::new(0, self.area.top + 3, self.area.width, self.area.height - 3);
        des.set_page_height(area.height as usize);
        let scrollbar = area.scrollbar(des.scroll as i32, des.listed_entries_count() as i32);
        if scrollbar.is_some() {
            area.width -= 1;
        }
        let name_width_u16 = (area.width / 3).min(30);
        let name_width = name_width_u16 as usize; // I must change termimad to use only usize
        let value_left = name_width + 2;
        let value_width = area.width as usize - value_left;
        let tbl_style = &self.skins.normal.table.compound_style;
        // -- header
        self.go_to_line(w, 1)?;
        tbl_style.queue_str(w, &"─".repeat(name_width + 1))?;
        tbl_style.queue_str(w, "┬")?;
        tbl_style.queue_str(w, &"─".repeat(value_width + 1))?;
        self.go_to_line(w, 2)?;
        if matches!(&des.focus, DrawerFocus::SearchEdit) {
            normal_style.queue_str(w, "/")?;
            des.search.input.change_area(1, 2, name_width_u16);
            des.search.input.display_on(w)?;
        } else if des.search.has_content() {
            normal_style.queue_str(w, "/")?;
            let (fitted, width) = StrFit::make_string(&des.search.input.get_content(), name_width.into());
            normal_style.queue_str(w, fitted)?;
            if width < name_width {
                tbl_style.queue_str(w, &" ".repeat(name_width - width))?;
            }
        } else {
            self.skins.normal.write_composite_fill(
                w,
                Composite::from_inline("**name**"),
                name_width + 1,
                Alignment::Center,
            )?;
        }
        tbl_style.queue_str(w, "│")?;
        self.skins.normal.write_composite_fill(
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
        let mut line = des.scroll;
        for iy in 0..area.height {
            let y = iy + area.top;
            w.queue(SetBackgroundColor(self.bg()))?;
            self.go_to_line(w, y)?;
            self.clear_line(w)?;
            if let Some((idx, name_match)) = des.listed_entry(line) {
                let entry = &des.drawer.entries[idx];
                let focus = &mut des.focus;
                // - selection mark
                if focus.line() == Some(line) {
                    self.skins.normal.write_inline_on(w, "▶")?;
                } else {
                    self.skins.normal.write_inline_on(w, " ")?;
                }
                // - name field
                if let Some(input) = focus.name_input(line) {
                    input.change_area(1, y, name_width_u16);
                    input.display_on(w)?;
                } else {
                    let skin = if focus.is_name_selected(line) {
                        &self.skins.sel_cell
                    } else {
                        &self.skins.cell
                    };
                    skin.write_composite_fill(
                        w,
                        Composite::from_inline(&entry.name),
                        name_width.into(),
                        Alignment::Left,
                    )?;
                }
                // - separator
                tbl_style.queue_str(w, "│")?;
                // - value field
                if let Some(input) = focus.value_input(line) {
                    input.change_area(value_left as u16, y, value_width as u16);
                    input.display_on(w)?;
                } else if focus.is_value_selected(line) {
                    self.skins.sel_cell.write_composite_fill(
                        w,
                        Composite::from_inline(&entry.value),
                        value_width.into(),
                        Alignment::Left,
                    )?;
                } else if des.drawer.settings.hide_values {
                    tbl_style.queue_str(w, &"▦".repeat(value_width as usize))?;
                } else {
                    self.skins.cell.write_composite_fill(
                        w,
                        Composite::from_inline(&entry.value),
                        value_width.into(),
                        Alignment::Left,
                    )?;
                }
                // - scrollbar
                if is_thumb(y.into(), scrollbar) {
                    self.skins.normal.scrollbar.thumb.queue(w)?;
                }
                line += 1;
            }
        }
        Ok(())
    }
}
