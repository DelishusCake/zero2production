use std::str::FromStr;

use regex::Regex;

use unicode_segmentation::UnicodeSegmentation;

use crate::Error;

const MAX_LEN: usize = 256;

#[derive(Debug)]
pub struct EmailAddress(String);

impl AsRef<str> for EmailAddress {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl FromStr for EmailAddress {
	type Err = Error;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		lazy_static::lazy_static! {
			static ref EMAIL_REGEX: Regex = Regex::new(r"^\w+@\w+\.\w+$").unwrap();
		}

		if value.trim().is_empty() {
			return Err(Error::ParsingError("Email address cannot be empty".into()));
		}
		if value.graphemes(true).count() > MAX_LEN {
			return Err(Error::ParsingError("Email address too long".into()));
		}
		if !EMAIL_REGEX.is_match(value) {
			return Err(Error::ParsingError("Email address of incorrect format".into()));
		}

		// Normalize
		let value = value.trim().to_lowercase();

		Ok(Self(value))
	}
}

#[cfg(test)]
mod tests {
	use claims::{assert_err, assert_ok};
	use super::*;

	#[derive(Debug, Clone)]
	struct VaidEmailFixture(pub String);

	impl quickcheck::Arbitrary for VaidEmailFixture {
		fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
			use fake::faker::internet::en::SafeEmail;
			use fake::Fake;

			let email: String = SafeEmail().fake_with_rng(g);
			Self(email)
		}
	}

	#[quickcheck_macros::quickcheck]
	fn safe_emails_valid(valid_email: VaidEmailFixture) -> bool {
		valid_email.0.parse::<EmailAddress>().is_ok()
	}

	#[test]
	fn long_email_valid() {
		let domain = "@test.com".to_string();
		let subject = "ё".repeat(256 - domain.len());
		let email = format!("{}{}", subject, domain);

		assert_ok!(email.parse::<EmailAddress>());
	}

	#[test]
	fn too_long_email_invalid() {
		let domain = "@test.com".to_string();
		let subject = "ё".repeat(258 - domain.len());
		let email = format!("{}{}", subject, domain);

		assert_err!(email.parse::<EmailAddress>());
	}

	#[test]
	fn blank_email_invalid() {
		let email = "    ";
		assert_err!(email.parse::<EmailAddress>());
	}

	#[test]
	fn empty_email_invalid() {
		let email = "";
		assert_err!(email.parse::<EmailAddress>());
	}

	#[test]
	fn domain_only_invalid() {
		let email = "test.com";
		assert_err!(email.parse::<EmailAddress>());
	}

	#[test]
	fn subject_only_invalid() {
		let email = "@test.com";
		assert_err!(email.parse::<EmailAddress>());
	}
}