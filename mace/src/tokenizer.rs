pub enum State {
    Initial(InitialState),
    BlockCommentTail(BlockCommentTailState),
    DoubleQuotedStringTail(DoubleQuotedStringTailState),
    RawDoubleQuotedStringTail(RawDoubleQuotedStringTailState),
}

impl Default for State {
    fn default() -> State {
        State::Initial(InitialState)
    }
}

impl State {
    pub fn next(self, cursor: &mut Cursor) -> (State, Option<Token>) {
        if cursor.peek(0) == '\0' {
            return (self, None);
        }
        let start = cursor.index;
        let (next_state, kind) = match self {
            State::Initial(state) => state.next(cursor),
            State::BlockCommentTail(state) => state.next(cursor),
            State::DoubleQuotedStringTail(state) => state.next(cursor),
            State::RawDoubleQuotedStringTail(state) => state.next(cursor),
        };
        let end = cursor.index;
        assert!(start < end);
        (
            next_state,
            Some(Token {
                len: end - start,
                kind,
            })
        )
    }
}

pub struct InitialState;

impl InitialState {
    fn next(self, cursor: &mut Cursor<'_>) -> (State, TokenKind) {
        match (cursor.peek(0), cursor.peek(1), cursor.peek(2)) {
            ('/', '*', _) => self.block_comment(cursor),
            ('"', _, _) => self.string(cursor),
            ('r', '#', '"') | ('r', '#', '#') => self.raw_string(cursor),
            (ch, _, _) if ch.is_identifier_start() => self.identifier_or_keyword(cursor),
            _ => {
                cursor.skip(1);
                (State::Initial(InitialState), TokenKind::Unknown)
            }
        }
    }

    fn block_comment(self, cursor: &mut Cursor<'_>) -> (State, TokenKind) {
        debug_assert!(cursor.peek(0) == '/' && cursor.peek(1) == '*');
        cursor.skip(2);
        BlockCommentTailState { depth: 0 }.next(cursor)
    }

    fn identifier_or_keyword(self, cursor: &mut Cursor) -> (State, TokenKind) {
        debug_assert!(cursor.peek(0).is_identifier_start());
        match cursor.peek(0) {
            'a' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'b' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("stract", cursor)
                    }
                    's' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'y' => self.identifier_or_keyword_tail("nc", cursor),
                            _ => self.identifier_or_keyword_tail("", cursor),
                        }
                    }
                    'w' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("ait", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'b' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'e' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("come", cursor)
                    }
                    'o' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("x", cursor)
                    }
                    'r' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("reak", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'c' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'o' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'n' => {
                                cursor.skip(1);
                                match cursor.peek(0) {
                                    's' => {
                                        cursor.skip(1);
                                        self.identifier_or_keyword_tail("t", cursor)
                                    }
                                    't' => {
                                        cursor.skip(1);
                                        self.identifier_or_keyword_tail("inue", cursor)
                                    }
                                    _ => self.identifier_tail(cursor),
                                }
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    'r' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("ate", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'd' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'o' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("", cursor)
                    }
                    'y' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("n", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'e' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'l' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("se", cursor)
                    }
                    'n' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("um", cursor)
                    }
                    'x' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("tern", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'f' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'a' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("lse", cursor)
                    }
                    'i' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("nal", cursor)
                    }
                    'n' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("", cursor)
                    }
                    'o' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("r", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'i' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'f' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("", cursor)
                    }
                    'm' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("pl", cursor)
                    }
                    'n' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'l' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'e' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("t", cursor)
                    }
                    'o' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("op", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'm' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'a' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'c' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("ro", cursor)
                            }
                            't' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("ch", cursor)
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    'o' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'd' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("", cursor)
                            }
                            'v' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("e", cursor)
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    'u' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("t", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'o' => {
                cursor.skip(1);
                self.identifier_or_keyword_tail("verride", cursor)
            }
            'p' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'r' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("iv", cursor)
                    }
                    'u' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("b", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'r' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'e' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'f' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("", cursor)
                            }
                            't' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("urn", cursor)
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            's' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'e' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("lf", cursor)
                    }
                    't' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'a' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("tic", cursor)
                            }
                            'r' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("uct", cursor)
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    'u' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("per", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            't' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'r' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'a' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("it", cursor)
                            }
                            'u' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("e", cursor)
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    'y' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'p' => {
                                cursor.skip(1);
                                match cursor.peek(0) {
                                    'e' => {
                                        cursor.skip(1);
                                        match cursor.peek(0) {
                                            'o' => {
                                                cursor.skip(1);
                                                self.identifier_or_keyword_tail("f", cursor)
                                            }
                                            _ => self.identifier_or_keyword_tail("", cursor),
                                        }
                                    }
                                    _ => self.identifier_tail(cursor),
                                }
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'u' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'n' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            's' => {
                                cursor.skip(1);
                                match cursor.peek(0) {
                                    'a' => {
                                        cursor.skip(1);
                                        self.identifier_or_keyword_tail("fe", cursor)
                                    }
                                    'i' => {
                                        cursor.skip(1);
                                        self.identifier_or_keyword_tail("zed", cursor)
                                    }
                                    _ => self.identifier_tail(cursor),
                                }
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    's' => {
                        cursor.skip(1);
                        self.identifier_or_keyword_tail("e", cursor)
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'v' => {
                cursor.skip(1);
                self.identifier_or_keyword_tail("irtual", cursor)
            }
            'w' => {
                cursor.skip(1);
                match cursor.peek(0) {
                    'h' => {
                        cursor.skip(1);
                        match cursor.peek(0) {
                            'e' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("re", cursor)
                            }
                            'i' => {
                                cursor.skip(1);
                                self.identifier_or_keyword_tail("le", cursor)
                            }
                            _ => self.identifier_tail(cursor),
                        }
                    }
                    _ => self.identifier_tail(cursor),
                }
            }
            'y' => {
                cursor.skip(1);
                self.identifier_or_keyword_tail("ield", cursor)
            }
            _ => self.identifier_tail(cursor),
        }
    }

    fn identifier_or_keyword_tail(self, string: &str, cursor: &mut Cursor) -> (State, TokenKind) {
        for expected in string.chars() {
            if !cursor.skip_if(|actual| actual == expected) {
                return (State::Initial(InitialState), TokenKind::Identifier);
            }
        }
        if cursor.peek(0).is_identifier_continue() {
            cursor.skip(1);
            return self.identifier_tail(cursor);
        }
        (State::Initial(InitialState), TokenKind::Keyword)
    }

    fn identifier_tail(self, cursor: &mut Cursor) -> (State, TokenKind) {
        while cursor.skip_if(|ch| ch.is_identifier_continue()) {}
        (State::Initial(InitialState), TokenKind::Identifier)
    }

    fn string(self, cursor: &mut Cursor) -> (State, TokenKind) {
        self.double_quoted_string(cursor)
    }

    fn double_quoted_string(self, cursor: &mut Cursor) -> (State, TokenKind) {
        debug_assert!(cursor.peek(0) == '"');
        cursor.skip(1);
        DoubleQuotedStringTailState.next(cursor)
    }

    fn raw_string(self, cursor: &mut Cursor) -> (State, TokenKind) {
        debug_assert!(cursor.peek(0) == 'r');
        cursor.skip(1);
        self.raw_double_quoted_string(cursor)
    }

    fn raw_double_quoted_string(self, cursor: &mut Cursor) -> (State, TokenKind) {
        let mut start_hash_count = 0;
        while cursor.skip_if(|ch| ch == '#') {
            start_hash_count += 1;
        }
        RawDoubleQuotedStringTailState { start_hash_count }.next(cursor)
    }
}

