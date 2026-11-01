//! The question panel's calculator, as a service (PRD FR-7.75, Code Standards M14, Q5).
//!
//! A thin wrapper over [`mandi_domain::figures`]: the model's `calculate` tool and the numbers
//! read from a question come through here, so `mandi-assistant` does no arithmetic of its own. A
//! refusal becomes a service rejection with the error code of the domain error, which the tool
//! returns to the model (the Owner never sees it). Nothing is written to the books.
//!
//! Example (E2E-61): `calculate(Sum, [2,880 Dr, 18,684 Dr])` is 21,564 Dr.
//!
//! ```
//! use mandi_domain::core::money::Paisa;
//! use mandi_domain::figures::{CalcOp, Value};
//! use mandi_services::figures::calculate;
//! let dr = |r| Value::Balance(Paisa::from_rupees(r).unwrap());
//! assert_eq!(calculate(CalcOp::Sum, &[dr(2_880), dr(18_684)]).unwrap().result, dr(21_564));
//! ```

use mandi_domain::figures::{self, Calc, CalcOp, QuestionNumber, Value};

use crate::error::ServiceError;

/// One calculation with its working.
///
/// # Errors
/// A rejection with the code of the refusal: `ERR_NO_BASE` for a percent of 0, `ERR_OVERFLOW` for
/// a result that does not fit, `ERR_OUT_OF_RANGE` for the rest (see `FigureError`).
pub fn calculate(op: CalcOp, inputs: &[Value]) -> Result<Calc, ServiceError> {
    figures::calculate(op, inputs).map_err(|error| ServiceError::reject(error.code()))
}

/// The numbers the Owner typed in his question, with their units (D6-system-prompt 5.3).
pub fn question_numbers(text: &str) -> Vec<QuestionNumber> {
    figures::question_numbers(text)
}
