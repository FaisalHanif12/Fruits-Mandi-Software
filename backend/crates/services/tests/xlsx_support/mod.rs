//! A tiny reader for the sheets the export writes, for tests only. It opens the `.xlsx` zip and
//! reads the first sheet's cells as the text Excel stores: the `<v>` of a number, the shared
//! string of a text. Reading the stored text (not a parsed float) lets a test see exactly what
//! the library wrote. Nothing like it is part of the app (money agent: never read a sheet back).
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Text(String),
    /// The stored number, as the text in the file (`2880`, `-10160`, `46305`).
    Num(String),
    Blank,
}

#[derive(Debug)]
pub struct Sheet {
    pub tab: String,
    pub right_to_left: bool,
    pub frozen: bool,
    pub styles: String,
    pub formulas: usize,
    cells: BTreeMap<(u32, u32), Value>,
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn zip_text(path: &Path, name: &str) -> Option<String> {
    let file = std::fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut entry = archive.by_name(name).ok()?;
    let mut text = String::new();
    entry.read_to_string(&mut text).unwrap();
    Some(text)
}

fn shared_strings(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    for item in xml.split("<si>").skip(1) {
        let item = item.split("</si>").next().unwrap_or("");
        let mut text = String::new();
        for part in item.split("<t").skip(1) {
            if let Some(body) = part.split_once('>').map(|(_, b)| b) {
                text.push_str(&unescape(body.split("</t>").next().unwrap_or("")));
            }
        }
        out.push(text);
    }
    out
}

/// `A1` to (row 0, column 0).
fn position(reference: &str) -> (u32, u32) {
    let letters: String = reference
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect();
    let digits: String = reference.chars().skip(letters.len()).collect();
    let column = letters.bytes().fold(0_u32, |acc, b| {
        acc * 26 + u32::from(b.to_ascii_uppercase() - b'A') + 1
    }) - 1;
    (digits.parse::<u32>().unwrap() - 1, column)
}

pub fn read(path: &Path) -> Sheet {
    let strings = shared_strings(&zip_text(path, "xl/sharedStrings.xml").unwrap_or_default());
    let workbook = zip_text(path, "xl/workbook.xml").unwrap();
    let tab = workbook
        .split("<sheet ")
        .nth(1)
        .and_then(|tag| attr(&format!(" {tag}"), "name").map(unescape))
        .unwrap();
    let xml = zip_text(path, "xl/worksheets/sheet1.xml").unwrap();
    let mut cells = BTreeMap::new();
    for chunk in xml.split("<c ").skip(1) {
        let (tag, rest) = chunk.split_once('>').unwrap();
        let spaced = format!(" {tag}");
        let reference = attr(&spaced, "r").unwrap();
        let kind = attr(&spaced, "t");
        let value = if tag.ends_with('/') {
            Value::Blank
        } else {
            let body = rest.split("</c>").next().unwrap_or("");
            match body
                .split("<v>")
                .nth(1)
                .and_then(|v| v.split("</v>").next())
            {
                None => Value::Blank,
                Some(v) if kind == Some("s") => {
                    Value::Text(strings[v.parse::<usize>().unwrap()].clone())
                }
                Some(v) => Value::Num(v.to_owned()),
            }
        };
        cells.insert(position(reference), value);
    }
    Sheet {
        tab,
        right_to_left: xml.contains("rightToLeft=\"1\""),
        frozen: xml.contains("<pane "),
        styles: zip_text(path, "xl/styles.xml").unwrap_or_default(),
        formulas: xml.matches("<f>").count() + xml.matches("<f ").count(),
        cells,
    }
}

impl Sheet {
    pub fn get(&self, row: u32, col: u32) -> &Value {
        self.cells.get(&(row, col)).unwrap_or(&Value::Blank)
    }

    pub fn last_row(&self) -> u32 {
        self.cells.keys().map(|(r, _)| *r).max().unwrap_or(0)
    }

    pub fn row_texts(&self, row: u32) -> Vec<String> {
        self.cells
            .range((row, 0)..(row + 1, 0))
            .filter_map(|(_, v)| match v {
                Value::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    /// The first row where column `col` holds exactly `text`.
    pub fn find_text(&self, col: u32, text: &str) -> Option<u32> {
        self.cells
            .iter()
            .find(|((_, c), v)| *c == col && **v == Value::Text(text.to_owned()))
            .map(|((r, _), _)| *r)
    }

    /// Every text of the sheet, in reading order.
    pub fn all_texts(&self) -> Vec<String> {
        self.cells
            .values()
            .filter_map(|v| match v {
                Value::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn num(&self, row: u32, col: u32) -> String {
        match self.get(row, col) {
            Value::Num(n) => n.clone(),
            other => panic!("row {row} column {col} is not a number: {other:?}"),
        }
    }

    pub fn text(&self, row: u32, col: u32) -> String {
        match self.get(row, col) {
            Value::Text(t) => t.clone(),
            other => panic!("row {row} column {col} is not text: {other:?}"),
        }
    }
}
