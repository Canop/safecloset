use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::{
        cursor,
        style::{Color, SetBackgroundColor},
        terminal,
    },
    termimad::Area,
};

/// A part of the screen.
///
/// Note that this isn't a general purpose TUI framework, it's only
/// suitable for this application
pub trait View: Default {

    fn set_area(&mut self, area: Area);

    fn get_area(&self) -> &Area;

    fn width(&self) -> usize {
        self.get_area().width as usize
    }

    fn bg(&self) -> Color;

    /// Render the view in its area
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut AppState, // mutable to allow adapt to terminal size changes
    ) -> Result<(), SafeClosetError>;

    fn go_to(&self, w: &mut W, x: u16, y: u16) -> Result<(), SafeClosetError> {
        w.queue(cursor::MoveTo(x, y))?;
        Ok(())
    }

    fn go_to_line(&self, w: &mut W, y: u16) -> Result<(), SafeClosetError> {
        w.queue(cursor::MoveTo(self.get_area().left, y))?;
        Ok(())
    }

    /// Clear the whole area (and everything to the right)
    fn clear(&self, w: &mut W) -> Result<(), SafeClosetError> {
        let area = self.get_area();
        w.queue(SetBackgroundColor(self.bg()))?;
        for y in area.top..area.top + area.height {
            self.go_to_line(w, y)?;
            self.clear_line(w)?;
        }
        Ok(())
    }

    /// Clear from the cursor to end of line, whatever the area
    fn clear_line(&self, w: &mut W) -> Result<(), SafeClosetError> {
        w.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
        Ok(())
    }
}
