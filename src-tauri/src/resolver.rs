//! Template variable resolution (README §14/§41).
//!
//! Deliberately has zero DB dependency: given a set of already-loaded scope maps, it just
//! does string substitution. The DB-facing side (`store::variable_store::gather_scope_chain`)
//! is a thin adapter that loads rows into these maps.
//!
//! Precedence (highest wins first): Runtime > Request > Folder > Collection > Environment > Global.
//! Runtime is never persisted — it only ever exists as an in-memory overlay supplied by the
//! caller (today: nothing; later: the pre-request script engine), which is why it's not a
//! `VariableScope` enum variant in `models.rs`.

use std::collections::HashMap;

pub type VarMap = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct ScopeChain<'a> {
    pub runtime: Option<&'a VarMap>,
    pub request: Option<&'a VarMap>,
    pub folder: Option<&'a VarMap>,
    pub collection: Option<&'a VarMap>,
    pub environment: Option<&'a VarMap>,
    pub global: Option<&'a VarMap>,
}

impl<'a> ScopeChain<'a> {
    fn tiers_high_to_low(&self) -> [Option<&'a VarMap>; 6] {
        [
            self.runtime,
            self.request,
            self.folder,
            self.collection,
            self.environment,
            self.global,
        ]
    }

    /// First-seen-wins merge across tiers, highest precedence first.
    fn merge(&self) -> VarMap {
        let mut merged = VarMap::new();
        for tier in self.tiers_high_to_low().into_iter().flatten() {
            for (key, value) in tier {
                merged.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        merged
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResolvedTemplate {
    pub resolved: String,
    /// Keys referenced by `{{key}}` that had no value in any scope — left as literal text.
    pub missing: Vec<String>,
}

/// Resolve every `{{key}}` in `template` against the merged scope chain.
///
/// Variable *values* are pre-resolved against each other first (bounded to `MAX_PASSES`
/// iterations, cycle-safe) so `baseUrl = "https://{{host}}"` composes correctly — this is
/// what the spec calls "nested/combined values".
pub fn resolve_template(template: &str, chain: &ScopeChain) -> ResolvedTemplate {
    let merged = resolve_values_against_each_other(chain.merge());
    let (resolved, missing) = substitute(template, &merged);
    ResolvedTemplate { resolved, missing }
}

const MAX_PASSES: usize = 5;

fn resolve_values_against_each_other(mut vars: VarMap) -> VarMap {
    for _ in 0..MAX_PASSES {
        let mut changed = false;
        let snapshot = vars.clone();
        for value in vars.values_mut() {
            let (resolved, _missing) = substitute(value, &snapshot);
            if resolved != *value {
                *value = resolved;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    vars
}

/// Scans for `{{key}}` (no nested braces, keys trimmed of surrounding whitespace) and
/// substitutes from `vars`. Unknown keys are left untouched and reported in `missing`.
fn substitute(template: &str, vars: &VarMap) -> (String, Vec<String>) {
    let mut output = String::with_capacity(template.len());
    let mut missing = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        match after_open.find("}}") {
            Some(end) => {
                let key = after_open[..end].trim();
                match vars.get(key).cloned().or_else(|| dynamic_value(key)) {
                    Some(value) => output.push_str(&value),
                    None => {
                        missing.push(key.to_string());
                        output.push_str(&rest[start..start + 4 + end]);
                    }
                }
                rest = &after_open[end + 2..];
            }
            None => {
                // Unterminated "{{" — treat the rest of the string as literal.
                output.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
    }
    output.push_str(rest);
    (output, missing)
}

/// Dynamic variables (README §41 "Dynamic variables" / task-pack LP-0210): generated fresh
/// on every resolution, never persisted. Postman's `$`-prefix convention, kept intentionally
/// small — add a case here to extend it, not a new subsystem. An unrecognized `$name` is
/// *not* claimed here — it falls through to the normal "missing" reporting like any other
/// unknown key, so a typo doesn't silently start meaning something.
fn dynamic_value(key: &str) -> Option<String> {
    match key {
        "$guid" => Some(uuid::Uuid::new_v4().to_string()),
        "$timestamp" => Some(chrono::Utc::now().timestamp().to_string()),
        "$isoTimestamp" => Some(chrono::Utc::now().to_rfc3339()),
        // Sourced from a fresh UUID's random bytes rather than pulling in the `rand` crate
        // for one call site — good enough for "give me a plausible-looking test value",
        // which is all a mock/dynamic variable needs to be.
        "$randomInt" => {
            let bytes = uuid::Uuid::new_v4().into_bytes();
            let n = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            Some((n % 1000).to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> VarMap {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn basic_resolution_substitutes_known_keys() {
        let global = map(&[("baseUrl", "https://api.example.com")]);
        let chain = ScopeChain { global: Some(&global), ..Default::default() };

        let result = resolve_template("{{baseUrl}}/users", &chain);

        assert_eq!(result.resolved, "https://api.example.com/users");
        assert!(result.missing.is_empty());
    }

    #[test]
    fn missing_variable_is_left_literal_and_reported() {
        let chain = ScopeChain::default();

        let result = resolve_template("{{baseUrl}}/users/{{userId}}", &chain);

        assert_eq!(result.resolved, "{{baseUrl}}/users/{{userId}}");
        assert_eq!(result.missing, vec!["baseUrl".to_string(), "userId".to_string()]);
    }

    #[test]
    fn multiple_scopes_each_contribute_distinct_keys() {
        let global = map(&[("baseUrl", "https://api.example.com")]);
        let environment = map(&[("token", "env-token")]);
        let request = map(&[("userId", "42")]);
        let chain = ScopeChain {
            global: Some(&global),
            environment: Some(&environment),
            request: Some(&request),
            ..Default::default()
        };

        let result = resolve_template("{{baseUrl}}/users/{{userId}}?token={{token}}", &chain);

        assert_eq!(result.resolved, "https://api.example.com/users/42?token=env-token");
        assert!(result.missing.is_empty());
    }

    #[test]
    fn scope_precedence_request_beats_environment_beats_global() {
        let global = map(&[("env", "global")]);
        let environment = map(&[("env", "environment")]);
        let request = map(&[("env", "request")]);

        let request_wins = resolve_template(
            "{{env}}",
            &ScopeChain {
                global: Some(&global),
                environment: Some(&environment),
                request: Some(&request),
                ..Default::default()
            },
        );
        assert_eq!(request_wins.resolved, "request");

        let environment_wins = resolve_template(
            "{{env}}",
            &ScopeChain { global: Some(&global), environment: Some(&environment), ..Default::default() },
        );
        assert_eq!(environment_wins.resolved, "environment");

        let global_wins = resolve_template("{{env}}", &ScopeChain { global: Some(&global), ..Default::default() });
        assert_eq!(global_wins.resolved, "global");
    }

    #[test]
    fn runtime_outranks_every_persisted_scope() {
        let global = map(&[("env", "global")]);
        let runtime = map(&[("env", "runtime")]);

        let result = resolve_template(
            "{{env}}",
            &ScopeChain { global: Some(&global), runtime: Some(&runtime), ..Default::default() },
        );

        assert_eq!(result.resolved, "runtime");
    }

    #[test]
    fn nested_values_compose_through_recursive_resolution() {
        let global = map(&[("host", "api.example.com"), ("baseUrl", "https://{{host}}")]);
        let chain = ScopeChain { global: Some(&global), ..Default::default() };

        let result = resolve_template("{{baseUrl}}/users", &chain);

        assert_eq!(result.resolved, "https://api.example.com/users");
    }

    #[test]
    fn cyclic_values_terminate_without_hanging() {
        let global = map(&[("a", "{{b}}"), ("b", "{{a}}")]);
        let chain = ScopeChain { global: Some(&global), ..Default::default() };

        // Must not loop forever; exact leftover text isn't important, termination is.
        let result = resolve_template("{{a}}", &chain);
        assert!(result.resolved.contains("{{") || !result.resolved.is_empty());
    }

    #[test]
    fn environment_switching_changes_resolution_without_touching_the_template() {
        let template = "{{baseUrl}}/health";
        let dev = map(&[("baseUrl", "https://dev.example.com")]);
        let prod = map(&[("baseUrl", "https://prod.example.com")]);

        let dev_result = resolve_template(template, &ScopeChain { environment: Some(&dev), ..Default::default() });
        let prod_result = resolve_template(template, &ScopeChain { environment: Some(&prod), ..Default::default() });

        assert_eq!(dev_result.resolved, "https://dev.example.com/health");
        assert_eq!(prod_result.resolved, "https://prod.example.com/health");
    }

    #[test]
    fn dollar_guid_resolves_to_a_valid_uuid_and_changes_every_call() {
        let chain = ScopeChain::default();
        let first = resolve_template("{{$guid}}", &chain);
        let second = resolve_template("{{$guid}}", &chain);
        assert!(first.missing.is_empty());
        assert!(uuid::Uuid::parse_str(&first.resolved).is_ok(), "not a valid UUID: {}", first.resolved);
        assert_ne!(first.resolved, second.resolved, "each resolution should mint a fresh value");
    }

    #[test]
    fn dollar_timestamp_resolves_to_a_number() {
        let result = resolve_template("{{$timestamp}}", &ScopeChain::default());
        assert!(result.missing.is_empty());
        assert!(result.resolved.parse::<i64>().is_ok(), "not numeric: {}", result.resolved);
    }

    #[test]
    fn dollar_iso_timestamp_resolves_to_valid_rfc3339() {
        let result = resolve_template("{{$isoTimestamp}}", &ScopeChain::default());
        assert!(chrono::DateTime::parse_from_rfc3339(&result.resolved).is_ok());
    }

    #[test]
    fn dollar_random_int_resolves_to_a_bounded_number() {
        let result = resolve_template("{{$randomInt}}", &ScopeChain::default());
        let n: u32 = result.resolved.parse().unwrap();
        assert!(n < 1000);
    }

    #[test]
    fn a_persisted_variable_of_the_same_name_takes_precedence_over_a_dynamic_one() {
        // Dynamic names are only a fallback for keys with no scope-map entry — an explicit
        // variable literally named "$guid" (unusual, but not forbidden) should still win.
        let global = map(&[("$guid", "not-a-real-guid")]);
        let result = resolve_template("{{$guid}}", &ScopeChain { global: Some(&global), ..Default::default() });
        assert_eq!(result.resolved, "not-a-real-guid");
    }

    #[test]
    fn unrecognized_dollar_token_is_still_reported_missing() {
        let result = resolve_template("{{$notARealDynamicVar}}", &ScopeChain::default());
        assert_eq!(result.missing, vec!["$notARealDynamicVar".to_string()]);
        assert_eq!(result.resolved, "{{$notARealDynamicVar}}");
    }
}
