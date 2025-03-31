use parse_display::Display;
use regex::{Regex, escape};
use std::fmt::Write;
use std::ops::Range;
use std::str::{self, CharIndices};
use std::sync::LazyLock;
use std::{borrow::Cow, fmt};

mod vars;

mod tests_readme;

pub use vars::Vars;

/// RFC6570 Level 2
#[derive(Clone)]
pub struct UriTemplate {
    source: String,
    segments: Vec<Segment>,
    exprs: Vec<Expr>,
    regex: Regex,
}
impl std::fmt::Debug for UriTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.source)
    }
}
impl fmt::Display for UriTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Debug, Clone)]
enum Segment {
    Literals { len: usize },
    LiteralsNeedEncode { len: usize },
    Expr,
}
impl Segment {
    fn expand(
        &self,
        source: &str,
        source_index: &mut usize,
        exprs: &[Expr],
        expr_index: &mut usize,
        vars: &mut impl Vars,
        out: &mut String,
    ) {
        match self {
            Segment::Literals { len } => {
                out.push_str(&source[*source_index..*source_index + len]);
                *source_index += len;
            }
            Segment::LiteralsNeedEncode { len } => {
                for c in source[*source_index..*source_index + len].chars() {
                    encode_char(c, out);
                }
                *source_index += len;
            }
            Segment::Expr => {
                let expr = &exprs[*expr_index];
                expr.expand(source, *expr_index, vars, out);
                *source_index += expr.len();
                *expr_index += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Expr {
    op: Option<Operator>,
    var_name_range: Range<usize>,
}
impl Expr {
    fn len(&self) -> usize {
        self.var_name_range.len() + 2 + if self.op.is_some() { 1 } else { 0 }
    }
    fn to_regex(&self) -> String {
        match self.op {
            Some(op) => {
                let prefix = escape(op.to_prefix());
                format!("(?:{prefix}([{RE_UNRESERVED}{RE_RESERVED}%]*))?",)
            }
            None => format!("([{RE_UNRESERVED}%]*)",),
        }
    }
    fn expand(&self, source: &str, expr_index: usize, vars: &mut impl Vars, out: &mut String) {
        let var_name = &source[self.var_name_range.clone()];
        let var = vars.var(expr_index, var_name);
        let Some(var) = var else {
            return;
        };
        match self.op {
            Some(op) => {
                out.push_str(op.to_prefix());
                encode_str_url(&var, out);
            }
            None => {
                encode_str_unresreved(&var, out);
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Operator {
    /// `+`
    Reserved,
    /// `#`
    Fragment,
}
impl Operator {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Reserved),
            '#' => Some(Self::Fragment),
            _ => None,
        }
    }
    fn to_prefix(self) -> &'static str {
        match self {
            Self::Reserved => "",
            Self::Fragment => "#",
        }
    }
}

impl UriTemplate {
    pub fn new(s: &str) -> Result<Self> {
        let mut segments = Vec::new();
        let mut exprs = Vec::new();
        let mut iter = DecodedIter::new(s);
        let mut current = iter.next();
        let mut re = String::from("^");

        'root: while let Some(d) = current {
            match d {
                Decoded::Char { index, ch: '{' } => {
                    let var_start = index;
                    let mut op = None;
                    current = iter.next();
                    if let Some(d) = current {
                        if let Some(ch) = d.ch() {
                            op = Operator::from_char(ch);
                        }
                        if op.is_some() {
                            current = iter.next();
                        }
                    }
                    if let Some(d) = current {
                        let var_name_start = d.index();
                        while let Some(d) = current {
                            if d.ch() == Some('}') {
                                let expr = Expr {
                                    op,
                                    var_name_range: var_name_start..d.index(),
                                };
                                re.push_str(&expr.to_regex());
                                exprs.push(expr);
                                segments.push(Segment::Expr);
                                current = iter.next();
                                continue 'root;
                            }
                            current = iter.next();
                        }
                        return Err(Error {
                            source: s.to_string(),
                            kind: ErrorKind::InvalidExpression,
                            source_index: var_start,
                        });
                    }
                }
                Decoded::Char { ch, .. } => {
                    let len = ch.len_utf8();
                    if is_reserved(ch) || is_unreserved(ch) {
                        segments.push(Segment::Literals { len: 1 });
                        re.push_str(&escape(&ch.to_string()));
                    } else {
                        segments.push(Segment::LiteralsNeedEncode { len });
                        let mut s0 = String::new();
                        encode_char(ch, &mut s0);
                        re.push_str(&escape(&s0));
                    }
                }
                Decoded::Byte { s, .. } => {
                    segments.push(Segment::Literals { len: s.len() });
                    re.push_str(&escape(s));
                }
            }
            current = iter.next();
        }
        re.push('$');
        Ok(Self {
            source: s.to_string(),
            segments,
            exprs,
            regex: Regex::new(&re).unwrap(),
        })
    }

    pub fn expand(&self, mut vars: impl Vars) -> String {
        let mut out = String::new();
        let mut expr_index = 0;
        let mut source_index = 0;
        for segment in &self.segments {
            segment.expand(
                &self.source,
                &mut source_index,
                &self.exprs,
                &mut expr_index,
                &mut vars,
                &mut out,
            );
        }
        out
    }
    pub fn captures<'a>(&'a self, input: &'a str) -> Option<Captures<'a>> {
        let captures = self.regex.captures(input)?;
        let mut ms = Vec::with_capacity(self.exprs.len());
        for (expr_index, expr) in self.exprs.iter().enumerate() {
            if let Some(m) = captures.get(expr_index + 1) {
                ms.push(Some(Match::new(m, self.var_name(expr_index), expr.op)));
            } else {
                ms.push(None);
            }
        }
        Some(Captures { template: self, ms })
    }
    fn var_name(&self, index: usize) -> &str {
        &self.source[self.exprs[index].var_name_range.clone()]
    }

    pub fn var_names(&self) -> impl Iterator<Item = &str> {
        (0..self.exprs.len()).map(|i| self.var_name(i))
    }
    pub fn find_var_name(&self, name: &str) -> Option<usize> {
        (0..self.exprs.len()).find(|&i| self.var_name(i) == name)
    }
}

fn is_unreserved(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~')
}
const RE_UNRESERVED: &str = r"A-Za-z0-9\-._~";

fn is_reserved(c: char) -> bool {
    matches!(
        c,
        ':' | '/'
            | '?'
            | '#'
            | '['
            | ']'
            | '@'
            | '!'
            | '$'
            | '&'
            | '\'' // https://www.rfc-editor.org/errata/eid6937
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | ';'
            | '='
    )
}
const RE_RESERVED: &str = r":/?#\[\]@!$&'()*+,;=";

fn encode_char(ch: char, out: &mut String) {
    for b in ch.encode_utf8(&mut [0; 4]).as_bytes() {
        write!(out, "%{b:02X}").unwrap();
    }
}
fn encode_str_unresreved(s: &str, out: &mut String) {
    for ch in s.chars() {
        if is_unreserved(ch) {
            out.push(ch);
        } else {
            encode_char(ch, out);
        }
    }
}
fn encode_str_url(s: &str, out: &mut String) {
    let iter = DecodedIter::new(s);
    for d in iter {
        match d {
            Decoded::Char { ch, .. } => {
                if is_unreserved(ch) || is_reserved(ch) {
                    out.push(ch);
                } else {
                    encode_char(ch, out);
                }
            }
            Decoded::Byte { s, .. } => {
                out.push_str(s);
            }
        }
    }
}

struct Decoder<'a> {
    source: &'a str,
    source_index: usize,
    out: String,
    bytes: Vec<u8>,
}
impl<'a> Decoder<'a> {
    fn new(source: &'a str, source_index: usize) -> Self {
        Self {
            source,
            source_index,
            out: String::new(),
            bytes: Vec::new(),
        }
    }
    fn push_char(&mut self, ch: char) -> Result<()> {
        self.commit_bytes()?;
        self.source_index += ch.len_utf8();
        self.out.push(ch);
        Ok(())
    }
    fn push_byte(&mut self, b: u8) {
        self.bytes.push(b);
    }
    fn commit_bytes(&mut self) -> Result<()> {
        for check in self.bytes.utf8_chunks() {
            let valid = check.valid();
            self.source_index += valid.len() * 3;
            self.out.push_str(valid);
            if !check.invalid().is_empty() {
                return Err(Error::new(
                    self.source,
                    self.source_index,
                    ErrorKind::InvalidUtf8,
                ));
            }
        }
        self.bytes.clear();
        Ok(())
    }
    fn build(mut self) -> Result<String> {
        self.commit_bytes()?;
        Ok(self.out)
    }
}

fn decode_str(s: &str, source_index: usize) -> Result<String> {
    let mut out = Decoder::new(s, source_index);
    for d in DecodedIter::new(s) {
        match d {
            Decoded::Char { ch, .. } => {
                out.push_char(ch)?;
            }
            Decoded::Byte { b, .. } => {
                out.push_byte(b);
            }
        }
    }
    out.build()
}

fn to_u8(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum Decoded<'a> {
    Char { index: usize, ch: char },
    Byte { index: usize, b: u8, s: &'a str },
}
impl Decoded<'_> {
    fn ch(&self) -> Option<char> {
        match self {
            Self::Char { ch, .. } => Some(*ch),
            Self::Byte { .. } => None,
        }
    }
    fn index(&self) -> usize {
        match self {
            Self::Char { index, .. } => *index,
            Self::Byte { index, .. } => *index,
        }
    }
}
#[derive(Clone)]
struct DecodedIter<'a> {
    source: &'a str,
    chars_indices: CharIndices<'a>,
}
impl<'a> DecodedIter<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            chars_indices: source.char_indices(),
        }
    }
}
impl<'a> Iterator for DecodedIter<'a> {
    type Item = Decoded<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, ch) = self.chars_indices.next()?;
        if ch == '%' {
            let this = self.clone();
            if let Some(b) = next_decoded_u8(&mut self.chars_indices) {
                Some(Decoded::Byte {
                    index,
                    b,
                    s: &self.source[index..][..3],
                })
            } else {
                *self = this;
                Some(Decoded::Char { index, ch: '%' })
            }
        } else {
            Some(Decoded::Char { index, ch })
        }
    }
}

