use std::{
    collections::{HashMap, HashSet},
    io,
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
            text: &model.text,
            token_lens: &model.token_lens,
        }
    }

    pub fn create_view<P>(&mut self, path: Option<P>) -> Result<ViewId, io::Error>
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
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
        self.views.insert(view_id, View { model_id });
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
                    .map(|string| TokenLen {
                        len: string.len(),
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
                token_lens: tokens,
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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ModelId(usize);

#[derive(Debug)]
struct Model {
    view_ids: HashSet<ViewId>,
    path: Option<PathBuf>,
    text: Vec<String>,
    token_lens: Vec<Vec<TokenLen>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TokenLen {
    pub len: usize,
    pub kind: TokenKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenKind {
    Whitespace,
    Unknown,
}

#[derive(Debug)]
pub struct Context<'a> {
    text: &'a [String],
    token_lens: &'a [Vec<TokenLen>],
}

impl<'a> Context<'a> {
    pub fn lines(&self) -> Lines<'a> {
        Lines {
            text: self.text.iter(),
            token_lens: self.token_lens.iter(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lines<'a> {
    text: slice::Iter<'a, String>,
    token_lens: slice::Iter<'a, Vec<TokenLen>>,
}

impl<'a> Iterator for Lines<'a> {
    type Item = Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Line {
            text: self.text.next()?,
            token_lens: self.token_lens.next()?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Line<'a> {
    text: &'a str,
    token_lens: &'a [TokenLen],
}

impl<'a> Line<'a> {
    pub fn tokens(&self) -> Tokens<'a> {
        Tokens {
            text: &self.text,
            token_lens: self.token_lens.iter(),
            position: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tokens<'a> {
    text: &'a str,
    position: usize,
    token_lens: slice::Iter<'a, TokenLen>,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token_len = self.token_lens.next()?;
        let position = self.position;
        self.position += token_len.len;
        Some(Token {
            text: &self.text[position..][..token_len.len],
            kind: token_len.kind,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: TokenKind,
}
