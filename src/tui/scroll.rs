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
    fn to_entries(
        self,
        entries_count: usize,
        page_entries_count: usize,
    ) -> i32 {
        match self {
            Self::Top => -(entries_count as i32),
            Self::Bottom => entries_count as i32,
            Self::Lines(n) => n,
            Self::Pages(n) => n * page_entries_count as i32,
        }
    }
    /// compute the new scroll value, which is a number of entries
    pub fn apply(
        self,
        scroll: usize,
        entries_count: usize,
        page_entries_count: usize,
    ) -> usize {
        (scroll as i32 + self.to_entries(entries_count, page_entries_count))
            .min(entries_count as i32 - 1)
            .max(0) as usize
    }
}
