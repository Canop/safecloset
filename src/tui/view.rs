use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::cursor,
    termimad::Area,
};

/// A part of the screen.
///
/// Note that this isn't a general purpose TUI framework, it's only
/// suitable for this application
pub trait View: Default {

    type State;

    /// set the outside area. This view may take it wholly or partially
    fn set_available_area(&mut self, area: Area);

    fn get_area(&self) -> &Area;

    /// Render the view in its area
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut Self::State, // mutable to allow adapt to terminal size changes
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError>;

    fn go_to(&self, w: &mut W, x: u16, y: u16) -> Result<(), SafeClosetError> {
        w.queue(cursor::MoveTo(x, y))?;
        Ok(())
    }

    fn go_to_line(&self, w: &mut W, y: u16) -> Result<(), SafeClosetError> {
        w.queue(cursor::MoveTo(self.get_area().left, y))?;
        Ok(())
    }

}
