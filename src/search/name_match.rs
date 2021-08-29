use {
    super::Pos,
};

/// A NameMatch is a positive result of pattern matching inside
/// a filename or subpath
#[derive(Debug, Clone)]
pub struct NameMatch {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Pos, // positions of the matching chars
}

impl NameMatch {
    // cut the name match in two parts by recomputing the pos
    // arrays
    pub fn cut_after(&mut self, chars_count: usize) -> Self {
        let mut tail = Self {
            score: self.score,
            pos: Vec::new(),
        };
        let idx = self.pos.iter().position(|&p| p >= chars_count);
        if let Some(idx) = idx {
            for p in self.pos.drain(idx..) {
                tail.pos.push(p - chars_count);
            }
        }
        tail
    }
}

