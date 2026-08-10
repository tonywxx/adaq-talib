//! 集成测试共享工具：极简 JSON 解析器 + 黄金向量 fixture 加载。
//!
//! Shared integration-test utilities: a minimal JSON parser and golden-vector fixture loader.
//!
//! 受 No-Dependencies 约束（见 ADR 0003），不引入 `serde_json`，仅实现读取本仓库
//! fixture 所需的子集（对象 / 数组 / 数字 / null / 字符串）。
//!
//! Under the No-Dependencies constraint (ADR 0003) we do not pull in `serde_json`; this
//! implements only the subset needed to read our fixtures (object / array / number / null / string).
//!
//! 通用解析器，当前 fixture 仅用到对象/数组/数字/null；其余变体/方法为通用能力，允许 dead_code。
//! General-purpose parser; current fixtures only use object/array/number/null, so other
//! variants/methods are allowed to be dead_code.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(HashMap<String, Json>),
}

impl Json {
    pub fn as_object(&self) -> Option<&HashMap<String, Json>> {
        if let Json::Obj(m) = self { Some(m) } else { None }
    }
    pub fn as_array(&self) -> Option<&Vec<Json>> {
        if let Json::Arr(a) = self { Some(a) } else { None }
    }
    pub fn as_f64(&self) -> Option<f64> {
        if let Json::Num(n) = self { Some(*n) } else { None }
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser { b: s.as_bytes(), i: 0 }
    }

    fn skip_ws(&mut self) {
        while self.i < self.b.len() {
            match self.b[self.i] {
                b' ' | b'\t' | b'\n' | b'\r' => self.i += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => Ok(Json::Str(self.parse_string()?)),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(format!("unexpected char '{}' at {}", c as char, self.i)),
            None => Err("unexpected end of input".into()),
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.i += 1; // consume '{'
        let mut map = HashMap::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Obj(map));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err("expected object key string".into());
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err("expected ':'".into());
            }
            self.i += 1;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    break;
                }
                _ => return Err("expected ',' or '}'".into()),
            }
        }
        Ok(Json::Obj(map))
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.i += 1; // consume '['
        let mut arr = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Json::Arr(arr));
        }
        loop {
            let v = self.parse_value()?;
            arr.push(v);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    break;
                }
                _ => return Err("expected ',' or ']'".into()),
            }
        }
        Ok(Json::Arr(arr))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.i += 1; // consume opening quote
        let mut s = String::new();
        while self.i < self.b.len() {
            let c = self.b[self.i];
            self.i += 1;
            match c {
                b'"' => return Ok(s),
                b'\\' => {
                    if self.i >= self.b.len() {
                        return Err("bad escape".into());
                    }
                    let e = self.b[self.i];
                    self.i += 1;
                    match e {
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        b'n' => s.push('\n'),
                        b't' => s.push('\t'),
                        b'r' => s.push('\r'),
                        _ => return Err("unsupported escape".into()),
                    }
                }
                _ => s.push(c as char),
            }
        }
        Err("unterminated string".into())
    }

    fn parse_bool(&mut self) -> Result<Json, String> {
        if self.b[self.i..].starts_with(b"true") {
            self.i += 4;
            Ok(Json::Bool(true))
        } else if self.b[self.i..].starts_with(b"false") {
            self.i += 5;
            Ok(Json::Bool(false))
        } else {
            Err("invalid literal".into())
        }
    }

    fn parse_null(&mut self) -> Result<Json, String> {
        if self.b[self.i..].starts_with(b"null") {
            self.i += 4;
            Ok(Json::Null)
        } else {
            Err("invalid literal".into())
        }
    }

    fn parse_number(&mut self) -> Result<Json, String> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while self.i < self.b.len() {
            let c = self.b[self.i];
            if c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'+' || c == b'-' {
                self.i += 1;
            } else {
                break;
            }
        }
        let slice = &self.b[start..self.i];
        let s = std::str::from_utf8(slice).map_err(|e| e.to_string())?;
        s.parse::<f64>().map(Json::Num).map_err(|e| e.to_string())
    }
}

/// 解析 JSON 文本。/ Parse JSON text.
pub fn parse(s: &str) -> Result<Json, String> {
    let mut p = Parser::new(s);
    let v = p.parse_value()?;
    p.skip_ws();
    if p.i != p.b.len() {
        return Err("trailing characters after JSON value".into());
    }
    Ok(v)
}

/// 从 fixture 对象中取出名为 `key` 的数组，元素为数字或 `null`（`null` → `NaN`）。
/// Extract the array named `key` from a fixture object; `null` elements become `NaN`.
pub fn load_f64_array(json: &Json, key: &str) -> Result<Vec<f64>, String> {
    let obj = json.as_object().ok_or("fixture root is not an object")?;
    let arr = obj
        .get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("missing array field '{key}'"))?;
    arr.iter()
        .map(|v| match v {
            Json::Num(n) => Ok(*n),
            Json::Null => Ok(f64::NAN),
            _ => Err(format!("field '{key}' contains non-number element")),
        })
        .collect()
}

/// 加载 `tests/fixtures/<name>` 黄金向量，返回 (输入, 期望输出)。
/// Load the golden vector `tests/fixtures/<name>`, returning (input, expected).
pub fn load_fixture(name: &str) -> Result<(Vec<f64>, Vec<f64>), String> {
    let json = load_json(name)?;
    let input = load_f64_array(&json, "input")?;
    let expected = load_f64_array(&json, "expected")?;
    Ok((input, expected))
}

/// 加载 `tests/fixtures/<name>` 的原始 JSON 对象，便于读取多字段 fixture
/// （如 `midprice` 的 `high`/`low`/`expected`）。
/// Load the raw JSON object of `tests/fixtures/<name>`, for multi-field fixtures
/// (e.g. `midprice`'s `high`/`low`/`expected`).
pub fn load_json(name: &str) -> Result<Json, String> {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    parse(&text)
}
