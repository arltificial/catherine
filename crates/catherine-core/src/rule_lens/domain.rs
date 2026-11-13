//! Purpose: the rule lens's input domain and predicate evaluator
//! (openspec change `add-rule-lens` phase B; design.md Decisions 2/3) —
//! the declared finite enumeration of X and the structural semantics of
//! its tiny decidable predicate language.
//!
//! Responsibilities: [`RuleDomain`] binds the presentation's typed columns
//! to caller-supplied finite value lists (validated: every column has
//! values, every value matches the column type, no undeclared column) and
//! enumerates inputs in the oracle's `itertools.product` order — or, via
//! [`RuleDomain::from_tuples`], stores an exact input set (width- and
//! type-validated, in the caller's order) for callers whose domain is
//! genuinely enumerated, not a product (the `rules --project` bridge, and
//! a future extraction); [`evaluate`] implements comparisons
//! (`eq`/`ne`/`lt`/`le`/`gt`/`ge`, strings byte-wise, bools
//! `false < true`), `mod` as a non-negative remainder, and
//! `and`/`or`/`not`.
//!
//! Rationale: X is declared data, not the instance rows — the rule lens
//! analyses the table against the attribute alphabet, so the caller must
//! supply the finite values in scope (bounded search never invents an
//! infinite `int`/`string` domain). A projection of stored rows is the
//! one case where the caller *does* hold the exact inputs: enumerating
//! their product would invent combinations no row has and report them as
//! gaps, so `from_tuples` stores exactly what it was given and never
//! expands. Value lists of different lengths make the enumeration
//! mixed-radix; the first declared column varies slowest, matching
//! `itertools.product` in the oracle so gap examples and equivalence
//! witnesses are identical across sim and kernel. Evaluation is total
//! over well-typed presentations (the parser guarantees literal and
//! column agree; a hand-built mismatch evaluates as unequal and
//! unordered, never panics).

use crate::model::{CmpOp, ColumnType, Ident, Literal, Predicate, RulePresentation};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// One input of X: its values in the domain's declared column order.
pub type RuleInput = Vec<Literal>;

/// The declared finite enumeration of the rule presentation's input
/// domain X. Column order is the presentation's key order (`columns` is a
/// `BTreeMap`, so sorted); values are as supplied by the caller. A domain
/// is either a product of per-column value lists ([`Self::from_presentation`])
/// or an exact input set ([`Self::from_tuples`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDomain {
    columns: Vec<Ident>,
    values: Vec<Vec<Literal>>,
    positions: BTreeMap<Ident, usize>,
    exact: Option<Vec<RuleInput>>,
}

/// A labeled rejection of a domain that does not match its presentation.
/// The rule lens never guesses a value list or a column type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleDomainError {
    /// The presentation declares a column the caller supplied no values
    /// for.
    MissingValues(Ident),
    /// The caller supplied values for a column the presentation never
    /// declared.
    UnexpectedColumn(Ident),
    /// A supplied literal does not have its column's declared type.
    WrongType {
        /// The offending column.
        column: Ident,
        /// The column's declared type.
        expected: ColumnType,
    },
    /// An exact input has the wrong number of values for the
    /// presentation's column list.
    WrongWidth {
        /// The presentation's declared column count.
        expected: usize,
        /// The input's value count.
        found: usize,
    },
}

impl fmt::Display for RuleDomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValues(column) => {
                write!(formatter, "no values supplied for column `{}`", column.0)
            }
            Self::UnexpectedColumn(column) => write!(
                formatter,
                "column `{}` is not declared by the rule presentation",
                column.0
            ),
            Self::WrongType { column, expected } => write!(
                formatter,
                "a value supplied for column `{}` is not `{}`",
                column.0,
                expected.as_str()
            ),
            Self::WrongWidth { expected, found } => write!(
                formatter,
                "an input has {found} value(s), but the presentation declares {expected} column(s)"
            ),
        }
    }
}

impl std::error::Error for RuleDomainError {}

impl RuleDomain {
    /// Bind the presentation's columns to the supplied finite value lists.
    /// Every declared column needs an entry (an empty list means X has no
    /// inputs on that column); every value must match its column's type.
    pub fn from_presentation(
        presentation: &RulePresentation,
        mut supplied: BTreeMap<Ident, Vec<Literal>>,
    ) -> Result<Self, RuleDomainError> {
        let mut columns = Vec::with_capacity(presentation.columns.len());
        let mut values = Vec::with_capacity(presentation.columns.len());
        let mut positions = BTreeMap::new();
        for (position, (column, column_type)) in presentation.columns.iter().enumerate() {
            let Some(list) = supplied.remove(column) else {
                return Err(RuleDomainError::MissingValues(column.clone()));
            };
            if list.iter().any(|value| !matches_type(value, *column_type)) {
                return Err(RuleDomainError::WrongType {
                    column: column.clone(),
                    expected: *column_type,
                });
            }
            positions.insert(column.clone(), position);
            columns.push(column.clone());
            values.push(list);
        }
        if let Some(undeclared) = supplied.keys().next() {
            return Err(RuleDomainError::UnexpectedColumn(undeclared.clone()));
        }
        Ok(Self {
            columns,
            values,
            positions,
            exact: None,
        })
    }

