use crate::error::*;

pub struct Lexer<I> {
    chars: I,
    lookahead: Option<char>,
    line: usize,
    buffer: String,
}

impl<I> Lexer<I> {
    pub fn new(chars: I) -> Self {
        Lexer {
            chars,
            lookahead: Some(' '),
            line: 1,
            buffer: String::new(),
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl<I: Iterator<Item = char>> Lexer<I> {
    fn advance(&mut self) {
        if let Some(c) = self.lookahead {
            if c == '\n' {
                self.line += 1;
            }

            self.lookahead = self.chars.next();
        }
    }

    fn skip_to_newline(&mut self) {
        while let Some(c) = self.lookahead {
            if c == '\n' {
                break;
            } else {
                self.advance();
            }
        }
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.lookahead {
            if c == '#' {
                self.skip_to_newline();
            } else if !c.is_whitespace() {
                break;
            } else {
                self.advance();
            }
        }
    }

    fn collect_backslash_escape(&mut self) -> Result<()> {
        self.advance();
        if let Some(c) = self.lookahead {
            match c {
                '\n' => {
                    self.buffer.push(' ');
                    self.skip_ws();
                    Ok(())
                }

                '\"' => {
                    self.buffer.push('\"');
                    self.advance();
                    Ok(())
                }

                '\\' => {
                    self.buffer.push('\\');
                    self.advance();
                    Ok(())
                }

                c => {
                    let mut tok = String::new();
                    tok.push('\\');
                    tok.push(c);
                    illegal_token(tok, self.line)
                }
            }
        } else {
            unexpected_eof(self.line)
        }
    }

    fn collect_to_quote(&mut self) -> Result<()> {
        while let Some(c) = self.lookahead {
            match c {
                '\"' => {
                    self.advance();
                    return Ok(());
                }

                '\\' => {
                    self.collect_backslash_escape()?;
                }

                '\n' => {
                    return unterminated_string(self.line);
                }

                c => {
                    self.buffer.push(c);
                    self.advance();
                }
            }
        }

        unterminated_string(self.line)
    }

    fn collect_to_ws(&mut self) {
        while let Some(c) = self.lookahead {
            if c.is_whitespace() {
                break;
            }

            self.buffer.push(c);
            self.advance();
        }
    }

    pub fn scan(&mut self) -> Result<Option<String>> {
        self.buffer.clear();
        self.skip_ws();

        if let Some(c) = self.lookahead {
            if c == '\"' {
                self.advance();
                self.collect_to_quote()?;
            } else {
                self.collect_to_ws();
            }

            Ok(Some(self.buffer.clone()))
        } else {
            Ok(None)
        }
    }
}
