//! Pre-flight validation and post-write verification of putNode flow trees.
//!
//! `com.wm.lang.flow.FlowElement.create(Values)` builds the step tree from
//! the `type` key and returns `null` for any type it does not know -- the
//! step AND its whole subtree disappear without an error or a log line.
//! putNode then answers HTTP 200, the flow compiles, and the service runs
//! with the remaining steps (a `REPEAT` typed retry loop becomes nothing,
//! its body included). Unknown KEYS are ignored the same way, which is how
//! `label-expressions` on a BRANCH silently produces an unlabeled BRANCH
//! that fails at run time with `[ISC.0049.9009] Missing required property
//! switch`.
//!
//! The lists below are taken from the decompiled `FlowElement` (IS 12.1):
//! the `create()` factory and the `KEY_*` constants of each step class.

use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// Every `type` value `FlowElement.create()` instantiates (plus ROOT).
pub const KNOWN_STEP_TYPES: &[&str] = &[
    "ROOT",
    "INVOKE",
    "SEQUENCE",
    "BRANCH",
    "RETRY",
    "LOOP",
    "MAP",
    "EXIT",
    "SPAWN",
    "NOTIFY",
    "WAIT",
    "MAPCOPY",
    "MAPSET",
    "MAPDELETE",
    "MAPINVOKE",
    "MAPFOREACH",
    "WHILE",
    "BREAK",
    "CONTINUE",
    "SWITCH",
    "DO",
    "UNTIL",
];

/// Spellings people reach for that the factory rejects (dropped silently),
/// with the construct that actually exists.
const WRONG_TYPES: &[(&str, &str)] = &[
    (
        "REPEAT",
        "the retry/repeat step is type \"RETRY\" with keys \"count\", \"backoff\" (seconds between iterations) and \"repeat-on\" (\"SUCCESS\" | \"FAILURE\")",
    ),
    (
        "TRY",
        "TRY/CATCH/FINALLY are SEQUENCE steps with \"form\": \"TRY\" / \"CATCH\" / \"FINALLY\" (adjacent siblings)",
    ),
    (
        "CATCH",
        "TRY/CATCH/FINALLY are SEQUENCE steps with \"form\": \"TRY\" / \"CATCH\" / \"FINALLY\" (adjacent siblings)",
    ),
    (
        "FINALLY",
        "TRY/CATCH/FINALLY are SEQUENCE steps with \"form\": \"TRY\" / \"CATCH\" / \"FINALLY\" (adjacent siblings)",
    ),
    (
        "IF",
        "conditions are a BRANCH with \"evaluate-labels\": \"true\" whose children carry the expression in \"label\"",
    ),
    (
        "ELSEIF",
        "conditions are a BRANCH with \"evaluate-labels\": \"true\" whose children carry the expression in \"label\"",
    ),
    ("ELSE", "use a child labeled \"$default\" inside the BRANCH"),
    (
        "CASE",
        "a BRANCH case is a child step (usually a SEQUENCE) whose \"label\" is the switch value",
    ),
    (
        "MAPEMPTY",
        "not supported by the flow compiler; use MAPSET with an empty value or MAPDELETE",
    ),
];

/// Keys the step classes do not read (silently ignored), with the real key.
const WRONG_KEYS: &[(&str, &str)] = &[
    ("label-expressions", "evaluate-labels"),
    ("labelexpressions", "evaluate-labels"),
    ("evaluate_labels", "evaluate-labels"),
    ("repeat-interval", "backoff"),
    ("repeat_interval", "backoff"),
    ("back-off", "backoff"),
    ("interval", "backoff"),
    ("loop-on", "repeat-on"),
    ("repeat_on", "repeat-on"),
    ("retry-on", "repeat-on"),
    ("in_array", "in-array"),
    ("out_array", "out-array"),
    ("exit_on", "exit-on"),
    ("exit-from", "from"),
    ("validate_in", "validate-in"),
    ("validate_out", "validate-out"),
    ("failure_message", "failure-message"),
];

/// Walk a flow tree and collect everything IS would drop or misread.
/// An empty result means the tree is made only of constructs the flow
/// compiler instantiates.
pub fn validate_flow_tree(flow: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    walk(flow, "flow", &mut problems);
    problems
}

