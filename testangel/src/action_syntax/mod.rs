use pest::Parser;
use pest_derive::Parser;
use testangel_ipc::prelude::ParameterKind;

#[derive(Parser)]
#[grammar = "action_syntax/descriptor_grammar.pest"]
struct DescriptorParser;

#[derive(Debug)]
pub struct Descriptor {
    pub descriptor_kind: DescriptorKind,
    pub kind: ParameterKind,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DescriptorKind {
    Parameter,
    Return,
}

impl Descriptor {
    /// Parse all the descriptors in a given file.
    pub fn parse_all<S: AsRef<str>>(file: S) -> Vec<Descriptor> {
        let mut descriptors = vec![];
        for line in file.as_ref().lines() {
            if let Some(descriptor) = Self::parse_line(line) {
                descriptors.push(descriptor);
            }
        }
        descriptors
    }

    /// Parse a line and return a descriptor, if one can be created from the provided line.
    pub fn parse_line<S: AsRef<str>>(line: S) -> Option<Descriptor> {
        let mut res = DescriptorParser::parse(Rule::Line, line.as_ref()).ok()?;
        let line_pair = res.next()?;
        assert_eq!(line_pair.as_rule(), Rule::Line);

        let descriptor_pair = line_pair.into_inner().next()?;
        assert_eq!(descriptor_pair.as_rule(), Rule::Descriptor);
        let mut descriptor = Descriptor {
            descriptor_kind: DescriptorKind::Parameter,
            kind: ParameterKind::Integer,
            name: String::new(),
        };
        for pair in descriptor_pair.into_inner() {
            match pair.as_rule() {
                Rule::DescriptorKind => {
                    descriptor.descriptor_kind = match pair.as_str() {
                        "param" => DescriptorKind::Parameter,
                        "return" => DescriptorKind::Return,
                        _ => unreachable!(),
                    }
                }
                Rule::Kind => {
                    descriptor.kind = match pair.as_str() {
                        "Boolean" => ParameterKind::Boolean,
                        "Decimal" => ParameterKind::Decimal,
                        "Integer" => ParameterKind::Integer,
                        "String" => ParameterKind::String,
                        _ => unreachable!(),
                    }
                }
                Rule::Name => descriptor.name = pair.as_str().to_string(),
                _ => unreachable!(),
            }
        }
        Some(descriptor)
    }
}
