use termimad::Area;

#[derive(Clone, Default)]
pub struct DrawerDrawingLayout {
    /// the area containing the lines, without the header
    pub lines_area: Area,

    pub name_width: u16,

    /// heights of values, excluding entries filtered out by search
    pub value_heights_by_line: Vec<usize>,

    pub content_height: usize,

    pub has_scrollbar: bool,
}

impl DrawerDrawingLayout {
    pub fn is_in_name_column(
        &self,
        x: u16,
    ) -> bool {
        x <= self.name_width
    }
    pub fn value_width(&self) -> usize {
        let name_width = self.name_width as usize;
        let value_left = name_width + 2; // 1 for selection mark, one for '|'
        self.lines_area.width as usize - value_left
    }
}
