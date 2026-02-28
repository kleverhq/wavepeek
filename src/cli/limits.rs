use std::str::FromStr;

const UNLIMITED_LITERAL: &str = "unlimited";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimitArg {
    Numeric(usize),
    Unlimited,
}

impl LimitArg {
    pub const fn is_unlimited(&self) -> bool {
        matches!(self, Self::Unlimited)
    }

    pub const fn numeric(&self) -> Option<usize> {
        match self {
            Self::Numeric(value) => Some(*value),
            Self::Unlimited => None,
        }
    }
}

impl FromStr for LimitArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == UNLIMITED_LITERAL {
            return Ok(Self::Unlimited);
        }

        value
            .parse::<usize>()
            .map(Self::Numeric)
            .map_err(|_| format!("expected a non-negative integer or '{UNLIMITED_LITERAL}'"))
    }
}

#[cfg(test)]
mod tests {
    use super::LimitArg;

    #[test]
    fn limit_arg_parses_numeric_values() {
        let parsed = "42".parse::<LimitArg>().expect("numeric parse should work");
        assert_eq!(parsed, LimitArg::Numeric(42));
        assert_eq!(parsed.numeric(), Some(42));
        assert!(!parsed.is_unlimited());
    }

    #[test]
    fn limit_arg_parses_unlimited_literal() {
        let parsed = "unlimited"
            .parse::<LimitArg>()
            .expect("unlimited parse should work");
        assert_eq!(parsed, LimitArg::Unlimited);
        assert_eq!(parsed.numeric(), None);
        assert!(parsed.is_unlimited());
    }

    #[test]
    fn limit_arg_rejects_invalid_literals() {
        let error = "1.5"
            .parse::<LimitArg>()
            .expect_err("invalid limit should fail");
        assert_eq!(error, "expected a non-negative integer or 'unlimited'");
    }
}
