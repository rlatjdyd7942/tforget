use tforge::toolcheck::{check_tool, ToolStatus};

#[test]
fn test_check_existing_tool() {
    let status = check_tool("echo");
    assert!(matches!(status, ToolStatus::Found(_)));
}

#[test]
fn test_check_nonexistent_tool() {
    let status = check_tool("nonexistent_tool_xyz_12345");
    assert!(matches!(status, ToolStatus::NotFound));
}
