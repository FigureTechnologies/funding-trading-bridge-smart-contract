use cosmwasm_std::Response;

pub trait AttributeExtractor {
    fn expect_attribute(&self, key: &str) -> &str;
    fn assert_attribute<S: Into<String>>(&self, key: &str, expected_value: S) {
        assert_eq!(
            expected_value.into(),
            self.expect_attribute(key),
            "expected the correct value for [{key}]",
        );
    }
    fn assert_attribute_with_message_prefix<S1: Into<String>, S2: Into<String>>(
        &self,
        key: &str,
        expected_value: S1,
        error_prefix: S2,
    ) {
        assert_eq!(
            expected_value.into(),
            self.expect_attribute(key),
            "{}: expected the correct value for [{key}]",
            error_prefix.into(),
        );
    }
}

impl<T> AttributeExtractor for Response<T> {
    fn expect_attribute(&self, key: &str) -> &str {
        self.attributes
            .iter()
            .find(|attr| attr.key.as_str() == key)
            .unwrap_or_else(|| panic!("expected attributes to contain key [{key}]"))
            .value
            .as_str()
    }
}