pub struct BlockCommentTailState { depth: usize }

impl BlockCommentTailState {
    fn next(self, cursor: &mut Cursor<'_>) -> (State, TokenKind) {
        let mut state = self;
        loop {
            match (cursor.peek(0), cursor.peek(1)) {
                ('/', '*') => {
                    cursor.skip(2);
                    state.depth += 1;
                }
                ('*', '/') => {
                    cursor.skip(2);
                    if state.depth == 0 {
                        break (State::Initial(InitialState), TokenKind::Comment);
                    }
                    state.depth -= 1;
                }
                ('\0', _) => {
                    break (State::BlockCommentTail(state), TokenKind::Comment);
                }
                _ => cursor.skip(1),
            }
        }

    }
}

pub struct DoubleQuotedStringTailState;

impl DoubleQuotedStringTailState {
    fn next(self, cursor: &mut Cursor<'_>) -> (State, TokenKind) {
        loop {
            match (cursor.peek(0), cursor.peek(1)) {
                ('"', _) => {
                    cursor.skip(1);
                    cursor.skip_suffix();
                    break (State::Initial(InitialState), TokenKind::String);
                }
                ('\0', _) => {
                    break (State::DoubleQuotedStringTail(DoubleQuotedStringTailState), TokenKind::String);
                }
                ('\\', '"') => cursor.skip(2),
                _ => cursor.skip(1),
            }
        }
    }
}

pub struct RawDoubleQuotedStringTailState { start_hash_count: usize }

impl RawDoubleQuotedStringTailState {
    fn next(self, cursor: &mut Cursor<'_>) -> (State, TokenKind) {
        loop {
            match cursor.peek(0) {
                '"' => {
                    cursor.skip(1);
                    let mut end_hash_count = 0;
                    while end_hash_count < self.start_hash_count && cursor.skip_if(|ch| ch == '#') {
                        end_hash_count += 1;
                    }
                    if end_hash_count == self.start_hash_count {
                        cursor.skip_suffix();
                        break (State::Initial(InitialState), TokenKind::String);
                    }
                }
                '\0' => {
                    break (State::RawDoubleQuotedStringTail(self), TokenKind::String);
                }
                _ => cursor.skip(1),
            }
        }

    }
}

pub struct Cursor<'a> {
    chars: &'a [char],
    index: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(chars: &'a [char]) -> Cursor<'a> {
        Cursor {
            chars,
            index: 0
        }
    }

    fn peek(&self, index: usize) -> char {
        self.chars.get(self.index + index).cloned().unwrap_or('\0')
    }

    fn skip(&mut self, count: usize) {
        self.index += count;
    }

    fn skip_if<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(char) -> bool,
    {
        if predicate(self.peek(0)) {
            self.skip(1);
            true
        } else {
            false
        }
    }

    fn skip_suffix(&mut self) {
        if self.peek(0).is_identifier_start() {
            self.skip(1);
            while self.skip_if(|ch| ch.is_identifier_continue()) {}
        }
    }

}

#[derive(Debug)]
pub struct Token {
    pub len: usize,
    pub kind: TokenKind,
}

#[derive(Debug)]
pub enum TokenKind {
    Comment,
    Identifier,
    Keyword,
    String,
    Unknown,
}

trait CharExt {
    fn is_identifier_start(self) -> bool;

    fn is_identifier_continue(self) -> bool;
}

impl CharExt for char {
    fn is_identifier_start(self) -> bool {
        match self {
            'A'..='Z' | '_' | 'a'..='z' => true,
            _ => false,
        }
    }

    fn is_identifier_continue(self) -> bool {
        match self {
            '0'..='9' | 'A'..='Z' | '_' | 'a'..='z' => true,
            _ => false,
        }
    }
}