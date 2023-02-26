use {
    super::*,
    crate::error::SafeClosetError,
    termimad::Area,
};

/// A part of the screen.
///
/// Note that this isn't a general purpose TUI framework, it's only
/// suitable for this application
pub trait View<State>: Default {
    /// set the outside area. This view may take it wholly or partially
    fn set_available_area(
        &mut self,
        area: Area,
    );

    /// Render the view in its area
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut State, // mutable to allow adapt to terminal size changes
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError>;
}