    /// Store an exact input set: every tuple must have one value per
    /// declared column, each of its column's declared type. The inputs are
    /// kept **in the order given** (this constructor never sorts or
    /// deduplicates — the caller's set semantics, the caller's order), so
    /// dense witness indices are exactly the caller's enumeration.
    pub fn from_tuples(
        presentation: &RulePresentation,
        inputs: Vec<RuleInput>,
    ) -> Result<Self, RuleDomainError> {
        let columns: Vec<Ident> = presentation.columns.keys().cloned().collect();
        let types: Vec<ColumnType> = presentation.columns.values().copied().collect();
        for input in &inputs {
            if input.len() != columns.len() {
                return Err(RuleDomainError::WrongWidth {
                    expected: columns.len(),
                    found: input.len(),
                });
            }
            for ((value, column_type), column) in input.iter().zip(&types).zip(&columns) {
                if !matches_type(value, *column_type) {
                    return Err(RuleDomainError::WrongType {
                        column: column.clone(),
                        expected: *column_type,
                    });
                }
            }
        }
        let positions = columns
            .iter()
            .cloned()
            .enumerate()
            .map(|(position, column)| (column, position))
            .collect();
        Ok(Self {
            columns,
            values: Vec::new(),
            positions,
            exact: Some(inputs),
        })
    }

    /// The domain's columns, in enumeration order.
    pub fn columns(&self) -> &[Ident] {
        &self.columns
    }

    /// The number of declared values per column — the exact shape of the
    /// enumeration (the rule lens's analogue of a search scope's rows).
    /// An exact set has no per-column list shape, so this reports the
    /// distinct value count per column (the closest honest restatement;
    /// the cardinality is [`Self::inputs`]`.len()`, not their product).
    pub fn column_sizes(&self) -> BTreeMap<Ident, usize> {
        match &self.exact {
            Some(inputs) => self
                .columns
                .iter()
                .enumerate()
                .map(|(position, column)| {
                    let distinct: BTreeSet<&Literal> = inputs
                        .iter()
                        .filter_map(|input| input.get(position))
                        .collect();
                    (column.clone(), distinct.len())
                })
                .collect(),
            None => self
                .columns
                .iter()
                .cloned()
                .zip(self.values.iter().map(Vec::len))
                .collect(),
        }
    }

    /// Every input of X, in declared enumeration order (first column
    /// varies slowest; an empty value list makes X empty). An exact set
    /// yields its stored inputs in the order the caller supplied them.
    pub fn inputs(&self) -> Vec<RuleInput> {
        self.enumerate_within(usize::MAX)
            .expect("the full domain enumeration cannot exhaust usize::MAX inputs")
    }

    /// Enumerate at most `max_candidates` inputs. `Ok` means exhaustion
    /// completed within the budget; `Err(explored)` means the budget
    /// expired after exactly `explored` candidates — the caller must
    /// report `UNKNOWN { scope }`, never a partial analysis.
    pub(crate) fn enumerate_within(&self, max_candidates: usize) -> Result<Vec<RuleInput>, usize> {
        if max_candidates == 0 {
            return Err(0);
        }
        if let Some(inputs) = &self.exact {
            return if inputs.len() <= max_candidates {
                Ok(inputs.clone())
            } else {
                Err(max_candidates)
            };
        }
        if self.values.iter().any(Vec::is_empty) {
            return Ok(Vec::new());
        }
        let widths: Vec<usize> = self.values.iter().map(Vec::len).collect();
        let mut digits = vec![0usize; widths.len()];
        let mut inputs = Vec::new();
        loop {
            if inputs.len() == max_candidates {
                return Err(inputs.len());
            }
            inputs.push(self.input_at(&digits));
            if !increment(&mut digits, &widths) {
                return Ok(inputs);
            }
        }
    }

    /// The input at one mixed-radix position.
    fn input_at(&self, digits: &[usize]) -> RuleInput {
        digits
            .iter()
            .zip(&self.values)
            .map(|(&digit, values)| values[digit].clone())
            .collect()
    }

    fn value<'a>(&self, input: &'a [Literal], column: &Ident) -> Option<&'a Literal> {
        let position = *self.positions.get(column)?;
        input.get(position)
    }
}

/// Whether a literal is of the column's declared type.
fn matches_type(value: &Literal, column_type: ColumnType) -> bool {
    matches!(
        (value, column_type),
        (Literal::Int(_), ColumnType::Int)
            | (Literal::Bool(_), ColumnType::Bool)
            | (Literal::String(_), ColumnType::String)
    )
}

/// Advance a mixed-radix counter (last digit fastest); `false` when it
/// wraps, meaning every combination was visited.
fn increment(digits: &mut [usize], widths: &[usize]) -> bool {
    for (digit, width) in digits.iter_mut().zip(widths).rev() {
        *digit += 1;
        if *digit < *width {
            return true;
        }
        *digit = 0;
    }
    false
}

