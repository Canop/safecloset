#[derive(Debug, Clone, Copy)]
pub enum ScrollCommand {
    Top,
    Bottom,
    Lines(i32),
    Pages(i32),
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
}

impl ScrollCommand {
    fn to_lines(
        self,
        content_height: usize,
        page_height: usize,
    ) -> i32 {
        match self {
            Self::Top => -(content_height as i32),
            Self::Bottom => content_height as i32,
            Self::Lines(n) => n,
            Self::Pages(n) => n * page_height as i32,
        }
    }
    /// compute the new scroll value
    pub fn apply(
        self,
        scroll: usize,
        content_height: usize,
        page_height: usize,
    ) -> usize {
        if content_height > page_height {
            (scroll as i32 + self.to_lines(content_height, page_height))
                .min((content_height - page_height - 1) as i32)
                .max(0) as usize
        } else {
            0
        }
    }
}
