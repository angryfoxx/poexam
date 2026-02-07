// SPDX-FileCopyrightText: 2026 Sébastien Helleu <flashcode@flashtux.org>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::c_format::get_index_end_c_format;

pub struct WordPos<'a> {
    s: &'a str,
    bytes: &'a [u8],
    len: usize,
    skip_c_format: bool,
    pos: usize,
}

impl<'a> WordPos<'a> {
    /// Create a new `WordPos` iterator.
    ///
    /// Argument `format` can be `c` or an empty string.
    pub fn new(s: &'a str, format: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();
        Self {
            s,
            bytes,
            len,
            skip_c_format: format == "c",
            pos: 0,
        }
    }
}

impl Iterator for WordPos<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let mut idx_start = None;
        let mut idx_end = None;
        while self.pos < self.len {
            // Skip C format.
            if self.skip_c_format && idx_start.is_none() && self.bytes[self.pos] == b'%' {
                self.pos += 1;
                if self.pos < self.len && self.bytes[self.pos] == b'%' {
                    self.pos += 1;
                } else {
                    self.pos = get_index_end_c_format(self.bytes, self.pos, self.len);
                }
                if self.pos >= self.len {
                    return None;
                }
            } else {
                match self.s[self.pos..].chars().next() {
                    Some(c) => {
                        let len_c = c.len_utf8();
                        if c.is_alphanumeric() || (idx_start.is_some() && c == '-') {
                            if idx_start.is_none() {
                                idx_start = Some(self.pos);
                            }
                            idx_end = Some(self.pos + len_c);
                        } else if idx_start.is_some() {
                            break;
                        }
                        self.pos += len_c;
                    }
                    None => return None,
                }
            }
        }
        match (idx_start, idx_end) {
            (Some(start), Some(end)) => Some((start, end)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let s = "";
        let pos: Vec<_> = WordPos::new(s, "").collect();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_punct() {
        let s = " ,.!? ";
        let pos: Vec<_> = WordPos::new(s, "").collect();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_c_format() {
        let s = "%05d";
        // Do not skip any format.
        let pos: Vec<_> = WordPos::new(s, "").collect();
        assert_eq!(pos, vec![(1, 4)]);
        assert_eq!(&s[pos[0].0..pos[0].1], "05d");
        // Skip C format.
        let pos: Vec<_> = WordPos::new(s, "c").collect();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_basic_ascii() {
        let s = "Hello, world! %llu test-word 42.";
        // Do not skip any format.
        let pos: Vec<_> = WordPos::new(s, "").collect();
        assert_eq!(pos, vec![(0, 5), (7, 12), (15, 18), (19, 28), (29, 31)]);
        assert_eq!(&s[pos[0].0..pos[0].1], "Hello");
        assert_eq!(&s[pos[1].0..pos[1].1], "world");
        assert_eq!(&s[pos[2].0..pos[2].1], "llu");
        assert_eq!(&s[pos[3].0..pos[3].1], "test-word");
        assert_eq!(&s[pos[4].0..pos[4].1], "42");
        // Skip C format.
        let pos: Vec<_> = WordPos::new(s, "c").collect();
        assert_eq!(pos, vec![(0, 5), (7, 12), (19, 28), (29, 31)]);
        assert_eq!(&s[pos[0].0..pos[0].1], "Hello");
        assert_eq!(&s[pos[1].0..pos[1].1], "world");
        assert_eq!(&s[pos[2].0..pos[2].1], "test-word");
        assert_eq!(&s[pos[3].0..pos[3].1], "42");
    }

    #[test]
    fn test_unicode() {
        let s = "héllo, мир! %lld 你好";
        let positions: Vec<_> = WordPos::new(s, "").collect();
        assert_eq!(positions, vec![(0, 6), (8, 14), (17, 20), (21, 27)]);
        assert_eq!(&s[positions[0].0..positions[0].1], "héllo");
        assert_eq!(&s[positions[1].0..positions[1].1], "мир");
        assert_eq!(&s[positions[2].0..positions[2].1], "lld");
        assert_eq!(&s[positions[3].0..positions[3].1], "你好");
    }
}