/// Evaluate a predicate on one input aligned to `domain`'s column order.
/// The presentation is assumed well-typed (the parser enforces it); a
/// column or literal outside the domain evaluates as false rather than
/// panicking.
pub fn evaluate(predicate: &Predicate, input: &[Literal], domain: &RuleDomain) -> bool {
    match predicate {
        Predicate::Cmp { column, op, value } => domain
            .value(input, column)
            .is_some_and(|actual| compare(actual, value, *op)),
        Predicate::Mod {
            column,
            modulus,
            equals,
        } => {
            let Some(Literal::Int(value)) = domain.value(input, column) else {
                return false;
            };
            *modulus > 0 && value.rem_euclid(*modulus) == *equals
        }
        Predicate::And(args) => args.iter().all(|arg| evaluate(arg, input, domain)),
        Predicate::Or(args) => args.iter().any(|arg| evaluate(arg, input, domain)),
        Predicate::Not(arg) => !evaluate(arg, input, domain),
    }
}

/// One typed comparison. Strings compare byte-wise (Rust `str` order);
/// bools order `false < true`; mismatched literal types are unequal and
/// unordered.
fn compare(left: &Literal, right: &Literal, op: CmpOp) -> bool {
    match (left, right) {
        (Literal::Int(left), Literal::Int(right)) => ordered(left, right, op),
        (Literal::Bool(left), Literal::Bool(right)) => ordered(left, right, op),
        (Literal::String(left), Literal::String(right)) => ordered(left, right, op),
        _ => matches!(op, CmpOp::Ne),
    }
}

/// Apply a comparison operator to two ordered values.
fn ordered<T: Ord>(left: &T, right: &T, op: CmpOp) -> bool {
    match op {
        CmpOp::Eq => left == right,
        CmpOp::Ne => left != right,
        CmpOp::Lt => left < right,
        CmpOp::Le => left <= right,
        CmpOp::Gt => left > right,
        CmpOp::Ge => left >= right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two typed columns, sorted by name (`age` < `name`) — the exact-set
    /// tests' presentation.
    fn presentation() -> RulePresentation {
        RulePresentation {
            domain: Ident("Input".to_string()),
            columns: BTreeMap::from([
                (Ident("age".to_string()), ColumnType::Int),
                (Ident("name".to_string()), ColumnType::String),
            ]),
            rules: Vec::new(),
            policy: None,
        }
    }

    fn tuple(age: i64, name: &str) -> RuleInput {
        vec![Literal::Int(age), Literal::String(name.to_string())]
    }

    /// The exact set is stored and enumerated as given, never expanded to
    /// a product; the budget reports the count it could not finish.
    #[test]
    fn exact_tuples_enumerate_in_the_order_given() {
        let inputs = vec![tuple(2, "b"), tuple(1, "a")];
        let domain = RuleDomain::from_tuples(&presentation(), inputs.clone()).expect("valid set");
        assert_eq!(domain.inputs(), inputs, "the exact set, as supplied");
        assert_eq!(domain.enumerate_within(2), Ok(inputs.clone()));
        assert_eq!(domain.enumerate_within(1), Err(1), "a budget expiry");
        assert_eq!(
            domain.column_sizes(),
            BTreeMap::from([
                (Ident("age".to_string()), 2),
                (Ident("name".to_string()), 2),
            ]),
            "an exact set reports its distinct values per column"
        );
    }

    /// A literal of the wrong variant is rejected with its column and the
    /// expected type — the domain never guesses.
    #[test]
    fn an_exact_tuple_of_the_wrong_type_is_rejected() {
        let inputs = vec![vec![
            Literal::String("1".to_string()),
            Literal::String("a".to_string()),
        ]];
        assert_eq!(
            RuleDomain::from_tuples(&presentation(), inputs),
            Err(RuleDomainError::WrongType {
                column: Ident("age".to_string()),
                expected: ColumnType::Int,
            })
        );
    }

    /// An exact tuple with a value per declared column only.
    #[test]
    fn an_exact_tuple_of_the_wrong_width_is_rejected() {
        assert_eq!(
            RuleDomain::from_tuples(&presentation(), vec![tuple(1, "a"), vec![Literal::Int(1)]]),
            Err(RuleDomainError::WrongWidth {
                expected: 2,
                found: 1,
            })
        );
    }

    /// Predicate evaluation reads exact inputs through the same column
    /// positions as a product domain.
    #[test]
    fn exact_tuples_evaluate_predicates_by_position() {
        let inputs = vec![tuple(1, "a"), tuple(2, "b")];
        let domain = RuleDomain::from_tuples(&presentation(), inputs).expect("valid set");
        let predicate = Predicate::Cmp {
            column: Ident("age".to_string()),
            op: CmpOp::Eq,
            value: Literal::Int(2),
        };
        assert!(!evaluate(&predicate, &tuple(1, "a"), &domain));
        assert!(evaluate(&predicate, &tuple(2, "b"), &domain));
    }
}
