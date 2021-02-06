use std::env::{self, VarError};
use std::ffi::{OsStr, OsString};

pub fn env_var<K: AsRef<OsStr>>(name: K) -> Option<OsString> {
    match env::var(name) {
        Ok(val) => Some(OsString::from(val)),
        Err(VarError::NotPresent) => None,
        Err(VarError::NotUnicode(val)) => Some(val),
    }
}

pub fn interpolate<S: AsRef<str>>(text: S) -> Option<OsString> {
    let mut res = Some(OsString::new());
    Lexer::new(text.as_ref().chars()).for_each(|tok| {
        let r = res.take();
        if let Some(mut r) = r {
            match tok {
                Token::Text(text) => {
                    r.push(text);
                    res = Some(r);
                }

                Token::Var(name) => {
                    if let Some(val) = env_var(name) {
                        if let Some(s) = val.to_str() {
                            if let Some(interp) = interpolate(s) {
                                r.push(interp);
                                res = Some(r);
                            }
                        } else {
                            r.push(val);
                            res = Some(r);
                        }
                    }
                }
            }
        }
    });

    res
}

fn is_id(c: char) -> bool {
    ('A'..='Z').contains(&c) || ('a'..='z').contains(&c) || ('0'..='9').contains(&c) || c == '_'
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerState {
    Text,
    Dollar,
    VarNameNoBrace,
    VarNameBrace,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text(String),
    Var(String),
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
        let mut input = input.into_iter();
        let lookahead = input.next();
        Lexer {
            input,
            lookahead,
            buffer: String::new(),
            state: LexerState::Text,
        }
    }

    fn get_next(&mut self) {
        self.lookahead = self.input.next();
    }

    fn clear_buf(&mut self) -> String {
        let contents = self.buffer.clone();
        self.buffer.clear();
        contents
    }

    fn advance_text(&mut self) -> Option<Token> {
        if let Some(c) = self.lookahead {
            if c == '$' {
                self.state = LexerState::Dollar;
            } else {
                self.buffer.push(c);
            }

            self.get_next();
            None
        } else {
            self.state = LexerState::End;

            if self.buffer.is_empty() {
                None
            } else {
                Some(Token::Text(self.clear_buf()))
            }
        }
    }

    fn advance_dollar(&mut self) -> Option<Token> {
        if let Some(c) = self.lookahead {
            if c == '$' {
                self.state = LexerState::Text;

                self.buffer.push('$');
                self.get_next();

                None
            } else if c == '{' {
                self.state = LexerState::VarNameBrace;
                self.get_next();

                if self.buffer.is_empty() {
                    None
                } else {
                    Some(Token::Text(self.clear_buf()))
                }
            } else if is_id(c) {
                self.state = LexerState::VarNameNoBrace;

                if self.buffer.is_empty() {
                    None
                } else {
                    Some(Token::Text(self.clear_buf()))
                }
            } else {
                self.state = LexerState::Text;

                self.buffer.push('$');

                None
            }
        } else {
            self.state = LexerState::End;

            self.buffer.push('$');

            if self.buffer.is_empty() {
                None
            } else {
                Some(Token::Text(self.clear_buf()))
            }
        }
    }

    fn advance_no_brace(&mut self) -> Option<Token> {
        if let Some(c) = self.lookahead {
            if is_id(c) {
                self.buffer.push(c);
                self.get_next();
                None
            } else {
                self.state = LexerState::Text;
                Some(Token::Var(self.clear_buf()))
            }
        } else {
            debug_assert!(!self.buffer.is_empty());
            self.state = LexerState::End;
            Some(Token::Var(self.clear_buf()))
        }
    }

    fn advance_brace(&mut self) -> Option<Token> {
        if let Some(c) = self.lookahead {
            self.get_next();

            if c == '}' {
                self.state = LexerState::Text;
                Some(Token::Var(self.clear_buf()))
            } else {
                self.buffer.push(c);
                None
            }
        } else {
            self.state = LexerState::End;
            let mut text = String::from("${");
            text.push_str(&self.clear_buf());
            Some(Token::Text(text))
        }
    }

    fn advance(&mut self) -> Option<Token> {
        match self.state {
            LexerState::Text => self.advance_text(),
            LexerState::Dollar => self.advance_dollar(),
            LexerState::VarNameNoBrace => self.advance_no_brace(),
            LexerState::VarNameBrace => self.advance_brace(),
            LexerState::End => None,
        }
    }

    fn scan(&mut self) -> Option<Token> {
        loop {
            if self.state == LexerState::End {
                return None;
            } else if let Some(tok) = self.advance() {
                return Some(tok);
            }
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.scan()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Mutex;

    lazy_static! {
        static ref ENV_LOCK: Mutex<()> = Mutex::new(());
    }

    fn text<S>(s: S) -> Token
    where
        String: From<S>,
    {
        Token::Text(String::from(s))
    }

    fn var<S>(s: S) -> Token
    where
        String: From<S>,
    {
        Token::Var(String::from(s))
    }

    fn simple_test(input: &str, expected: &[Token]) {
        let toks: Vec<_> = Lexer::new(input.chars()).collect();
        assert_eq!(&toks, expected);
    }

    #[test]
    fn just_text() {
        let input = "hey there just text here";
        simple_test(input, &[text(input)]);
    }

    #[test]
    fn just_var_no_brace() {
        let input = "$HOME_HERE";
        simple_test(input, &[var("HOME_HERE")]);
    }

    #[test]
    fn just_var_brace() {
        let input = "${HERES_A_VAR}";
        simple_test(input, &[var("HERES_A_VAR")]);
    }

    #[test]
    fn mixed_var_no_brace() {
        let input = "/home/$USER/what";
        simple_test(input, &[text("/home/"), var("USER"), text("/what")]);
    }

    #[test]
    fn mixed_var_brace() {
        let input = "/home/${USER}/what";
        simple_test(input, &[text("/home/"), var("USER"), text("/what")]);
    }

    #[test]
    fn escaped_dollar() {
        let input = "$$what";
        simple_test(input, &[text("$what")]);
    }

    #[test]
    fn trailing_dollar() {
        let input = "what$";
        simple_test(input, &[text(input)]);
    }

    #[test]
    fn unterminated_brace() {
        let input = "what${gives";
        simple_test(input, &[text("what"), text("${gives")]);
    }

    #[test]
    fn interpolate_vars_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("FOO", "bar");
        let input = "/home/$FOO/baz";
        let res = interpolate(input).unwrap();
        assert_eq!(res, "/home/bar/baz");
    }

    #[test]
    fn interpolate_vars_unset() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("FOO");
        let input = "/home/$FOO/baz";
        assert!(interpolate(input).is_none());
    }

    #[test]
    fn recursive_interpolation() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("FOO", "$BAR/$BAZ");
        env::set_var("BAR", "bar");
        env::set_var("BAZ", "baz");
        let input = "/home/$FOO";
        let res = interpolate(input).unwrap();
        assert_eq!(res, "/home/bar/baz");
    }

    #[test]
    fn recursive_interpolation_subvars_unset() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("FOO", "$BAR/$BAZ");
        env::set_var("BAR", "bar");
        env::remove_var("BAZ");
        let input = "/home/$FOO";
        assert!(interpolate(input).is_none());
    }
}
