use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::Validator;

pub struct ParseErrorsValidator;

impl Validator for ParseErrorsValidator {
    fn id(&self) -> CheckId {
        CheckId::ParseErrors
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        ctx.parse_failures.clone()
    }
}
