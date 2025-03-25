use pest::Parser;
use pest_derive::Parser;
use testangel_ipc::prelude::ParameterKind;

#[derive(Parser)]
#[grammar = "action_syntax/descriptor_grammar.pest"]
struct DescriptorParser;

#[derive(Debug)]
pub enum Descriptor {
    TypedDescriptor {
        descriptor_kind: TypedDescriptorKind,
        kind: ParameterKind,
        name: String,
    },
    KeyValueDescriptor {
        descriptor_kind: KeyValueDescriptorKind,
        value: String,
    },
    FlagDescriptor(FlagDescriptorKind),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypedDescriptorKind {
    Parameter,
    Return,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeyValueDescriptorKind {
    Name,
    Group,
    Creator,
    Description,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FlagDescriptorKind {
    HideInFlowEditor,
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
    ///
    /// ## Panics
    ///
    /// This cannot panic as long as this function remains updated alongside the pest grammar.
    pub fn parse_line<S: AsRef<str>>(line: S) -> Option<Descriptor> {
        let mut res = DescriptorParser::parse(Rule::Line, line.as_ref()).ok()?;
        let line_pair = res.next()?;
        assert_eq!(line_pair.as_rule(), Rule::Line);

        let descriptor_pair = line_pair.into_inner().next()?;
        assert_eq!(descriptor_pair.as_rule(), Rule::Descriptor);
        let descriptor_inner = descriptor_pair.into_inner().next()?;
        assert!([
            Rule::TypedDescriptor,
            Rule::KeyValueDescriptor,
            Rule::FlagDescriptor
        ]
        .contains(&descriptor_inner.as_rule()));

        Some(match descriptor_inner.as_rule() {
            Rule::TypedDescriptor => {
                let mut inner = descriptor_inner.into_inner();
                let kind = inner.next()?;
                assert_eq!(Rule::TypedDescriptorKind, kind.as_rule());
                let ty = inner.next()?;
                assert_eq!(Rule::Type, ty.as_rule());
                let value = inner.next()?;
                assert_eq!(Rule::Value, value.as_rule());

                Descriptor::TypedDescriptor {
                    descriptor_kind: match kind.as_str() {
                        "param" => TypedDescriptorKind::Parameter,
                        "return" => TypedDescriptorKind::Return,
                        _ => unreachable!(),
                    },
                    kind: match ty.as_str() {
                        "Boolean" => ParameterKind::Boolean,
                        "Decimal" => ParameterKind::Decimal,
                        "Integer" => ParameterKind::Integer,
                        "Text" => ParameterKind::String,
                        _ => unreachable!(),
                    },
                    name: value.as_str().to_owned(),
                }
            }
            Rule::KeyValueDescriptor => {
                let mut inner = descriptor_inner.into_inner();
                let kind = inner.next()?;
                assert_eq!(Rule::KeyValueDescriptorKind, kind.as_rule());
                let value = inner.next()?;
                assert_eq!(Rule::Value, value.as_rule());

                Descriptor::KeyValueDescriptor {
                    descriptor_kind: match kind.as_str() {
                        "name" => KeyValueDescriptorKind::Name,
                        "group" => KeyValueDescriptorKind::Group,
                        "creator" => KeyValueDescriptorKind::Creator,
                        "description" => KeyValueDescriptorKind::Description,
                        _ => unreachable!(),
                    },
                    value: value.as_str().to_owned(),
                }
            }
            Rule::FlagDescriptor => {
                let inner = descriptor_inner.into_inner().next()?;
                assert_eq!(Rule::FlagDescriptorKind, inner.as_rule());
                Descriptor::FlagDescriptor(match inner.as_str() {
                    "hide-in-flow-editor" => FlagDescriptorKind::HideInFlowEditor,
                    _ => unreachable!(),
                })
            }
            _ => unreachable!(),
        })
    }
}
