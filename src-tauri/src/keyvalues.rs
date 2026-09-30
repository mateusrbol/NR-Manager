use std::collections::BTreeMap;

use crate::error::{AppError, Result};

/// Arvore KeyValues da Valve (usada em .vdf, .acf e GameInfo).
#[derive(Debug, Clone)]
pub enum Kv {
    Str(String),
    Obj(BTreeMap<String, Kv>),
}

impl Kv {
    pub fn get(&self, key: &str) -> Option<&Kv> {
        match self {
            Kv::Obj(map) => map.get(key).or_else(|| {
                // Busca case-insensitive como fallback.
                map.iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(key))
                    .map(|(_, v)| v)
            }),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Kv::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&String, &Kv)> + '_> {
        match self {
            Kv::Obj(map) => Box::new(map.iter()),
            _ => Box::new(std::iter::empty()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Open,
    Close,
    Value(String),
}

fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let bytes: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            '{' => {
                tokens.push(Token::Open);
                i += 1;
            }
            '}' => {
                tokens.push(Token::Close);
                i += 1;
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                while i < bytes.len() && bytes[i] != '"' {
                    if bytes[i] == '\\' && i + 1 < bytes.len() {
                        i += 1;
                        s.push(bytes[i]);
                    } else {
                        s.push(bytes[i]);
                    }
                    i += 1;
                }
                i += 1; // fecha aspas
                tokens.push(Token::Value(s));
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] == '/' => {
                while i < bytes.len() && bytes[i] != '\n' {
                    i += 1;
                }
            }
            c if c.is_whitespace() => i += 1,
            _ => {
                let mut s = String::new();
                while i < bytes.len()
                    && !bytes[i].is_whitespace()
                    && bytes[i] != '{'
                    && bytes[i] != '}'
                {
                    s.push(bytes[i]);
                    i += 1;
                }
                if !s.is_empty() {
                    tokens.push(Token::Value(s));
                }
            }
        }
    }
    Ok(tokens)
}

/// Faz o parse de um texto KeyValues.
pub fn parse(input: &str) -> Result<Kv> {
    let input = input.trim_start_matches('\u{feff}');
    let tokens = tokenize(input)?;
    let mut pos = 0usize;

    fn parse_obj(tokens: &[Token], pos: &mut usize) -> Result<Kv> {
        let mut map = BTreeMap::new();
        while *pos < tokens.len() {
            match &tokens[*pos] {
                Token::Close => {
                    *pos += 1;
                    break;
                }
                Token::Value(key) => {
                    let key = key.clone();
                    *pos += 1;
                    if *pos >= tokens.len() {
                        map.insert(key, Kv::Str(String::new()));
                        break;
                    }
                    match &tokens[*pos] {
                        Token::Open => {
                            *pos += 1;
                            let child = parse_obj(tokens, pos)?;
                            map.insert(key, child);
                        }
                        Token::Value(v) => {
                            let v = v.clone();
                            *pos += 1;
                            map.insert(key, Kv::Str(v));
                        }
                        Token::Close => {
                            map.insert(key, Kv::Str(String::new()));
                        }
                    }
                }
                Token::Open => {
                    // objeto sem chave: ignora
                    *pos += 1;
                    let _ = parse_obj(tokens, pos)?;
                }
            }
        }
        Ok(Kv::Obj(map))
    }

    // Ignora uma eventual chave raiz (ex.: "libraryfolders").
    if let Some(Token::Value(root)) = tokens.first() {
        if tokens.get(1) == Some(&Token::Open) {
            pos = 2;
            let obj = parse_obj(&tokens, &mut pos)?;
            let mut map = BTreeMap::new();
            map.insert(root.clone(), obj);
            return Ok(Kv::Obj(map));
        }
    }

    loop {
        match tokens.get(pos) {
            Some(Token::Open) => {
                pos += 1;
                return parse_obj(&tokens, &mut pos);
            }
            Some(Token::Value(_)) => {
                // Formato sem chave raiz: reconstroi um objeto.
                pos = 0;
                return parse_obj(&tokens, &mut pos);
            }
            None => return Ok(Kv::Obj(BTreeMap::new())),
            _ => pos += 1,
        }
    }
}

/// Le e interpreta um arquivo KeyValues.
pub fn parse_file(path: &std::path::Path) -> Result<Kv> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| AppError::msg(format!("Falha ao ler {}: {e}", path.display())))?;
    parse(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let input = r#"
"libraryfolders"
{
    "0"
    {
        "path" "C:\\Games\\Steam"
        "label" ""
    }
    "1"
    {
        "path" "D:\\SteamLibrary"
    }
}
"#;
        let kv = parse(input).unwrap();
        let libs = kv.get("libraryfolders").unwrap();
        assert_eq!(libs.get("0").unwrap().get_str("path").unwrap(), "C:\\Games\\Steam");
        assert_eq!(libs.get("1").unwrap().get_str("path").unwrap(), "D:\\SteamLibrary");
    }
}