fn next_decoded_u8(chars_indices: &mut CharIndices) -> Option<u8> {
    let c0 = next_hex(chars_indices)?;
    let c1 = next_hex(chars_indices)?;
    Some(c0 * 16 + c1)
}
fn next_hex(chars_indices: &mut CharIndices) -> Option<u8> {
    let (_, c) = chars_indices.next()?;
    to_u8(c)
}

#[derive(Debug)]
pub struct Captures<'a> {
    template: &'a UriTemplate,
    ms: Vec<Option<Match<'a>>>,
}

impl Captures<'_> {
    pub fn empty() -> Self {
        static DUMMY_TEMPLATE: LazyLock<UriTemplate> =
            LazyLock::new(|| UriTemplate::new("").unwrap());
        Self {
            template: &DUMMY_TEMPLATE,
            ms: Vec::new(),
        }
    }

    pub fn name(&self, name: &str) -> Option<&Match> {
        for (expr, m) in self.template.exprs.iter().zip(&self.ms) {
            if &self.template.source[expr.var_name_range.clone()] == name {
                return m.as_ref();
            }
        }
        None
    }
    pub fn get(&self, i: usize) -> Option<&Match> {
        self.ms.get(i)?.as_ref()
    }
    pub fn len(&self) -> usize {
        self.ms.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ms.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, Option<&Match>)> {
        (0..self.ms.len()).map(|i| (self.template.var_name(i), self.get(i)))
    }
}

