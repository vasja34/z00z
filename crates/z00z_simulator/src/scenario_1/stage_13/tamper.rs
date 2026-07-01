use super::report::{RedactedError, Stage13TamperCase};

pub(crate) fn case(
    scenario_id: u32,
    stage: u32,
    example_id: impl Into<String>,
    backend_mode: impl Into<String>,
    api_surface: impl Into<String>,
    root_generation: u8,
    case_id: impl Into<String>,
    error: RedactedError,
) -> Stage13TamperCase {
    Stage13TamperCase {
        schema_version: super::report::STAGE13_SCHEMA_VERSION,
        scenario_id,
        stage,
        example_id: example_id.into(),
        backend_mode: backend_mode.into(),
        api_surface: api_surface.into(),
        proof_surface: "stage13_example".to_string(),
        verifier_status: "rejected".to_string(),
        root_generation,
        path_count: None,
        path_shape: None,
        case_id: case_id.into(),
        typed_error: error,
    }
}
