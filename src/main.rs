extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "ppd.pest"]
pub struct PPDParser;

use std::error::Error;
use std::fs;

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
                let value = inner.next().unwrap().as_str();
                attributes.push(Attribute { key, value })
            }
            _ => {}
        }
    }

    Ok(attributes)
}

fn interpret(attributes: Vec<Attribute>) {
    let mut in_ui_for: Option<Key> = None;

    for attr in attributes {
        match attr.key.main {
            "CloseUI" => {
                in_ui_for = None;
            }
            "OpenUI" if attr.value == "PickOne" => {
                // Looks like:
                //   *OpenUI *ColorModel/Color Mode: PickOne
                // Or
                //   *OpenUI *CNIJGrayScale/Grayscale Printing: PickOne
                in_ui_for = Some(attr.key.clone());
            }
            _ => {}
        }

        if let Some(ref in_ui_for) = in_ui_for {
            // Remove * prefix from sub1, which isn't included in main.
            let (_, sub1) = in_ui_for.sub1.unwrap().split_at(1);

            if attr.key.main == sub1 {
                println!(
                    "found match {:?} {:?} {:?}",
                    sub1, attr.key.sub1, attr.key.sub2
                );
                println!("  {:?}", in_ui_for);
            }
        }
    }
}

fn main() {
    let unparsed_file = fs::read_to_string("test.ppd").expect("cannot read file");

    let attributes = parse(&unparsed_file).unwrap();

    interpret(attributes);
}
