use tforge::condition::evaluate_condition;
use std::collections::HashMap;

#[test]
fn test_contains_true() {
    let mut vars = HashMap::new();
    vars.insert("services".into(), "crashlytics,auth,analytics".into());
    assert!(evaluate_condition("services contains 'crashlytics'", &vars).unwrap());
}

#[test]
fn test_contains_false() {
    let mut vars = HashMap::new();
    vars.insert("services".into(), "auth,analytics".into());
    assert!(!evaluate_condition("services contains 'crashlytics'", &vars).unwrap());
}

#[test]
fn test_equals_true() {
    let mut vars = HashMap::new();
    vars.insert("db_engine".into(), "mysql-9.0".into());
    assert!(evaluate_condition("db_engine == 'mysql-9.0'", &vars).unwrap());
}

#[test]
fn test_equals_false() {
    let mut vars = HashMap::new();
    vars.insert("db_engine".into(), "postgres-16".into());
    assert!(!evaluate_condition("db_engine == 'mysql-9.0'", &vars).unwrap());
}

#[test]
fn test_missing_variable() {
    let vars = HashMap::new();
    let result = evaluate_condition("missing contains 'x'", &vars);
    assert!(result.is_err());
}
