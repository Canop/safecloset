use {
    crate::search::NameMatch,
    minimad::Alignment,
    std::io::Write,
    termimad::*,
};

pub struct MatchedString<'a> {
    pub name_match: Option<NameMatch>,
    pub string: &'a str,
    pub base_style: &'a CompoundStyle,
    pub match_style: &'a CompoundStyle,
    pub display_width: Option<usize>,
    pub align: Alignment,
}

impl<'a, 'w> MatchedString<'a> {
    pub fn new(
        name_match: Option<NameMatch>,
        string: &'a str,
        base_style: &'a CompoundStyle,
        match_style: &'a CompoundStyle,
    ) -> Self {
        Self {
            name_match,
            string,
            base_style,
            match_style,
            display_width: None,
            align: Alignment::Left,
        }
    }
    pub fn queue_on<W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
    ) -> Result<(), termimad::Error> {
        if let Some(m) = &self.name_match {
            let mut pos_idx: usize = 0;
            let mut combined_style = self.base_style.clone();
            combined_style.overwrite_with(self.match_style);
            let mut right_filling = 0;
            let mut s = self.string;
            if let Some(dw) = self.display_width {
                let w = unicode_width::UnicodeWidthStr::width(s);
                #[allow(clippy::comparison_chain)]
                if w > dw {
                    let (count_bytes, _) = StrFit::count_fitting(s, dw);
                    s = &s[0..count_bytes];
                } else if w < dw {
                    match self.align {
                        Alignment::Right => {
                            cw.repeat(self.base_style, &SPACE_FILLING, dw - w)?;
                        }
                        Alignment::Center => {
                            right_filling = (dw - w) / 2;
                            cw.repeat(self.base_style, &SPACE_FILLING, dw - w - right_filling)?;
                        }
                        _ => {
                            right_filling = dw - w;
                        }
                    }
                }
            }
            // we might call queue_char more than allowed but that's okay
            // because the cropwriter will crop them
            for (cand_idx, cand_char) in s.chars().enumerate() {
                if pos_idx < m.pos.len() && m.pos[pos_idx] == cand_idx {
                    cw.queue_char(&combined_style, cand_char)?;
                    pos_idx += 1;
                } else {
                    cw.queue_char(self.base_style, cand_char)?;
                }
            }
            if right_filling > 0 {
                cw.repeat(self.base_style, &SPACE_FILLING, right_filling)?;
            }
        } else if let Some(w) = self.display_width {
            match self.align {
                Alignment::Center => {
                    cw.queue_str(self.base_style, &format!("{:^w$}", self.string))?;
                }
                Alignment::Right => {
                    cw.queue_str(self.base_style, &format!("{:>w$}", self.string))?;
                }
                _ => {
                    cw.queue_str(self.base_style, &format!("{:<w$}", self.string))?;
                }
            }
        } else {
            cw.queue_str(self.base_style, self.string)?;
        }
        Ok(())
    }
}
