//! Purpose: the rule-presentation parse pass (add-rule-lens phase A) — the
//! additive format 1.2 section is validated on its own: the declared input
//! domain, the attribute columns, the resolution policy, and each rule's
//! action and predicate.
//!
//! Responsibilities: enforce that `domain` names a declared schema object;
//! that columns are idents, unique, and typed `int`/`bool`/`string`; that
//! `policy` is absent or one of the three declared policies — never
//! defaulted (design.md Decision 3); that every rule carries a non-empty
//! action (the action must be total on X); and that every predicate stays
//! inside the tiny decidable language (`super::predicate`). Rejections are
//! labeled with the offending element's JSON pointer.
//!
//! Rationale: the section is declared data — rule order is priority and is
//! preserved verbatim (never sorted, unlike equations), and an absent
//! policy means no precedence rather than a default. Structural
//! malformations (missing `columns`/`rules`/`name` fields) remain serde
//! `Syntax` errors; the semantic codes are the stable contract.

use super::predicate::compile_predicate;
use super::reject;
use super::schema::check_ident;
use super::spans::Spans;
use crate::error::{ErrorCode, ParseError};
use crate::model::{ColumnType, Ident, Policy, Rule, RulePresentation};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// The raw 1.2 section: presentation strings and untyped predicates.
#[derive(Deserialize)]
pub(super) struct RawRulePresentation {
    pub(super) domain: String,
    pub(super) columns: Vec<RawColumn>,
    pub(super) rules: Vec<RawRule>,
    #[serde(default)]
    pub(super) policy: Option<Value>,
}

#[derive(Deserialize)]
pub(super) struct RawColumn {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) column_type: String,
}

#[derive(Deserialize)]
pub(super) struct RawRule {
    pub(super) name: String,
    #[serde(default)]
    pub(super) predicate: Value,
    #[serde(default)]
    pub(super) action: Option<String>,
}

/// Build the typed presentation: declared domain and columns, declared
/// policy (or none), and rules in written order — rule order is priority,
/// so it is preserved, never sorted.
pub(super) fn build_rule_presentation(
    raw: &RawRulePresentation,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<RulePresentation, ParseError> {
    let domain = declared_domain(&raw.domain, objects, spans)?;
    let columns = declared_columns(&raw.columns, spans)?;
    let policy = declared_policy(raw.policy.as_ref(), spans)?;
    let rules = declared_rules(&raw.rules, &columns, spans)?;
    Ok(RulePresentation {
        domain,
        columns,
        rules,
        policy,
    })
}

/// The input domain X must be a declared schema object.
fn declared_domain(
    domain: &str,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<Ident, ParseError> {
    let ident = Ident(domain.to_string());
    if objects.contains_key(&ident) {
        return Ok(ident);
    }
    let message = format!("rule presentation domain `{domain}` is not a declared schema object");
    Err(reject(
        spans,
        "/rule_presentation/domain",
        ErrorCode::UndeclaredRuleDomain,
        message,
    ))
}

/// The attribute columns: idents, unique, and typed `int`/`bool`/`string`.
fn declared_columns(
    columns: &[RawColumn],
    spans: &Spans,
) -> Result<BTreeMap<Ident, ColumnType>, ParseError> {
    let mut declared = BTreeMap::new();
    for (i, raw) in columns.iter().enumerate() {
        let base = format!("/rule_presentation/columns/{i}");
        check_ident(spans, &format!("{base}/name"), &raw.name)?;
        let name = Ident(raw.name.clone());
        if declared.contains_key(&name) {
            let message = format!("duplicate column ident `{}`", raw.name);
            let pointer = format!("{base}/name");
            return Err(reject(spans, &pointer, ErrorCode::DuplicateIdent, message));
        }
        let Some(column_type) = ColumnType::parse(&raw.column_type) else {
            let message = format!(
                "invalid type `{}` for column `{}`: must be `int`, `bool`, or `string`",
                raw.column_type, raw.name
            );
            let pointer = format!("{base}/type");
            return Err(reject(
                spans,
                &pointer,
                ErrorCode::InvalidColumnType,
                message,
            ));
        };
        declared.insert(name, column_type);
    }
    Ok(declared)
}

/// The declared policy: absent means *no* policy (never a default); a
/// present policy must be one of the three spellings.
fn declared_policy(raw: Option<&Value>, spans: &Spans) -> Result<Option<Policy>, ParseError> {
    const POINTER: &str = "/rule_presentation/policy";
    let Some(value) = raw else {
        return Ok(None);
    };
    let Some(text) = value.as_str() else {
        let message = format!("a rule presentation's policy must be a string, got `{value}`");
        return Err(reject(spans, POINTER, ErrorCode::InvalidPolicy, message));
    };
    let Some(policy) = Policy::parse(text) else {
        let message = format!(
            "invalid policy `{text}`: must be `PriorityOrder`, `FirstMatch`, or `DeclaredDisjoint`"
        );
        return Err(reject(spans, POINTER, ErrorCode::InvalidPolicy, message));
    };
    Ok(Some(policy))
}

/// The rules in written order: unique idents, a required non-empty action,
/// and a predicate compiled against the declared columns.
fn declared_rules(
    rules: &[RawRule],
    columns: &BTreeMap<Ident, ColumnType>,
    spans: &Spans,
) -> Result<Vec<Rule>, ParseError> {
    let mut declared = Vec::with_capacity(rules.len());
    let mut names: BTreeMap<Ident, ()> = BTreeMap::new();
    for (i, raw) in rules.iter().enumerate() {
        let base = format!("/rule_presentation/rules/{i}");
        check_ident(spans, &format!("{base}/name"), &raw.name)?;
        let name = Ident(raw.name.clone());
        if names.insert(name.clone(), ()).is_some() {
            let message = format!("duplicate rule ident `{}`", raw.name);
            let pointer = format!("{base}/name");
            return Err(reject(spans, &pointer, ErrorCode::DuplicateIdent, message));
        }
        let action = declared_action(raw, &base, spans)?;
        let pointer = format!("{base}/predicate");
        declared.push(Rule {
            predicate: compile_predicate(&raw.predicate, columns, &pointer, &raw.name, spans)?,
            name,
            action,
        });
    }
    Ok(declared)
}

/// One rule's action: required and non-empty — a rule that assigns nothing
/// leaves the action partial, which the presentation cannot express.
fn declared_action(raw: &RawRule, base: &str, spans: &Spans) -> Result<String, ParseError> {
    if let Some(action) = raw.action.as_deref().filter(|a| !a.trim().is_empty()) {
        return Ok(action.to_string());
    }
    let pointer = if raw.action.is_some() {
        format!("{base}/action")
    } else {
        base.to_string()
    };
    let message = format!(
        "rule `{}` assigns no action: every rule must assign a non-empty action \
         (the action is a required total function on the input domain)",
        raw.name
    );
    Err(reject(spans, &pointer, ErrorCode::MissingAction, message))
}
