use {
    termimad::Area,
};

#[derive(Clone, Default)]
pub struct DrawerDrawingLayout {
    /// the area containing the lines, without the header
    pub lines_area: Area,

    pub name_width: u16,

    /// the additional height of the selected line due to
    /// a selected value being several lines
    pub value_height_addition: usize,

    pub has_scrollbar: bool,
}

impl DrawerDrawingLayout {
    pub fn is_in_name_column(&self, x: u16) -> bool {
        x <= self.name_width
    }
    pub fn value_width(&self) -> usize {
        let name_width = self.name_width as usize;
        let value_left = name_width + 2; // 1 for selection mark, one for '|'
        self.lines_area.width as usize - value_left
    }
}


