// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn ops_list_payload(profile_count: usize, mut action_ids: Vec<String>) -> Value {
    action_ids.sort();
    let rows = vec![
        json!({"kind":"capability","name":"inventory","subprocess":false,"write":false}),
        json!({"kind":"capability","name":"validate","subprocess":false,"write":false}),
        json!({"kind":"capability","name":"render","subprocess":true,"write":"flag_gated"}),
        json!({"kind":"capability","name":"install","subprocess":true,"write":"flag_gated"}),
        json!({"kind":"capability","name":"status","subprocess":"target_gated","write":false}),
        json!({"kind":"capability","name":"cleanup","subprocess":"profile_dependent","write":false}),
        json!({"kind":"profiles","count": profile_count}),
        json!({"kind":"actions","count": action_ids.len(), "action_ids": action_ids}),
    ];

    json!({
        "schema_version": 1,
        "text": "ops list capabilities and actions",
        "rows": rows,
        "summary": {"total": 8, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::ops_list_payload;

    #[test]
    fn ops_list_payload_sorts_actions_and_reports_counts() {
        let payload = ops_list_payload(
            3,
            vec![
                "stack-down".to_string(),
                "inventory".to_string(),
                "stack-up".to_string(),
            ],
        );

        assert_eq!(payload["summary"]["total"], 8);
        assert_eq!(payload["rows"][6]["count"], 3);
        assert_eq!(
            payload["rows"][7]["action_ids"],
            serde_json::json!(["inventory", "stack-down", "stack-up"])
        );
    }
}
