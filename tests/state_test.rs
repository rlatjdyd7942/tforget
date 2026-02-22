use tempfile::TempDir;
use tforge::state::{PipelineState, StepState};

#[test]
fn test_save_and_load_state() {
    let tmp = TempDir::new().unwrap();
    let state_file = tmp.path().join(".tforge-state.json");

    let mut state = PipelineState::new();
    state.mark_completed("flutter-app", 0);
    state.mark_completed("flutter-app", 1);
    state.mark_failed("gcp-project", 0, "quota exceeded");
    state.save(&state_file).unwrap();

    let loaded = PipelineState::load(&state_file).unwrap();
    assert_eq!(loaded.get("flutter-app", 0), StepState::Completed);
    assert_eq!(loaded.get("flutter-app", 1), StepState::Completed);
    assert!(matches!(loaded.get("gcp-project", 0), StepState::Failed(_)));
    assert_eq!(loaded.get("gcp-project", 1), StepState::Pending);
}

#[test]
fn test_load_nonexistent_returns_empty() {
    let tmp = TempDir::new().unwrap();
    let state_file = tmp.path().join(".tforge-state.json");
    let state = PipelineState::load(&state_file).unwrap();
    assert_eq!(state.get("anything", 0), StepState::Pending);
}
