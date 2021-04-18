extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::Parser;

use serde::{Deserialize, Serialize};

use std::env;
use std::error::Error;
use std::fs;

#[derive(Parser)]
#[grammar = "ppd.pest"]
pub struct PPDParser;

#[derive(Clone, Debug)]
struct Key<'a> {
    main: &'a str,
    sub1: Option<&'a str>,
    sub2: Option<&'a str>,
}

#[derive(Debug)]
struct Attribute<'a> {
    key: Key<'a>,
    value: &'a str,
}

fn parse_key(key: Pair<Rule>) -> Key {
    let mut inner = key.into_inner();
    Key {
        main: inner.next().unwrap().as_str(),
        sub1: inner.next().map(|pair| pair.as_str()),
        sub2: inner.next().map(|pair| pair.as_str()),
    }
}

fn parse(input: &str) -> Result<Vec<Attribute>, Box<dyn Error>> {
    let file = PPDParser::parse(Rule::file, input)?.next().unwrap();

    let mut attributes = Vec::new();
    for line in file.into_inner() {
        match line.as_rule() {
            Rule::data => {
                let mut inner = line.into_inner();
                let key = parse_key(inner.next().unwrap());
                let value = inner.next().map(|value| value.as_str()).unwrap_or("");
                attributes.push(Attribute { key, value })
            }
            _ => {}
        }
    }

    Ok(attributes)
}

#[derive(Debug, Serialize, Deserialize)]
struct Setting {
    main: String,
    description: String,
    options: Vec<(String, String)>,
}

fn interpret(attributes: Vec<Attribute>) -> Vec<Setting> {
    let mut settings = Vec::new();

    let mut in_ui_for: Option<Key> = None;
    let mut options = Vec::new();

    for attr in attributes {
        match attr.key.main {
            "CloseUI" => {
                if let Some(in_ui_for) = in_ui_for {
                    settings.push(Setting {
                        main: in_ui_for.sub1.unwrap().split_at(1).1.to_owned(),
                        description: in_ui_for.sub2.unwrap_or(&"").to_owned(),
                        options,
                    });
                    options = Vec::new();
                }

                in_ui_for = None;
            }
            "OpenUI" => {
                // Looks like:
                //   *OpenUI *ColorModel/Color Mode: PickOne
                // Or
                //   *OpenUI *CNIJGrayScale/Grayscale Printing: PickOne
                in_ui_for = Some(attr.key.clone());
                options = Vec::new()
            }
            _ => {}
        }

        if let Some(ref in_ui_for) = in_ui_for {
            // Remove * prefix from sub1, which isn't included in main.
            let (_, sub1) = in_ui_for.sub1.unwrap().split_at(1);

            if attr.key.main == sub1 {
                options.push((
                    attr.key.sub1.unwrap_or("").to_owned(),
                    attr.key.sub2.unwrap_or("").to_owned(),
                ));
            }
        }
    }

    return settings;
}

fn main() {
    let path = env::args()
        .nth(1)
        .expect("missing <file> command lind argument");
    let unparsed_file = fs::read_to_string(path).expect("cannot read file");

    let attributes = parse(&unparsed_file).unwrap();
    let settings = interpret(attributes);

    let json_string = serde_json::to_string_pretty(&settings).unwrap();
    println!("{}", json_string);
}
