#![cfg(test)]
#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/harness.rs"]
mod harness;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

#[test]
fn probe_group() {
    use serde_json::json;
    let h = harness::Harness::over(worked::d8().w);
    let call = |args: serde_json::Value| json!({"model":"openai/gpt-6-luna","choices":[{"finish_reason":"tool_calls","message":{"role":"assistant","content":null,"tool_calls":[{"id":"c0","type":"function","function":{"name":"analytics","arguments":args.to_string()}}]}}],"usage":{"prompt_tokens":100,"completion_tokens":10,"prompt_tokens_details":{"cached_tokens":0},"completion_tokens_details":{"reasoning_tokens":0}}});
    for (view, buyer) in [("C", Some("a26")), ("A", None), ("B", None)] {
        let input = json!({"view":view,"from":"2026-10-01","to":"2026-11-08","group_by":"month","item":null,"item_group":null,"seller":null,"buyer":buyer,"city":null,"adda":null,"garden":null,"trip":null,"goods":null,"partner":null,"side":null,"compare":null,"sort":null,"rows":null,"grade":null});
        let (_, requests) = h.ask("mani ne is saal mahine ke hisab se kitna maal liya", None, vec![call(input)]);
        let last = requests.last().unwrap();
        let tool = last["messages"].as_array().unwrap().iter().filter(|m| m["role"]=="tool").last().map(|m| m["content"].to_string().chars().take(1500).collect::<String>());
        println!("GP {view} {tool:?}");
    }
}