#[derive(Debug)]
pub struct Match<'a> {
    m: regex::Match<'a>,
    name: &'a str,
    op: Option<Operator>,
}
impl<'a> Match<'a> {
    fn new(m: regex::Match<'a>, name: &'a str, op: Option<Operator>) -> Self {
        Self { m, name, op }
    }
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn value(&self) -> Result<Cow<str>> {
        match self.op {
            None => Ok(Cow::Owned(decode_str(self.m.as_str(), 0)?)),
            Some(Operator::Reserved | Operator::Fragment) => Ok(Cow::Borrowed(self.source())),
        }
    }
    pub fn source(&self) -> &str {
        self.m.as_str()
    }
    pub fn start(&self) -> usize {
        self.m.start()
    }
    pub fn end(&self) -> usize {
        self.m.end()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Display)]
enum ErrorKind {
    InvalidExpression,
    InvalidUtf8,
}

#[derive(Clone, Debug)]
pub struct Error {
    source: String,
    source_index: usize,
    kind: ErrorKind,
}

impl Error {
    fn new(source: &str, source_index: usize, kind: ErrorKind) -> Self {
        Self {
            source: source.to_string(),
            source_index,
            kind,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{} (\"{} >>>> {}\")",
            self.kind,
            &self.source[..self.source_index],
            &self.source[self.source_index..],
        )
    }
}

impl std::error::Error for Error {}
