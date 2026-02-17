use bijux_atlas_core::{sha256, ErrorCode, ExitCode, MachineError};

#[test]
fn display_and_debug_contracts_are_stable() {
    let hash = sha256(b"atlas");
    assert_eq!(
        format!("{hash:?}"),
        "Hash256(7c82602500857aa6ed0cf38c4c3e4ec645bdcaa82c00b9155eb08be100c778a9)"
    );
    assert_eq!(
        format!("{hash}"),
        "7c82602500857aa6ed0cf38c4c3e4ec645bdcaa82c00b9155eb08be100c778a9"
    );

    assert_eq!(format!("{}", ErrorCode::Internal), "Internal");
    assert_eq!(format!("{:?}", ExitCode::Internal), "Internal");

    let err = MachineError::new("usage_error", "invalid");
    assert_eq!(format!("{err}"), "usage_error: invalid");
    assert_eq!(
        format!("{err:?}"),
        "MachineError { code: \"usage_error\", message: \"invalid\", details: {} }"
    );
}
