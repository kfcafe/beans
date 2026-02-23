use serde::Serialize;

/// JSON-line events emitted by `bn run --json-stream` for programmatic consumers.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    RunStart {
        parent_id: String,
        total_beans: usize,
        total_rounds: usize,
        beans: Vec<BeanInfo>,
    },
    RoundStart {
        round: usize,
        total_rounds: usize,
        bean_count: usize,
    },
    BeanStart {
        id: String,
        title: String,
        round: usize,
    },
    BeanThinking {
        id: String,
        text: String,
    },
    BeanTool {
        id: String,
        tool_name: String,
        tool_count: usize,
        file_path: Option<String>,
    },
    BeanTokens {
        id: String,
        input_tokens: u64,
        output_tokens: u64,
        cache_read: u64,
        cache_write: u64,
        cost: f64,
    },
    BeanDone {
        id: String,
        success: bool,
        duration_secs: u64,
        error: Option<String>,
        total_tokens: Option<u64>,
        total_cost: Option<f64>,
    },
    RoundEnd {
        round: usize,
        success_count: usize,
        failed_count: usize,
    },
    RunEnd {
        total_success: usize,
        total_failed: usize,
        duration_secs: u64,
    },
    DryRun {
        parent_id: String,
        rounds: Vec<RoundPlan>,
    },
    Error {
        message: String,
    },
}

/// Metadata about a single bean within a run.
#[derive(Debug, Clone, Serialize)]
pub struct BeanInfo {
    pub id: String,
    pub title: String,
    pub round: usize,
}

/// Describes which beans will execute in a given round (used by `DryRun`).
#[derive(Debug, Clone, Serialize)]
pub struct RoundPlan {
    pub round: usize,
    pub beans: Vec<BeanInfo>,
}

/// Write a single JSON line to stdout for the given event.
pub fn emit(event: &StreamEvent) {
    if let Ok(json) = serde_json::to_string(event) {
        println!("{json}");
    }
}

/// Convenience wrapper to emit an `Error` event.
pub fn emit_error(message: &str) {
    emit(&StreamEvent::Error {
        message: message.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_event_serializes_with_type_tag() {
        let event = StreamEvent::RunStart {
            parent_id: "42".into(),
            total_beans: 3,
            total_rounds: 2,
            beans: vec![BeanInfo {
                id: "42.1".into(),
                title: "first".into(),
                round: 1,
            }],
        };
        let json: serde_json::Value = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "run_start");
        assert_eq!(json["parent_id"], "42");
        assert_eq!(json["total_beans"], 3);
        assert_eq!(json["beans"][0]["id"], "42.1");
    }

    #[test]
    fn stream_bean_done_serializes_optional_fields() {
        let event = StreamEvent::BeanDone {
            id: "1".into(),
            success: true,
            duration_secs: 10,
            error: None,
            total_tokens: Some(500),
            total_cost: Some(0.01),
        };
        let json: serde_json::Value = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "bean_done");
        assert!(json["error"].is_null());
        assert_eq!(json["total_tokens"], 500);
    }

    #[test]
    fn stream_error_event() {
        let event = StreamEvent::Error {
            message: "something broke".into(),
        };
        let json: serde_json::Value = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "error");
        assert_eq!(json["message"], "something broke");
    }

    #[test]
    fn stream_dry_run_with_round_plans() {
        let event = StreamEvent::DryRun {
            parent_id: "10".into(),
            rounds: vec![
                RoundPlan {
                    round: 1,
                    beans: vec![BeanInfo {
                        id: "10.1".into(),
                        title: "a".into(),
                        round: 1,
                    }],
                },
                RoundPlan {
                    round: 2,
                    beans: vec![BeanInfo {
                        id: "10.2".into(),
                        title: "b".into(),
                        round: 2,
                    }],
                },
            ],
        };
        let json: serde_json::Value = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "dry_run");
        assert_eq!(json["rounds"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn stream_emit_writes_json_line() {
        // Just ensure emit doesn't panic — stdout capture is not trivial in unit tests
        let event = StreamEvent::RoundEnd {
            round: 1,
            success_count: 2,
            failed_count: 0,
        };
        emit(&event);
    }

    #[test]
    fn stream_emit_error_convenience() {
        emit_error("test error");
    }
}
