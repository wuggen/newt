use std::ffi::OsStr;
use std::process::Command;

pub fn command<S: AsRef<OsStr>>(line: S) -> Option<Command> {
    let chars = line.as_ref().to_str()?.chars();
    let mut words = Lexer::new(chars);

    let mut cmd = Command::new(words.next()?);
    cmd.args(words);
    Some(cmd)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quote {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrevState {
    Text,
    Quote(Quote),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerState {
    Space,
    Text,
    Quote(Quote),
    Backslash(PrevState),
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
                    '\\' => LexerState::Backslash(PrevState::Text),
                    c => {
                        self.buffer.push(c);
                        LexerState::Text
                    }
                };
            }

            self.get_next();
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
                        self.state = LexerState::Backslash(PrevState::Text);
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

                ('\\', quote) => {
                    self.state = LexerState::Backslash(PrevState::Quote(quote));
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

    fn advance_backslash(&mut self, prev_state: PrevState) -> Option<String> {
        if let Some(c) = self.lookahead {
            if prev_state == PrevState::Quote(Quote::Single) && c != '\'' {
                self.buffer.push('\\');
            }

            self.buffer.push(c);
            self.get_next();

            self.state = match prev_state {
                PrevState::Text => LexerState::Text,
                PrevState::Quote(quote) => LexerState::Quote(quote),
            };

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
            LexerState::Backslash(prev_state) => self.advance_backslash(prev_state),
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
    use super::*;

    fn test_for_expected(input: &str, expected: &[&str]) {
        let words: Vec<_> = Lexer::new(input.chars()).collect();
        assert_eq!(words, expected);
    }

    #[test]
    fn scan_basic() {
        test_for_expected("vim todo.txt", &["vim", "todo.txt"])
    }

    #[test]
    fn scan_single_quoted() {
        test_for_expected(r#"vim 'whats up boyyy'"#, &["vim", "whats up boyyy"]);
    }

    #[test]
    fn scan_double_quoted() {
        test_for_expected(r#"vim "whats up boyyy""#, &["vim", "whats up boyyy"]);
    }

    #[test]
    fn scan_mixed_quotes() {
        test_for_expected(
            r#"vim "what's an apostrophe""#,
            &["vim", "what's an apostrophe"],
        );
    }

    #[test]
    fn scan_backslash_space() {
        test_for_expected(r"vim\ up", &["vim up"]);
    }

    #[test]
    fn scan_backslash_quote() {
        test_for_expected(r#"vim\"nope\""#, &[r#"vim"nope""#]);
    }

    #[test]
    fn scan_backslash_space_at_beginning() {
        test_for_expected(r"vim \ who\ does\ this", &["vim", " who does this"]);
    }

    #[test]
    fn scan_single_quote_raw_backslash() {
        test_for_expected(r"'hey\ guess\ what'", &[r"hey\ guess\ what"]);
    }

    #[test]
    fn scan_double_quote_interpreted_backslash() {
        test_for_expected(r#""hey\ guess\ what""#, &["hey guess what"]);
    }

    #[test]
    fn scan_single_quote_escaped() {
        test_for_expected(r"'hey what\'s that'", &["hey what's that"]);
    }
}
