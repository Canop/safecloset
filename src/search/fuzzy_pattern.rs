//! This is copied from Broot
//! (there will probably be a common crate in the future)
use {
    super::NameMatch,
    std::fmt::{
        self,
        Write,
    },
};

// weights used in match score computing
const BONUS_MATCH: i32 = 50_000;
const BONUS_EXACT: i32 = 1_000;
const BONUS_START: i32 = 10;
const BONUS_START_WORD: i32 = 5;
const BONUS_CANDIDATE_LENGTH: i32 = -1; // per char
const BONUS_MATCH_LENGTH: i32 = -10; // per char of length of the match
const BONUS_NB_HOLES: i32 = -30; // there's also a max on that number
const BONUS_SINGLED_CHAR: i32 = -15; // when there's a char, neither first not last, isolated

/// A pattern for fuzzy matching
#[derive(Debug, Clone)]
pub struct FuzzyPattern {
    chars: Box<[char]>, // secularized characters
    max_nb_holes: usize,
}

impl fmt::Display for FuzzyPattern {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        for &c in self.chars.iter() {
            f.write_char(c)?
        }
        Ok(())
    }
}

enum MatchSearchResult {
    Perfect(NameMatch), // no need to test other positions
    Some(NameMatch),
    None,
}

fn is_word_separator(c: char) -> bool {
    matches!(c, '_' | ' ' | '-')
}

impl FuzzyPattern {
    /// build a pattern which will later be usable for fuzzy search.
    /// A pattern should be reused
    pub fn from(pat: &str) -> Self {
        let chars = secular::normalized_lower_lay_string(pat)
            .chars()
            .collect::<Vec<char>>()
            .into_boxed_slice();
        let max_nb_holes = match chars.len() {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 2,
            5 => 2,
            6 => 3,
            7 => 3,
            8 => 4,
            _ => chars.len() * 4 / 7,
        };
        FuzzyPattern {
            chars,
            max_nb_holes,
        }
    }

    fn tight_match_from_index(
        &self,
        cand_chars: &[char],
        start_idx: usize, // start index in candidate, in chars
    ) -> MatchSearchResult {
        let mut pos = vec![0; self.chars.len()]; // positions of matching chars in candidate
        let mut cand_idx = start_idx;
        let mut pat_idx = 0; // index both in self.chars and pos
        let mut in_hole = false;
        loop {
            if cand_chars[cand_idx] == self.chars[pat_idx] {
                pos[pat_idx] = cand_idx;
                if in_hole {
                    // We're no more in a hole.
                    // Let's look if we can bring back the chars before the hole
                    let mut rev_idx = 1;
                    loop {
                        if pat_idx < rev_idx {
                            break;
                        }
                        if cand_chars[cand_idx - rev_idx] == self.chars[pat_idx - rev_idx] {
                            // we move the pos forward
                            pos[pat_idx - rev_idx] = cand_idx - rev_idx;
                        } else {
                            break;
                        }
                        rev_idx += 1;
                    }
                    in_hole = false;
                }
                pat_idx += 1;
                if pat_idx == self.chars.len() {
                    break; // match, finished
                }
            } else {
                // there's a hole
                if cand_chars.len() - cand_idx <= self.chars.len() - pat_idx {
                    return MatchSearchResult::None;
                }
                in_hole = true;
            }
            cand_idx += 1;
        }
        let mut nb_holes = 0;
        let mut nb_singled_chars = 0;
        for idx in 1..pos.len() {
            if pos[idx] > 1 + pos[idx - 1] {
                nb_holes += 1;
                if idx > 1 && pos[idx - 1] > 1 + pos[idx - 2] {
                    // we improve a simple case: the one of a singleton which was created
                    // by pushing forward a char
                    if cand_chars[pos[idx - 2] + 1] == cand_chars[pos[idx - 1]] {
                        // in some cases we're really removing another singletons but
                        // let's forget this
                        pos[idx - 1] = pos[idx - 2] + 1;
                        nb_holes -= 1;
                    } else {
                        nb_singled_chars += 1;
                    }
                }
            }
        }
        if nb_holes > self.max_nb_holes {
            return MatchSearchResult::None;
        }
        let match_len = 1 + cand_idx - pos[0];
        let mut score = BONUS_MATCH;
        score += BONUS_CANDIDATE_LENGTH * (cand_chars.len() as i32);
        score += BONUS_SINGLED_CHAR * nb_singled_chars;
        score += BONUS_NB_HOLES * (nb_holes as i32);
        score += match_len as i32 * BONUS_MATCH_LENGTH;
        if pos[0] == 0 {
            score += BONUS_START + BONUS_START_WORD;
            if cand_chars.len() == self.chars.len() {
                score += BONUS_EXACT;
                return MatchSearchResult::Perfect(NameMatch { score, pos });
            }
        } else {
            let previous = cand_chars[pos[0] - 1];
            if is_word_separator(previous) {
                score += BONUS_START_WORD;
                if cand_chars.len() - pos[0] == self.chars.len() {
                    return MatchSearchResult::Perfect(NameMatch { score, pos });
                }
            }
        }
        MatchSearchResult::Some(NameMatch { score, pos })
    }

    /// return a match if the pattern can be found in the candidate string.
    /// The algorithm tries to return the best one. For example if you search
    /// "abc" in "ababca-abc", the returned match would be at the end.
    pub fn find(
        &self,
        candidate: &str,
    ) -> Option<NameMatch> {
        if candidate.len() < self.chars.len() {
            return None;
        }
        let mut cand_chars: Vec<char> = Vec::with_capacity(candidate.len());
        cand_chars.extend(candidate.chars().map(secular::lower_lay_char));
        if cand_chars.len() < self.chars.len() {
            return None;
        }
        let mut best_score = 0;
        let mut best_match: Option<NameMatch> = None;
        let n = cand_chars.len() - self.chars.len();
        for start_idx in 0..=n {
            if cand_chars[start_idx] == self.chars[0] {
                match self.tight_match_from_index(&cand_chars, start_idx) {
                    MatchSearchResult::Perfect(m) => {
                        return Some(m);
                    }
                    MatchSearchResult::Some(m) => {
                        if m.score > best_score {
                            best_score = m.score;
                            best_match = Some(m);
                        }
                        // we could make start_idx jump to pos[0] here
                        // but it doesn't improve the perfs (it's rare
                        // anyway to have pos[0] much greater than the
                        // start of the search)
                    }
                    _ => {}
                }
            }
        }
        best_match
    }
}
