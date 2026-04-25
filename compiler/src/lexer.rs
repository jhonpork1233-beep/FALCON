use std::iter::Peekable;
use std::str::Chars;

/// Source location for a token: (line, column), both 1-indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),
    
    // Identifiers and keywords
    Identifier(String),
    Keyword(Keyword),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    Tilde, // Added Tilde
    Shl,
    Shr,
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    Pipe,       // | (closure parameter delimiter)
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    ColonColon, // ::
    Dot,
    DotDot, // ..
    Arrow, // ->
    FatArrow, // =>
    
    // Special
    Ampersand,
    Mut,
    Unsafe,
    Question, // ?
    Exclamation, // !
    Hash, // # (for attributes)
    
    // End of file
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Func,
    Let,
    Mut,
    If,
    Else,
    While,
    For,
    Loop,
    Break,
    Continue,
    Return,
    Match,
    Struct,
    Enum,
    Impl,
    Mod,
    Import,
    Pub,
    Const,
    True,
    False,
    As,
    Unsafe,
    Extern,
    Use,
    In,
    Trait,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current_char: Option<char>,
    position: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        let current_char = chars.next();
        Self {
            input: chars,
            current_char,
            position: 0,
            line: 1,
            col: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let (tokens, _spans) = self.tokenize_with_spans()?;
        Ok(tokens)
    }

    /// Tokenize and return both tokens and their source spans.
    pub fn tokenize_with_spans(&mut self) -> Result<(Vec<Token>, Vec<Span>), String> {
        let mut tokens = Vec::new();
        let mut spans = Vec::new();
        
        while self.current_char.is_some() {
            self.skip_whitespace();
            
            if self.current_char.is_none() {
                break;
            }
            
            // Check for comments before tokenizing
            if self.current_char == Some('/') {
                if self.peek() == Some('/') {
                    self.advance();
                    self.advance();
                    self.skip_line_comment();
                    continue;
                } else if self.peek() == Some('*') {
                    self.advance();
                    self.advance();
                    self.skip_block_comment()?;
                    continue;
                }
            }
            
            let span = Span { line: self.line, col: self.col };
            let token = self.next_token()?;
            tokens.push(token);
            spans.push(span);
        }
        
        let eof_span = Span { line: self.line, col: self.col };
        tokens.push(Token::Eof);
        spans.push(eof_span);
        Ok((tokens, spans))
    }
    
    fn next_token(&mut self) -> Result<Token, String> {
        let ch = self.current_char.unwrap();
        
        match ch {
            '+' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::PlusEq)
                } else {
                    Ok(Token::Plus)
                }
            }
            '-' => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    Ok(Token::Arrow)
                } else if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::MinusEq)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::StarEq)
                } else {
                    Ok(Token::Star)
                }
            }
            '/' => {
                self.advance();
                // Comments are handled in tokenize(), so this should only be division
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::SlashEq)
                } else {
                    Ok(Token::Slash)
                }
            }
            '%' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::PercentEq)
                } else {
                    Ok(Token::Percent)
                }
            }
            '=' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::EqEq)
                } else if self.current_char == Some('>') {
                    self.advance();
                    Ok(Token::FatArrow)
                } else {
                    Ok(Token::Eq)
                }
            }
            '!' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::Ne)
                } else {
                    Ok(Token::Exclamation)
                }
            }
            '<' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::Le)
                } else if self.current_char == Some('<') {
                    self.advance();
                    Ok(Token::Shl)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::Ge)
                } else if self.current_char == Some('>') {
                    self.advance();
                    Ok(Token::Shr)
                } else {
                    Ok(Token::Gt)
                }
            }
            '&' => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Ok(Token::Ampersand)
                }
            }
            '|' => {
                self.advance();
                if self.current_char == Some('|') {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Ok(Token::Pipe)  // Used for closure params: |x|
                }
            }
            '^' => {
                self.advance();
                Ok(Token::BitXor)
            }
            '~' => {
                self.advance();
                Ok(Token::Tilde)
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            }
            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            ':' => {
                self.advance();
                if self.current_char == Some(':') {
                    self.advance();
                    Ok(Token::ColonColon)
                } else {
                    Ok(Token::Colon)
                }
            }
            '.' => {
                self.advance();
                if self.current_char == Some('.') {
                    self.advance();
                    Ok(Token::DotDot)
                } else {
                    Ok(Token::Dot)
                }
            }
            '?' => {
                self.advance();
                Ok(Token::Question)
            }
            '#' => {
                self.advance();
                Ok(Token::Hash)
            }
            '"' => self.read_string(),
            '\'' => self.read_char(),
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(),
            _ => Err(format!("Unexpected character: {}", ch)),
        }
    }
    
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.current_char = self.input.next();
        self.position += 1;
    }
    
    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                self.advance(); // skip the newline too
                break;
            }
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self) -> Result<(), String> {
        // Called after the initial "/*" has already been consumed.
        let mut depth = 1usize;

        while let Some(ch) = self.current_char {
            if ch == '/' && self.peek() == Some('*') {
                self.advance(); // '/'
                self.advance(); // '*'
                depth += 1;
                continue;
            }

            if ch == '*' && self.peek() == Some('/') {
                self.advance(); // '*'
                self.advance(); // '/'
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
                continue;
            }

            self.advance();
        }

        Err("Unterminated block comment".to_string())
    }
    
    fn read_string(&mut self) -> Result<Token, String> {
        self.advance(); // skip opening quote
        let mut s = String::new();
        
        while let Some(ch) = self.current_char {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(Token::StringLiteral(s));
                }
                '\\' => {
                    self.advance();
                    let escaped = match self.current_char {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('\\') => '\\',
                        Some('"') => '"',
                        Some('0') => '\0',
                        Some(c) => return Err(format!("Invalid escape sequence: \\{}", c)),
                        None => return Err("Unexpected end of string".to_string()),
                    };
                    s.push(escaped);
                    self.advance();
                }
                _ => {
                    s.push(ch);
                    self.advance();
                }
            }
        }
        
        Err("Unterminated string literal".to_string())
    }
    
    fn read_char(&mut self) -> Result<Token, String> {
        self.advance(); // skip opening quote
        let ch = match self.current_char {
            Some('\\') => {
                self.advance();
                match self.current_char {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('0') => '\0',
                    Some(c) => return Err(format!("Invalid escape sequence: \\{}", c)),
                    None => return Err("Unexpected end of char literal".to_string()),
                }
            }
            Some(c) => c,
            None => return Err("Unexpected end of char literal".to_string()),
        };
        self.advance();
        
        if self.current_char != Some('\'') {
            return Err("Char literal must be single character".to_string());
        }
        self.advance();
        
        Ok(Token::CharLiteral(ch))
    }
    
    fn read_number(&mut self) -> Result<Token, String> {
        let mut num_str = String::new();
        let mut is_float = false;
        
        // Check for hex (0x) or binary (0b) prefix
        if self.current_char == Some('0') {
            num_str.push('0');
            self.advance();
            
            if self.current_char == Some('x') || self.current_char == Some('X') {
                // Hex literal
                self.advance(); // skip 'x'
                let mut hex_str = String::new();
                while let Some(ch) = self.current_char {
                    match ch {
                        '0'..='9' | 'a'..='f' | 'A'..='F' => {
                            hex_str.push(ch);
                            self.advance();
                        }
                        '_' => { self.advance(); }
                        _ => break,
                    }
                }
                if hex_str.is_empty() {
                    return Err("Expected hex digits after 0x".to_string());
                }
                return i64::from_str_radix(&hex_str, 16)
                    .map(Token::IntLiteral)
                    .map_err(|e| format!("Invalid hex literal: {}", e));
            } else if self.current_char == Some('b') || self.current_char == Some('B') {
                // Binary literal
                self.advance(); // skip 'b'
                let mut bin_str = String::new();
                while let Some(ch) = self.current_char {
                    match ch {
                        '0' | '1' => {
                            bin_str.push(ch);
                            self.advance();
                        }
                        '_' => { self.advance(); }
                        _ => break,
                    }
                }
                if bin_str.is_empty() {
                    return Err("Expected binary digits after 0b".to_string());
                }
                return i64::from_str_radix(&bin_str, 2)
                    .map(Token::IntLiteral)
                    .map_err(|e| format!("Invalid binary literal: {}", e));
            }
            // else: normal number starting with 0, fall through
        }
        
        while let Some(ch) = self.current_char {
            match ch {
                '0'..='9' => {
                    num_str.push(ch);
                    self.advance();
                }
                '.' => {
                    // Check if this is a Range operator (..) rather than a decimal point
                    // by peeking at the next character
                    if self.peek() == Some('.') {
                        // This is `..` (Range operator), don't consume the dot
                        break;
                    }
                    if is_float {
                        break;
                    }
                    is_float = true;
                    num_str.push(ch);
                    self.advance();
                }
                '_' => {
                    self.advance(); // skip underscores in numbers
                }
                _ => break,
            }
        }
        
        if is_float {
            num_str.parse::<f64>()
                .map(Token::FloatLiteral)
                .map_err(|e| format!("Invalid float literal: {}", e))
        } else {
            num_str.parse::<i64>()
                .map(Token::IntLiteral)
                .map_err(|e| format!("Invalid integer literal: {}", e))
        }
    }
    
    fn read_identifier_or_keyword(&mut self) -> Result<Token, String> {
        let mut ident = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Handle bool literals first (before keyword check)
        // This fixes the bug where true/false were parsed as keywords
        if ident == "true" {
            return Ok(Token::BoolLiteral(true));
        }
        if ident == "false" {
            return Ok(Token::BoolLiteral(false));
        }
        
        // Check if it's a keyword
        if let Some(keyword) = self.keyword_from_str(&ident) {
            Ok(Token::Keyword(keyword))
        } else {
            Ok(Token::Identifier(ident))
        }
    }
    
    fn keyword_from_str(&self, s: &str) -> Option<Keyword> {
        match s {
            "func" => Some(Keyword::Func),
            "let" => Some(Keyword::Let),
            "mut" => Some(Keyword::Mut),
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "for" => Some(Keyword::For),
            "loop" => Some(Keyword::Loop),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "return" => Some(Keyword::Return),
            "match" => Some(Keyword::Match),
            "struct" => Some(Keyword::Struct),
            "enum" => Some(Keyword::Enum),
            "impl" => Some(Keyword::Impl),
            "mod" => Some(Keyword::Mod),
            "import" => Some(Keyword::Import),
            "pub" => Some(Keyword::Pub),
            "const" => Some(Keyword::Const),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "as" => Some(Keyword::As),
            "unsafe" => Some(Keyword::Unsafe),
            "extern" => Some(Keyword::Extern),
            "use" => Some(Keyword::Use),
            "in" => Some(Keyword::In),
            "trait" => Some(Keyword::Trait),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("+ - * /");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
    }
    
    #[test]
    fn test_int_literal() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::IntLiteral(42));
    }
    
    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello\"");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::StringLiteral("hello".to_string()));
    }
    
    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("func let mut");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Keyword(Keyword::Func));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Let));
        assert_eq!(tokens[2], Token::Keyword(Keyword::Mut));
    }

    #[test]
    fn test_block_comment_between_tokens() {
        let mut lexer = Lexer::new("let /* comment */ x = 1;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Keyword(Keyword::Let));
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::Eq);
        assert_eq!(tokens[3], Token::IntLiteral(1));
        assert_eq!(tokens[4], Token::Semicolon);
        assert_eq!(tokens[5], Token::Eof);
    }

    #[test]
    fn test_nested_block_comments() {
        let mut lexer = Lexer::new("let /* outer /* inner */ still outer */ x = 1;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Keyword(Keyword::Let));
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::Eq);
        assert_eq!(tokens[3], Token::IntLiteral(1));
        assert_eq!(tokens[4], Token::Semicolon);
        assert_eq!(tokens[5], Token::Eof);
    }

    #[test]
    fn test_unterminated_block_comment_returns_error() {
        let mut lexer = Lexer::new("let /* never ends");
        let err = lexer.tokenize().unwrap_err();
        assert!(err.contains("Unterminated block comment"));
    }
}
