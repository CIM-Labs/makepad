use std::{
    collections::{HashMap, HashSet},
    io,
    ops::Range,
    path::{Path, PathBuf},
    slice,
};

#[derive(Debug)]
pub struct State {
    view_id: usize,
    views: HashMap<ViewId, View>,
    model_id: usize,
    models: HashMap<ModelId, Model>,
    model_ids_by_path: HashMap<PathBuf, ModelId>,
}

impl State {
    pub fn new() -> Self {
        Self {
            view_id: 0,
            views: HashMap::new(),
            model_id: 0,
            models: HashMap::new(),
            model_ids_by_path: HashMap::new(),
        }
    }

    pub fn context(&self, view_id: ViewId) -> Context<'_> {
        let view = &self.views[&view_id];
        let model = &self.models[&view.model_id];
        Context {
            selections: &view.selections,
            text: &model.text,
            tokens: &model.tokens,
        }
    }

    pub fn create_view<P>(&mut self, path: Option<P>) -> Result<ViewId, io::Error>
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let selections = vec![Selection {
            anchor: Position {
                line_index: 0,
                byte_index: 10,
            },
            cursor: Position {
                line_index: 10,
                byte_index: 4,
            }
        }, Selection {
            anchor: Position {
                line_index: 10,
                byte_index: 19,
            },
            cursor: Position {
                line_index: 15,
                byte_index: 0,
            }
        }];

        let model_id = if let Some(path) = path {
            if let Some(model_id) = self.model_ids_by_path.get(path.as_ref()).copied() {
                model_id
            } else {
                self.create_model(Some(path.into()))?
            }
        } else {
            self.create_model(None)?
        };

        let view_id = ViewId(self.view_id);
        self.view_id += 1;
        self.views.insert(
            view_id,
            View {
                selections,
                model_id,
            },
        );
        self.models
            .get_mut(&model_id)
            .unwrap()
            .view_ids
            .insert(view_id);
        Ok(view_id)
    }

    pub fn destroy_view(&mut self, view_id: ViewId) {
        let model_id = self.views[&view_id].model_id;
        let view_ids = &mut self.models.get_mut(&model_id).unwrap().view_ids;
        view_ids.remove(&view_id);
        if view_ids.is_empty() {
            self.destroy_model(model_id);
        }
        self.views.remove(&view_id);
    }

    fn create_model(&mut self, path: Option<PathBuf>) -> Result<ModelId, io::Error> {
        use {crate::StrExt, std::fs};

        let mut text: Vec<_> = if let Some(path) = &path {
            String::from_utf8_lossy(&fs::read(path)?).into_owned()
        } else {
            String::new()
        }
        .lines()
        .map(|string| string.to_string())
        .collect();
        if text.is_empty() {
            text.push(String::new());
        }

        let tokens = text
            .iter()
            .map(|text| {
                text.split_at_whitespace_boundaries()
                    .map(|string| Token {
                        byte_count: string.len(),
                        kind: if string.chars().next().unwrap().is_whitespace() {
                            TokenKind::Whitespace
                        } else {
                            TokenKind::Unknown
                        },
                    })
                    .collect()
            })
            .collect();

        let model_id = ModelId(self.model_id);
        self.model_id += 1;
        self.models.insert(
            model_id,
            Model {
                view_ids: HashSet::new(),
                path: path.clone(),
                text,
                tokens: tokens,
            },
        );
        if let Some(path) = path {
            self.model_ids_by_path.insert(path, model_id);
        }
        Ok(model_id)
    }

    fn destroy_model(&mut self, model_id: ModelId) {
        if let Some(path) = &self.models[&model_id].path {
            self.model_ids_by_path.remove(path);
        }
        self.models.remove(&model_id);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ViewId(usize);

#[derive(Debug)]
struct View {
    model_id: ModelId,
    selections: Vec<Selection>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Selection {
    anchor: Position,
    cursor: Position,
}

impl Selection {
    fn start(self) -> Position {
        self.anchor.min(self.cursor)
    }

    fn end(self) -> Position {
        self.anchor.max(self.cursor)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Position {
    pub line_index: usize,
    pub byte_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ModelId(usize);

#[derive(Debug)]
struct Model {
    view_ids: HashSet<ViewId>,
    path: Option<PathBuf>,
    text: Vec<String>,
    tokens: Vec<Vec<Token>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Token {
    byte_count: usize,
    kind: TokenKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenKind {
    Whitespace,
    Unknown,
}

#[derive(Debug)]
pub struct Context<'a> {
    selections: &'a [Selection],
    text: &'a [String],
    tokens: &'a [Vec<Token>],
}

impl<'a> Context<'a> {
    pub fn lines(&self) -> Lines<'a> {
        Lines {
            selections: self.selections,
            text: self.text.iter(),
            tokens: self.tokens.iter(),
            line_index: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lines<'a> {
    selections: &'a [Selection],
    text: slice::Iter<'a, String>,
    tokens: slice::Iter<'a, Vec<Token>>,
    line_index: usize,
}

impl<'a> Iterator for Lines<'a> {
    type Item = Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let end = self
            .selections
            .iter()
            .position(|selection| selection.start().line_index > self.line_index)
            .unwrap_or_else(|| self.selections.len());
        let start = if end > 0 && self.selections[end - 1].end().line_index > self.line_index {
            end - 1
        } else {
            end
        };
        let selections = &self.selections[..end];
        self.selections = &self.selections[start..];
        let line_index = self.line_index;
        self.line_index += 1;
        Some(Line {
            selections,
            text: self.text.next()?,
            tokens: self.tokens.next()?,
            line_index,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Line<'a> {
    selections: &'a [Selection],
    text: &'a str,
    tokens: &'a [Token],
    line_index: usize,
}

impl<'a> Line<'a> {
    pub fn selections(&self) -> Selections<'a> {
        Selections {
            selections: self.selections.iter(),
            text: &self.text,
            line_index: self.line_index,
        }
    }

    pub fn tokens(&self) -> Tokens<'a> {
        Tokens {
            text: &self.text,
            tokens: self.tokens.iter(),
            byte_index: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Selections<'a> {
    selections: slice::Iter<'a, Selection>,
    text: &'a str,
    line_index: usize,
}

impl<'a> Iterator for Selections<'a> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let selection = self.selections.next()?;
        let start_byte_index = if selection.start().line_index == self.line_index {
            selection.start().byte_index
        } else {
            0
        };
        let end_byte_index = if selection.end().line_index == self.line_index {
            selection.end().byte_index
        } else {
            self.text.len()
        };
        Some(start_byte_index..end_byte_index)
    }
}

#[derive(Clone, Debug)]
pub struct Tokens<'a> {
    text: &'a str,
    tokens: slice::Iter<'a, Token>,
    byte_index: usize,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = TokenRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.next()?;
        let byte_index = self.byte_index;
        self.byte_index += token.byte_count;
        Some(TokenRef {
            text: &self.text[byte_index..][..token.byte_count],
            kind: token.kind,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TokenRef<'a> {
    pub text: &'a str,
    pub kind: TokenKind,
}