//! TODO

use pest::{Parser, iterators::Pairs};

use crate::{EntryBody, EntryHead, PWDuckCoreError, SecString};

/// TODO
#[derive(Debug, Parser)]
#[grammar = "auto_type/auto_type.pest"]
pub struct AutoTypeSequenceParser;

impl AutoTypeSequenceParser {

    /// This checks if the sequence can be parsed. It will currently not check whether the fields or keys are valid.
    /// TODO
    pub fn validate_sequence(sequence: &str) -> bool {
        Self::parse(Rule::sequence, sequence).is_ok()
    }

    /// Parse the sequence and return the gathered information for the autotyper.
    /// TODO
    pub fn parse_sequence<'a>(
        sequence: &'a str,
        entry_head: &'a EntryHead,
        entry_body: &'a EntryBody,
    ) -> Result<Vec<SecString>, PWDuckCoreError> {
        let sequence = Self::parse(Rule::sequence, sequence)
            .map_err(|err| PWDuckCoreError::Error(format!("{}", err)))?;

        let mut key_sequence = Vec::new();

        Self::parse_inner(sequence, entry_head, entry_body, &mut key_sequence)?;

        Ok(key_sequence)
    }

    /// Parse a pair of rules and return the gathered information for the autotyper.
    /// TODO
    fn parse_inner<'a>(
        sequence: Pairs<Rule>,
        entry_head: &'a EntryHead,
        entry_body: &'a EntryBody,
        key_sequence: &mut Vec<SecString>,
    ) -> Result<(), PWDuckCoreError> {
        for pair in sequence {
            match pair.as_rule() {
                Rule::literal => key_sequence.push(pair.as_str().into()),
                Rule::field => match pair.as_str() {
                    "[title]" => key_sequence.push(entry_head.title().clone().into()),
                    "[username]" => key_sequence.push(entry_body.username().clone().into()),
                    "[password]" => key_sequence.push(entry_body.password().clone().into()),
                    "[email]" => key_sequence.push(entry_body.email().clone().into()),
                    _ => return Err(PWDuckCoreError::Error("No valid field".into())),
                },
                Rule::key => match pair.as_str() {
                    "<tab>" => key_sequence.push("\t".into()),
                    "<enter>" => key_sequence.push("\n".into()),
                    _ => return Err(PWDuckCoreError::Error("No valid key".into())),
                },
                Rule::sequence => Self::parse_inner(pair.into_inner(), entry_head, entry_body, key_sequence)?,
                _ => return Err(PWDuckCoreError::Error("Parse error".into())),
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {

    use crate::{AutoTypeSequenceParser, model::entry::{EntryHead, EntryBody}};

    fn default_head() -> EntryHead {
        let mut entry_head = EntryHead::new(
            vec![0u8].into(),
            "noparent".into(),
            "This is a test entry".into(),
            "body".into()
        );
        let _ = entry_head.set_web_address("https://example.org".into());

        entry_head
    }

    fn default_body() -> EntryBody {
        let mut entry_body = EntryBody::new(
            vec![0u8].into(),
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

        let expected = vec![
            "SecretUsername".into(),
            "abcd".into(),
            "\t".into(),
            "TopSecretPassword".into(),
            "\n".into(),
        ];

        assert_eq!(result, expected);

        let sequence = "[email]abcde[title]";

        let result = AutoTypeSequenceParser::parse_sequence(sequence, &entry_head, &entry_body)
            .expect("Parsing should not fail");

        let expected = vec![
            "person@example.org".into(),
            "abcde".into(),
            "This is a test entry".into(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_invalid_sequence() {
        let entry_head = default_head();
        let entry_body = default_body();

        let invalid_sequences = vec![
            "][",
            "[]",
            "><",
            "<>",
            "]username[",
            ">tab<",
        ];

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