fn walk(node: &Value, path: &str, problems: &mut Vec<String>) {
    let Some(obj) = node.as_object() else {
        problems.push(format!("{path}: a flow step must be a JSON object"));
        return;
    };
    match obj.get("type").and_then(Value::as_str) {
        None => problems.push(format!(
            "{path}: missing \"type\" -- FlowElement.create() returns null for it and the step is dropped"
        )),
        Some(t) if KNOWN_STEP_TYPES.contains(&t) => {}
        Some(t) => {
            let hint = WRONG_TYPES
                .iter()
                .find(|(wrong, _)| wrong.eq_ignore_ascii_case(t))
                .map(|(_, hint)| format!("; {hint}"))
                .unwrap_or_default();
            problems.push(format!(
                "{path}: unknown step type \"{t}\" -- IS drops it and its whole subtree silently{hint}"
            ));
        }
    }
    for (key, real) in WRONG_KEYS {
        if obj.contains_key(*key) {
            problems.push(format!(
                "{path}: key \"{key}\" is not read by IS (silently ignored); the real key is \"{real}\""
            ));
        }
    }
    if let Some(children) = obj.get("nodes") {
        match children.as_array() {
            Some(items) => {
                for (i, child) in items.iter().enumerate() {
                    walk(child, &format!("{path}.nodes[{i}]"), problems);
                }
            }
            None => problems.push(format!(
                "{path}.nodes: must be a JSON array of steps (got {})",
                type_name(children)
            )),
        }
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Count steps by `type`, ROOT excluded. `MAP` steps without any child are
/// counted separately under `MAP(empty)`: IS does not persist them, so they
/// must not be expected back.
pub fn count_steps(flow: &Value) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    count_into(flow, true, &mut counts);
    counts
}

fn count_into(node: &Value, is_root: bool, counts: &mut BTreeMap<String, usize>) {
    let Some(obj) = node.as_object() else { return };
    let children = obj.get("nodes").and_then(Value::as_array);
    if !is_root {
        let t = obj.get("type").and_then(Value::as_str).unwrap_or("?");
        let key = if t == "MAP" && children.is_none_or(|c| c.is_empty()) {
            "MAP(empty)".to_string()
        } else {
            t.to_string()
        };
        *counts.entry(key).or_insert(0) += 1;
    }
    if let Some(items) = children {
        for child in items {
            count_into(child, false, counts);
        }
    }
}

/// Count every field declared in a record tree (`rec_fields`, recursively).
pub fn count_fields(record: &Value) -> usize {
    let Some(fields) = record.get("rec_fields").and_then(Value::as_array) else {
        return 0;
    };
    fields.iter().map(|f| 1 + count_fields(f)).sum()
}

/// Compare the step counts of the tree that was sent with the tree read
/// back after the write. Returns a JSON report; `status` is `"ok"` when
/// nothing was lost, `"mismatch"` otherwise (with the per-type deficit in
/// `missing`). Steps IS added on its own are reported under `extra` for
/// information only.
pub fn compare_flows(sent: &Value, stored: &Value) -> Value {
    let sent_counts = count_steps(sent);
    let stored_counts = count_steps(stored);
    let mut missing = Map::new();
    let mut extra = Map::new();
    for (t, &n) in &sent_counts {
        if t == "MAP(empty)" {
            continue;
        }
        let have = stored_counts.get(t).copied().unwrap_or(0);
        if have < n {
            missing.insert(t.clone(), json!(n - have));
        }
    }
    for (t, &n) in &stored_counts {
        let want = sent_counts.get(t).copied().unwrap_or(0);
        if n > want {
            extra.insert(t.clone(), json!(n - want));
        }
    }
    let status = if missing.is_empty() { "ok" } else { "mismatch" };
    let mut report = json!({
        "status": status,
        "sent": sent_counts,
        "stored": stored_counts,
    });
    if !missing.is_empty() {
        report["missing"] = Value::Object(missing);
    }
    if !extra.is_empty() {
        report["extra"] = Value::Object(extra);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(nodes: Value) -> Value {
        json!({"type": "ROOT", "version": "3.0", "cleanup": "true", "nodes": nodes})
    }

    #[test]
    fn a_clean_tree_has_no_problems() {
        let flow = tree(json!([
            {"type": "RETRY", "count": "3", "backoff": "1", "repeat-on": "SUCCESS", "nodes": [
                {"type": "BRANCH", "evaluate-labels": "true", "nodes": [
                    {"type": "EXIT", "label": "%n% == 1", "from": "$loop", "signal": "SUCCESS"}
                ]}
            ]},
            {"type": "SEQUENCE", "form": "TRY", "exit-on": "FAILURE", "nodes": []},
            {"type": "SEQUENCE", "form": "CATCH", "exit-on": "FAILURE", "nodes": [
                {"type": "INVOKE", "service": "pub.flow:getLastError", "validate-in": "$none", "validate-out": "$none", "nodes": [
                    {"type": "MAP", "mode": "OUTPUT", "nodes": [{"type": "MAPCOPY", "from": "/a;1;0", "to": "/b;1;0"}]}
                ]}
            ]}
        ]));
        assert!(validate_flow_tree(&flow).is_empty());
    }

    #[test]
    fn repeat_spelling_is_reported_with_the_retry_hint() {
        let flow = tree(
            json!([{"type": "REPEAT", "count": "-1", "repeat-on": "SUCCESS", "repeat-interval": "0", "nodes": []}]),
        );
        let problems = validate_flow_tree(&flow);
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(problems[0].contains("unknown step type \"REPEAT\""));
        assert!(problems[0].contains("\"RETRY\""));
        assert!(problems[1].contains("\"repeat-interval\""));
        assert!(problems[1].contains("\"backoff\""));
    }

    #[test]
    fn wrong_branch_key_and_missing_type_are_reported_with_paths() {
        let flow = tree(json!([
            {"type": "LOOP", "in-array": "/rows", "nodes": [
                {"type": "BRANCH", "label-expressions": "true", "nodes": [{"from": "$loop", "signal": "SUCCESS"}]}
            ]}
        ]));
        let problems = validate_flow_tree(&flow);
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(problems[0].starts_with("flow.nodes[0].nodes[0]: key \"label-expressions\""));
        assert!(problems[0].contains("\"evaluate-labels\""));
        assert!(problems[1].starts_with("flow.nodes[0].nodes[0].nodes[0]: missing \"type\""));
    }

    #[test]
    fn try_catch_as_types_point_to_sequence_form() {
        let flow = tree(json!([{"type": "TRY", "nodes": []}, {"type": "CATCH", "nodes": []}]));
        let problems = validate_flow_tree(&flow);
        assert_eq!(problems.len(), 2);
        assert!(problems.iter().all(|p| p.contains("\"form\": \"TRY\"")));
    }

    #[test]
    fn nodes_must_be_an_array() {
        let flow = tree(json!("[INVOKE]"));
        let problems = validate_flow_tree(&flow);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("must be a JSON array of steps (got string)"));
    }

    #[test]
    fn step_counts_ignore_root_and_single_out_empty_maps() {
        let flow = tree(json!([
            {"type": "INVOKE", "service": "x:y", "nodes": [
                {"type": "MAP", "mode": "INPUT", "nodes": []},
                {"type": "MAP", "mode": "OUTPUT", "nodes": [{"type": "MAPCOPY", "from": "/a;1;0", "to": "/b;1;0"}]}
            ]},
            {"type": "MAP", "mode": "STANDALONE", "nodes": [{"type": "MAPSET", "field": "/c;1;0"}, {"type": "MAPDELETE", "field": "/a;1;0"}]}
        ]));
        let counts = count_steps(&flow);
        assert_eq!(counts.get("INVOKE"), Some(&1));
        assert_eq!(counts.get("MAP"), Some(&2));
        assert_eq!(counts.get("MAP(empty)"), Some(&1));
        assert_eq!(counts.get("MAPCOPY"), Some(&1));
        assert_eq!(counts.get("MAPSET"), Some(&1));
        assert_eq!(counts.get("MAPDELETE"), Some(&1));
        assert_eq!(counts.get("ROOT"), None);
    }

    #[test]
    fn compare_reports_lost_steps_and_tolerates_dropped_empty_maps() {
        let sent = tree(json!([
            {"type": "RETRY", "count": "3", "nodes": [{"type": "MAP", "mode": "STANDALONE", "nodes": [{"type": "MAPSET", "field": "/n;1;0"}]}]},
            {"type": "INVOKE", "service": "x:y", "nodes": [{"type": "MAP", "mode": "INPUT", "nodes": []}]}
        ]));
        let stored_ok = tree(json!([
            {"type": "RETRY", "count": "3", "nodes": [{"type": "MAP", "mode": "STANDALONE", "nodes": [{"type": "MAPSET", "field": "/n;1;0"}]}]},
            {"type": "INVOKE", "service": "x:y", "nodes": []}
        ]));
        let report = compare_flows(&sent, &stored_ok);
        assert_eq!(report["status"], json!("ok"), "{report}");
        assert!(report.get("missing").is_none());

        let stored_lost = tree(json!([{"type": "INVOKE", "service": "x:y", "nodes": []}]));
        let report = compare_flows(&sent, &stored_lost);
        assert_eq!(report["status"], json!("mismatch"));
        assert_eq!(report["missing"]["RETRY"], json!(1));
        assert_eq!(report["missing"]["MAP"], json!(1));
        assert_eq!(report["missing"]["MAPSET"], json!(1));
    }

    #[test]
    fn field_count_is_recursive() {
        let record = json!({"rec_fields": [
            {"field_name": "a"},
            {"field_name": "b", "rec_fields": [{"field_name": "c"}, {"field_name": "d"}]}
        ]});
        assert_eq!(count_fields(&record), 4);
        assert_eq!(count_fields(&json!({})), 0);
    }
}
