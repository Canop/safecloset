use {
    super::*,
    crossterm::{
        event::KeyEvent,
        style::Color,
    },
    minimad::Text,
    termimad::*,
};

#[derive(Debug)]
pub struct HelpState {
    scroll: usize,
    area: Area,
    text: Text<'static>,
    skin: MadSkin,
}

impl Default for HelpState {
    fn default() -> Self {
        let mut skin = MadSkin::default();
        skin.set_bg(gray(2));
        skin.set_fg(ansi(230));
        skin.set_headers_fg(ansi(222));
        skin.italic = CompoundStyle::with_fg(Color::AnsiValue(222));
        Self {
            scroll: 0,
            area: Area::default(),
            text: help_text(),
            skin,
        }
    }
}

impl HelpState {
    pub fn set_area(&mut self, area: Area) {
        self.area = area;
    }
    pub fn apply_key_event(
        &mut self,
        key: KeyEvent,
    ) {
        // the only events we're interested into are the ones which impact the
        // scroll position so we create a text view and ask it after the event
        // handling what's the new scroll
        let fmt_text = FmtText::from_text(
            &self.skin,
            self.text.clone(),
            Some((self.area.width - 1) as usize),
        );
        let mut text_view = TextView::from(&self.area, &fmt_text);
        text_view.set_scroll(self.scroll);
        text_view.apply_key_event(key);
        self.scroll = text_view.scroll;
    }
    pub fn draw(
        &mut self,
        w: &mut W,
    ) -> Result<(), SafeClosetError> {
        let fmt_text = FmtText::from_text(
            &self.skin,
            self.text.clone(),
            Some((self.area.width - 1) as usize),
        );
        let mut text_view = TextView::from(&self.area, &fmt_text);
        text_view.set_scroll(self.scroll);
        text_view.write_on(w)?;
        Ok(())
    }
}
