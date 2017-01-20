#![allow(dead_code)]

use std::io;
use std::io::BufRead;
use std::fs;
use std::path;
use std::error;
use std::ascii::AsciiExt;



struct Relation {
    pub filename: String,
    pub name: String,
    data: Vec<Instance>,
    schema: Vec<AttributeFormat>,
}

struct AttributeFormat {
    pub name: String,
    pub attr_type: AttributeType,
}

enum AttributeType {
    Numeric,
    Nominal(Vec<String>),
}

struct Instance {
    values: Vec<Value>,
}

enum Value {
    Numeric(f64),
    Nominal(usize),
    Missing,
}

fn next_quoted(iter: &mut Iterator<Item=&str>, split: char) -> Option<String> {
    while let Some(token) = iter.next() {
        if token.is_empty() { continue; }
        if token.starts_with("'") {
            let mut result = token[1..].to_string();
            while !result.ends_with("'") {
                let token = iter.next();
                match token {
                    Some(token) => {
                        result.push(split);
                        result.push_str(token);
                    },
                    None => return None,
                }
            }
            let len = result.len();
            assert!(len > 0);
            result.truncate(len - 1);
            return Some(result)
        } else {
            return Some(token.to_string())
        }
    }
    None
}

impl AttributeType {
    fn parse(type_str: &str) -> Result<AttributeType, String> {
        if ["real", "continuous", "integer"].iter().any(|x| type_str.eq_ignore_ascii_case(x)) {
            return Ok(AttributeType::Numeric);
        }
        if !(type_str.starts_with("{") && type_str.ends_with("}")) {
            return Err(format!("Invalid attribute type: {}", type_str));
        }
        let type_str = &type_str[1..type_str.len() - 1];
        let mut entries = type_str.split(',');
        let mut values = Vec::new();
        while let Some(value) = next_quoted(&mut entries, ',') {
            values.push(value);
        }
        if values.len() == 0 {
            Err("Incomplete nominal attribute type".to_string())
        } else {
            Ok(AttributeType::Nominal(values))
        }
    }
}

impl Relation {
    fn load_header_line(&mut self, line: &str) -> Result<bool, String> {
        let mut tokens = line.split(' ');
        let token = tokens.next();
        if token.is_none() {
            return Ok(false);
        }
        let token = token.unwrap();
//        .map(|x| x.to_ascii_lowercase());
        match token.as_ref() {
            "@relation" => {
                self.name = next_quoted(&mut tokens, ' ')
                    .ok_or("No relation name given")?.to_string();
                Ok(false)
            } ,
            "@attribute" => {
                let name = next_quoted(&mut tokens, ' ').ok_or("No attribute name given")?;
                let entry = AttributeFormat {
                    name: name,
                    attr_type: AttributeType::parse(
                        tokens.collect::<Vec<_>>().join(" ").trim()
                    )?
                };
                self.schema.push(entry);
                Ok(false)
            },
            "@data" => Ok(true),
            "" => Ok(false),
            token @ _ => Err(format!("Unrecognized token {} in header", token)),
        }
    }

    fn load_data_line(&mut self, line: &str) -> Result<(), String> {
        let values = line.split();
        unimplemented!();
    }

    pub fn load_arff(filename: &path::Path) -> Result<Relation, Box<error::Error>> {
        let file = fs::File::open(filename)?;
        let mut result = Relation {
            filename: match filename.to_str() {
                Some(v) => v.to_string(),
                None => "".to_string(),
            },
            name: String::new(),
            schema: Vec::new(),
            data: Vec::new()
        };

        let reader = io::BufReader::new(file);
        let mut in_header = false;

        for line in reader.lines().filter_map(|x| x.ok()).filter(|x| !x.starts_with("%")) {
            if in_header {
                if result.load_header_line(&line)? { in_header = false; }
            } else {
                result.load_data_line(&line)?;
            }
        }

        Ok(result)
    }
}
