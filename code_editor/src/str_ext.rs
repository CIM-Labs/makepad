use std::iter::{Chain, Once};

pub trait StrExt {
    fn is_grapheme_boundary(&self, index: usize) -> bool;

    fn next_grapheme_boundary(&self, index: usize) -> Option<usize>;

    fn graphemes(&self) -> Graphemes<'_>;

    fn grapheme_indices(&self) -> GraphemeIndices<'_>;

    fn split_at_whitespace_boundaries(&self) -> SplitAtWhitespaceBoundaries<'_>;
}

impl StrExt for str {
    fn is_grapheme_boundary(&self, index: usize) -> bool {
        self.is_char_boundary(index)
    }

    fn next_grapheme_boundary(&self, index: usize) -> Option<usize> {
        if index == self.len() {
            return None;
        }
        let mut index = index;
        loop {
            index += 1;
            if self.is_grapheme_boundary(index) {
                return Some(index);
            }
        }
    }

    fn graphemes(&self) -> Graphemes<'_> {
        Graphemes { string: self }
    }

    fn grapheme_indices(&self) -> GraphemeIndices<'_> {
        GraphemeIndices {
            start: self.as_ptr() as usize,
            graphemes: self.graphemes(),
        }
    }

    fn split_at_whitespace_boundaries(&self) -> SplitAtWhitespaceBoundaries<'_> {
        SplitAtWhitespaceBoundaries { string: self }
    }
}

#[derive(Clone, Debug)]
pub struct Graphemes<'a> {
    string: &'a str,
}

impl<'a> Iterator for Graphemes<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.string.next_grapheme_boundary(0)?;
        let (string_0, string_1) = self.string.split_at(index);
        self.string = string_1;
        Some(string_0)
    }
}

#[derive(Clone, Debug)]
pub struct GraphemeIndices<'a> {
    start: usize,
    graphemes: Graphemes<'a>,
}

impl<'a> Iterator for GraphemeIndices<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let grapheme = self.graphemes.next()?;
        Some((grapheme.as_ptr() as usize - self.start, grapheme))
    }
}

#[derive(Clone, Debug)]
pub struct SplitAtWhitespaceBoundaries<'a> {
    string: &'a str,
}

impl<'a> Iterator for SplitAtWhitespaceBoundaries<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.string.is_empty() {
            return None;
        }
        let mut prev_grapheme_is_whitespace = None;
        let (string_0, string_1) = self.string.split_at(
            self.string
                .grapheme_indices()
                .find_map(|(index, next_grapheme)| {
                    let next_grapheme_is_whitespace =
                        next_grapheme.chars().all(|char| char.is_whitespace());
                    let is_whitespace_boundary =
                        prev_grapheme_is_whitespace.map_or(false, |prev_grapheme_is_whitespace| {
                            prev_grapheme_is_whitespace != next_grapheme_is_whitespace
                        });
                    prev_grapheme_is_whitespace = Some(next_grapheme_is_whitespace);
                    if is_whitespace_boundary {
                        Some(index)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| self.string.len()),
        );
        self.string = string_1;
        Some(string_0)
    }
}

#[derive(Clone, Debug)]
pub struct SplitAtIndices<'a, I> {
    string: &'a str,
    start: usize,
    indices: Chain<I, Once<usize>>,
}

impl<'a, I> Iterator for SplitAtIndices<'a, I>
where
    I: Iterator<Item = usize>,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices.next()?;
        let start = self.start;
        self.start = index;
        Some(&self.string[start..index])
    }
}
