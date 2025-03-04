use super::generic_digest::{GenericDigest, HashDigest};
use ::md5::Md5;
use nu_protocol::{Example, Span, Value};

pub type HashMd5 = GenericDigest<Md5>;

impl HashDigest for Md5 {
    fn name() -> &'static str {
        "md5"
    }

    fn examples() -> Vec<Example> {
        vec![
            Example {
                description: "md5 encode a string",
                example: "echo 'abcdefghijklmnopqrstuvwxyz' | hash md5",
                result: Some(Value::String {
                    val: "c3fcd3d76192e4007dfb496cca67e13b".to_owned(),
                    span: Span::unknown(),
                }),
            },
            Example {
                description: "md5 encode a file",
                example: "open ./nu_0_24_1_windows.zip | hash md5",
                result: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::generic_digest;

    #[test]
    fn test_examples() {
        crate::test_examples(HashMd5::default())
    }

    #[test]
    fn hash_string() {
        let binary = Value::String {
            val: "abcdefghijklmnopqrstuvwxyz".to_owned(),
            span: Span::unknown(),
        };
        let expected = Value::String {
            val: "c3fcd3d76192e4007dfb496cca67e13b".to_owned(),
            span: Span::unknown(),
        };
        let actual = generic_digest::action::<Md5>(&binary);
        assert_eq!(actual, expected);
    }

    #[test]
    fn hash_bytes() {
        let binary = Value::Binary {
            val: vec![0xC0, 0xFF, 0xEE],
            span: Span::unknown(),
        };
        let expected = Value::String {
            val: "5f80e231382769b0102b1164cf722d83".to_owned(),
            span: Span::unknown(),
        };
        let actual = generic_digest::action::<Md5>(&binary);
        assert_eq!(actual, expected);
    }
}
