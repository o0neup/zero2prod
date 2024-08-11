use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Arbitrary;

    #[quickcheck_macros::quickcheck]
    fn valid_email_is_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn invalid_cases_are_rejected() {
        let cases = ["namedomain.com", "@gmail.com", "", "  "];
        for case in &cases {
            assert_err!(SubscriberEmail::parse(case.to_string()));
        }
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let choices: Vec<String> = (0..g.size()).map(|_| SafeEmail().fake()).collect();
            match g.choose(choices.as_slice()) {
                Some(v) => Self(v.to_owned()),
                None => panic!("Can't happen!"),
            }
        }
    }
}
