#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quote {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerState {
    Space,
    Text,
    Quote(Quote),
    Backslash,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Lexer<I> {
    input: I,
    lookahead: Option<char>,
    buffer: String,
    state: LexerState,
}

impl<I: Iterator<Item = char>> Lexer<I> {
    fn new<T: IntoIterator<IntoIter = I>>(input: T) -> Lexer<I> {
        Lexer {
            input: input.into_iter(),
            lookahead: Some(' '),
            buffer: String::new(),
            state: LexerState::Space,
        }
    }

    fn get_next(&mut self) {
        self.lookahead = self.input.next();
    }

    fn clear_buf(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            let contents = self.buffer.clone();
            self.buffer.clear();
            Some(contents)
        }
    }

    fn advance_space(&mut self) -> Option<String> {
        if let Some(c) = self.lookahead {
            if !c.is_whitespace() {
                self.state = match c {
                    '\"' => LexerState::Quote(Quote::Double),
                    '\'' => LexerState::Quote(Quote::Single),
                    '\\' => LexerState::Backslash,
                    c => {
                        self.buffer.push(c);
                        LexerState::Text
                    }
                };

                self.get_next();
            }
        } else {
            self.state = LexerState::End;
        }

        None
    }

    fn advance_text(&mut self) -> Option<String> {
        if let Some(c) = self.lookahead {
            let res = if c.is_whitespace() {
                self.state = LexerState::Space;
                self.clear_buf()
            } else {
                match c {
                    '\"' => {
                        self.state = LexerState::Quote(Quote::Double);
                    }

                    '\'' => {
                        self.state = LexerState::Quote(Quote::Single);
                    }

                    '\\' => {
                        self.state = LexerState::Backslash;
                    }

                    c => {
                        self.buffer.push(c);
                    }
                };

                None
            };

            self.get_next();
            res
        } else {
            self.state = LexerState::End;
            self.clear_buf()
        }
    }

    fn advance_quote(&mut self, quote: Quote) -> Option<String> {
        if let Some(c) = self.lookahead {
            match (c, quote) {
                ('\'', Quote::Single) | ('\"', Quote::Double) => {
                    self.state = LexerState::Text;
                }

                (c, _) => {
                    self.buffer.push(c);
                }
            };

            self.get_next();
            None
        } else {
            self.state = LexerState::End;
            self.clear_buf()
        }
    }

    fn advance_backslash(&mut self) -> Option<String> {
        if let Some(c) = self.lookahead {
            self.buffer.push(c);
            self.get_next();
            None
        } else {
            self.state = LexerState::End;
            self.buffer.push('\\');
            self.clear_buf()
        }
    }

    fn advance(&mut self) -> Option<String> {
        match self.state {
            LexerState::Space => self.advance_space(),
            LexerState::Text => self.advance_text(),
            LexerState::Quote(quote) => self.advance_quote(quote),
            LexerState::Backslash => self.advance_backslash(),
            LexerState::End => None,
        }
    }

    fn scan(&mut self) -> Option<String> {
        loop {
            if self.state == LexerState::End {
                return None;
            } else if let Some(s) = self.advance() {
                return Some(s);
            }
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.scan()
    }
}

#[cfg(test)]
mod test {
    // TODO
}
