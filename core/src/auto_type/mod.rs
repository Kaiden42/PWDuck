//! TODO

use std::ops::Deref;

use pest::{iterators::Pairs, Parser};

use crate::{error::SequenceParseError, EntryBody, EntryHead, PWDuckCoreError, SecString};

/// TODO
#[derive(Debug, Parser)]
#[grammar = "auto_type/auto_type.pest"]
pub struct AutoTypeSequenceParser;

impl AutoTypeSequenceParser {
    /// This checks if the sequence can be parsed. It will currently not check whether the fields or keys are valid.
    /// TODO
    #[must_use]
    pub fn validate_sequence(sequence: &str) -> bool {
        Self::parse(Rule::sequence, sequence).is_ok()
    }

    /// Parse the sequence and return the gathered information for the autotyper.
    /// TODO
    pub fn parse_sequence<'a>(
        sequence: &'a str,
        entry_head: &'a EntryHead,
        entry_body: &'a EntryBody,
    ) -> Result<Sequence, PWDuckCoreError> {
        let sequence = Self::parse(Rule::sequence, sequence)
            //.map_err(|err| PWDuckCoreError::Error(format!("{}", err)))?;
            .map_err(SequenceParseError::from)?;

        let mut parts = Vec::new();

        Self::parse_inner(sequence, entry_head, entry_body, &mut parts)?;

        Ok(Sequence::with(parts))
    }

    /// Parse a pair of rules and return the gathered information for the autotyper.
    /// TODO
    fn parse_inner<'a>(
        sequence: Pairs<Rule>,
        entry_head: &'a EntryHead,
        entry_body: &'a EntryBody,
        //key_sequence: &mut Vec<SecString>,
        parts: &mut Vec<Part>,
    ) -> Result<(), PWDuckCoreError> {
        for pair in sequence {
            match pair.as_rule() {
                Rule::literal => parts.push(Part::Literal(pair.as_str().into())),
                Rule::field => match pair.as_str() {
                    "[title]" => parts.push(Part::Field(entry_head.title().clone().into())),
                    "[username]" => parts.push(Part::Field(entry_body.username().clone().into())),
                    "[password]" => parts.push(Part::Field(entry_body.password().clone().into())),
                    "[email]" => parts.push(Part::Field(entry_body.email().clone().into())),
                    //_ => return Err(PWDuckCoreError::Error("No valid field".into())),
                    _ => return Err(SequenceParseError::InvalidField(pair.as_str().into()).into()),
                },
                Rule::key => match pair.as_str() {
                    "<enter>" => parts.push(Part::Key(Key::Return)),
                    "<tab>" => parts.push(Part::Key(Key::Tab)),
                    //_ => return Err(PWDuckCoreError::Error("No valid key".into())),
                    _ => return Err(SequenceParseError::InvalidKey(pair.as_str().into()).into()),
                },
                Rule::sequence => {
                    Self::parse_inner(pair.into_inner(), entry_head, entry_body, parts)?
                }
                Rule::char => return Err(PWDuckCoreError::Error("Parse error".into())),
            }
        }

        Ok(())
    }
}

/// The parsed autotype sequence.
#[derive(Debug, PartialEq)]
pub struct Sequence(Vec<Part>);

impl Sequence {
    /// Create a new sequence containing the given parts.
    #[must_use]
    pub fn with(parts: Vec<Part>) -> Self {
        Self(parts)
    }
}

impl Deref for Sequence {
    type Target = Vec<Part>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// One part of the [`Sequence`](Sequence).
#[derive(Debug, PartialEq)]
pub enum Part {
    /// A literal containing characters.
    Literal(SecString),
    /// The value of an entry field.
    Field(SecString),
    /// The key to press.
    Key(Key),
}

/// A key of the autotype [`Sequence`](Sequence).
#[derive(Debug, PartialEq)]
pub enum Key {
    /// The tab key.
    Tab,
    /// The enter key.
    Return,
}

#[cfg(test)]
mod tests {

    use crate::{
        model::{
            entry::{EntryBody, EntryHead},
            uuid,
        },
        AutoTypeSequenceParser, Key, Part, Sequence,
    };

    fn default_head() -> EntryHead {
        let mut entry_head = EntryHead::new(
            [0u8; uuid::SIZE].into(),
            [1u8; uuid::SIZE].into(),
            "This is a test entry".into(),
            [2u8; uuid::SIZE].into(),
        );
        let _ = entry_head.set_web_address("https://example.org".into());

        entry_head
    }

    fn default_body() -> EntryBody {
        let mut entry_body = EntryBody::new(
            [1u8; uuid::SIZE].into(),
            "SecretUsername".into(),
            "TopSecretPassword".into(),
        );
        let _ = entry_body.set_email("person@example.org".into());

        entry_body
    }

    #[test]
    fn test_parse_sequence() {
        let entry_head = default_head();
        let entry_body = default_body();

        let sequence = "[username]abcd<tab>[password]<enter>";

        let result = AutoTypeSequenceParser::parse_sequence(sequence, &entry_head, &entry_body)
            .expect("Parsing default sequence should not fail");

        let expected = Sequence::with(vec![
            Part::Field("SecretUsername".into()),
            Part::Literal("abcd".into()),
            Part::Key(Key::Tab),
            Part::Field("TopSecretPassword".into()),
            Part::Key(Key::Return),
        ]);

        assert_eq!(result, expected);

        let sequence = "[email]abcde[title]";

        let result = AutoTypeSequenceParser::parse_sequence(sequence, &entry_head, &entry_body)
            .expect("Parsing should not fail");

        let expected = Sequence::with(vec![
            Part::Field("person@example.org".into()),
            Part::Literal("abcde".into()),
            Part::Field("This is a test entry".into()),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_invalid_sequence() {
        let entry_head = default_head();
        let entry_body = default_body();

        let invalid_sequences = vec!["][", "[]", "><", "<>", "]username[", ">tab<"];

        for sequence in invalid_sequences {
            let _ = AutoTypeSequenceParser::parse_sequence(sequence, &entry_head, &entry_body)
                .expect_err("Parsing invalid sequence should fail");
        }
    }

    #[test]
    fn test_validator() {
        let sequence = "[a]b<c>";
        assert!(AutoTypeSequenceParser::validate_sequence(sequence));
        let sequence = "><aa]>h";
        assert!(!AutoTypeSequenceParser::validate_sequence(sequence));
    }
}
