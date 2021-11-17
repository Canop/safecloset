use {
    super::*,
    crossterm::event::KeyEvent,
    minimad::Text,
    termimad::*,
};

#[derive(Debug)]
pub struct Help {
    area: Area,
    scroll: usize,
    text: Text<'static>,
}

impl Default for Help {
    fn default() -> Self {
        Self {
            scroll: 0,
            area: Area::default(),
            text: help_text(),
        }
    }
}

impl Help {
    pub fn set_available_area(&mut self, area: Area) {
        self.area = area;
    }
    pub fn apply_key_event(
        &mut self,
        key: KeyEvent,
    ) -> bool {
        // the only events we're interested into are the ones which impact the
        // scroll position so we create a text view and ask it after the event
        // handling what's the new scroll
        let fmt_text = FmtText::from_text(
            termimad::get_default_skin(),
            self.text.clone(),
            Some((self.area.width - 1) as usize),
        );
        let mut text_view = TextView::from(&self.area, &fmt_text);
        text_view.set_scroll(self.scroll);
        if text_view.apply_key_event(key) {
            self.scroll = text_view.scroll;
            true
        } else {
            false
        }
    }
    pub fn draw(
        &mut self,
        w: &mut W,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        let fmt_text = FmtText::from_text(
            &app_skin.help,
            self.text.clone(),
            Some((self.area.width - 1) as usize),
        );
        let mut text_view = TextView::from(&self.area, &fmt_text);
        text_view.set_scroll(self.scroll);
        text_view.write_on(w)?;
        Ok(())
    }
}
