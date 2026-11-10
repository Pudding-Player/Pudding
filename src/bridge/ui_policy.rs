//! UI 是最后手段；模型说明不能替代本轮非 UI 路径不可用的真实观测。
use crate::{
    Error, Result,
    model::{Message, Role},
    runtime::types::Event,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub(crate) fn require(
    id: &str,
    args: &Value,
    history: &[Message],
    events: &[Event],
    run_id: &str,
) -> Result<()> {
    if !id.starts_with("bgi.ui.") || matches!(id, "bgi.ui.cancel" | "bgi.ui.operation") {
        return Ok(());
    }
    let user = history
        .iter()
        .rev()
        .find(|m| m.role == Role::User)
        .map(|m| m.content.as_str())
        .unwrap_or("");
    let denied =
        regex::Regex::new(r"(?i)(不要|禁止|不得|不许|别|do not|don't|never).{0,18}(界面|UI|点击|按钮|屏幕|interface|click|button)").unwrap();
    if denied.is_match(user) {
        return Err(Error::NotSubmitted(
            "用户禁止界面操作；本次未使用 UI。继续查本体业务接口，不能绕行界面。".into(),
        ));
    }
    let requested = regex::Regex::new(r"(?i)(^|[。！？；\n])\s*(请帮我|请|帮我|麻烦)?\s*((通过|使用|用|操作).{0,6}(界面|UI|图形界面)|(点击|点一下|按一下).{0,30}(按钮|菜单|选项卡)|在.{0,30}(窗口|页面|界面).{0,10}(点击|选择|填写|输入))").unwrap();
    if requested.is_match(user) {
        return Ok(());
    }
    let evidence = args["uiFallback"]["evidenceCallIds"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let calls = events
        .iter()
        .filter(|e| e.run_id == run_id && e.kind == "tool.started")
        .filter_map(|e| {
            Some((
                e.data["id"].as_str()?,
                (&e.data["name"], &e.data["arguments"]),
            ))
        })
        .collect::<HashMap<_, _>>();
    let mut ordinary_missing = false;
    let mut native_missing = false;
    for event in events
        .iter()
        .filter(|e| e.run_id == run_id && e.kind == "tool.completed")
    {
        let Some(call_id) = event.data["callId"]
            .as_str()
            .filter(|id| evidence.contains(id))
        else {
            continue;
        };
        let Some((tool, call)) = calls.get(call_id) else {
            continue;
        };
        let output = &event.data["result"];
        if *tool == "bgi.api.search"
            && call["group"] != "ui"
            && output["ok"] == true
            && output["value"]["total"] == 0
            && call["query"].as_str().is_some_and(|q| !q.trim().is_empty())
        {
            ordinary_missing = true;
        }
        if *tool == "bgi.api.read"
            && call["methodId"] == "bgi.clr.describe"
            && output["ok"] == true
            && output["value"]["canPrepare"] == false
        {
            native_missing = true;
        }
    }
    if ordinary_missing
        && native_missing
        && args["uiFallback"]["reason"]
            .as_str()
            .is_some_and(|s| s.trim().chars().count() >= 8)
    {
        return Ok(());
    }
    Err(Error::NotSubmitted("UI 操作默认禁止、优先级最低。用户未明确要求界面操作时，必须提供本轮常规接口未命中和本体 CLR 成员确实不可调用的真实 evidenceCallIds 及原因；先完成非 UI 的成员发现与调用，不得因单次查询为空而转向界面。".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn user(content: &str) -> Vec<Message> {
        vec![Message {
            role: Role::User,
            content: content.into(),
            tool_call_id: None,
            tool_calls: vec![],
            reasoning: None,
        }]
    }
    #[test]
    fn ui_requires_real_user_intent_not_mentions_or_model_claims() {
        let claim =
            json!({"uiFallback":{"reason":"我已经检查所有接口","evidenceCallIds":["a","b"]}});
        assert!(require("bgi.ui.read", &claim, &user("兑换这个码"), &[], "r").is_err());
        assert!(
            require(
                "bgi.ui.read",
                &claim,
                &user("解释文档里的“使用UI”是什么意思"),
                &[],
                "r"
            )
            .is_err()
        );
        assert!(
            require(
                "bgi.ui.invoke",
                &json!({}),
                &user("请点击兑换按钮"),
                &[],
                "r"
            )
            .is_ok()
        );
        assert!(require("bgi.ui.invoke", &claim, &user("禁止任何界面操作"), &[], "r").is_err());
        assert!(
            require(
                "bgi.clr.execute",
                &json!({}),
                &user("禁止任何界面操作"),
                &[],
                "r"
            )
            .is_ok()
        );
    }
    #[test]
    fn last_resort_needs_both_current_non_ui_paths_exhausted() {
        let mut events = vec![
            Event {
                sequence: 1,
                conversation_id: "c".into(),
                run_id: "r".into(),
                kind: "tool.started".into(),
                data: json!({"id":"a","name":"bgi.api.search","arguments":{"query":"目标业务"}}),
            },
            Event {
                sequence: 2,
                conversation_id: "c".into(),
                run_id: "r".into(),
                kind: "tool.completed".into(),
                data: json!({"callId":"a","result":{"ok":true,"value":{"total":0}}}),
            },
            Event {
                sequence: 3,
                conversation_id: "c".into(),
                run_id: "r".into(),
                kind: "tool.started".into(),
                data: json!({"id":"b","name":"bgi.api.read","arguments":{"methodId":"bgi.clr.describe"}}),
            },
            Event {
                sequence: 4,
                conversation_id: "c".into(),
                run_id: "r".into(),
                kind: "tool.completed".into(),
                data: json!({"callId":"b","result":{"ok":true,"value":{"canPrepare":true}}}),
            },
        ];
        let request = json!({"uiFallback":{"reason":"常规接口和本体成员均不可用","evidenceCallIds":["a","b"]}});
        assert!(require("bgi.ui.read", &request, &user("处理目标业务"), &events, "r").is_err());
        events[3].data["result"]["value"]["canPrepare"] = json!(false);
        assert!(require("bgi.ui.read", &request, &user("处理目标业务"), &events, "r").is_ok());
        assert!(
            require(
                "bgi.ui.read",
                &request,
                &user("禁止任何界面操作"),
                &events,
                "r"
            )
            .is_err()
        );
        assert!(
            require(
                "bgi.ui.read",
                &request,
                &user("处理目标业务"),
                &events,
                "another-run"
            )
            .is_err()
        );
    }
}
