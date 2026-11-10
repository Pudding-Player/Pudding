//! 用真实模型验证漏登记后定位并调用本体方法；UI 执行端禁止，不能以界面绕行算通过。
use super::*;
use std::{
    io::BufRead,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
struct ClrHost {
    child: Child,
    input: ChildStdin,
    output: std::io::BufReader<ChildStdout>,
    ready: Value,
}
impl ClrHost {
    fn start(root: &std::path::Path) -> Self {
        let mut child = Command::new("dotnet")
            .arg(std::env::var("PUDDING_CLR_HOST").expect("fixture DLL"))
            .arg(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let mut output = std::io::BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let ready = serde_json::from_str(line.trim_start_matches('\u{feff}')).unwrap();
        Self {
            child,
            input,
            output,
            ready,
        }
    }
    fn request(&mut self, method: &str, path: &str, body: &Value) -> Value {
        writeln!(
            self.input,
            "{}",
            json!({"method":method,"path":path,"body":body})
        )
        .unwrap();
        self.input.flush().unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
}
impl Drop for ClrHost {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
struct ClrProxy {
    url: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl ClrProxy {
    fn start(host: Arc<Mutex<ClrHost>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let done = stop.clone();
        let thread = std::thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                let Ok((mut stream, _)) = listener.accept() else {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut b = [0; 1];
                while !bytes.ends_with(b"\r\n\r\n") {
                    if stream.read(&mut b).unwrap_or(0) == 0 {
                        break;
                    }
                    bytes.push(b[0]);
                }
                let head = String::from_utf8_lossy(&bytes);
                let mut line = head.lines().next().unwrap_or("").split_whitespace();
                let method = line.next().unwrap_or("GET");
                let path = line.next().unwrap_or("/");
                let reply = host.lock().unwrap().request(method, path, &Value::Null);
                let body = reply["value"].to_string();
                let header = format!(
                    "HTTP/1.1 {} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    reply["status"].as_u64().unwrap(),
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(body.as_bytes());
            }
        });
        Self {
            url,
            stop,
            thread: Some(thread),
        }
    }
}
impl Drop for ClrProxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[tokio::test]
#[ignore = "显式真实模型与隔离 CLR 宿主；原方法仅记录，不运行真实 BGI/游戏"]
async fn live_bgi_clr_fallback_without_ui() {
    let mut config: Value = serde_json::from_str(
        &std::fs::read_to_string(std::env::var("PUDDING_MODEL_CONFIG").unwrap()).unwrap(),
    )
    .unwrap();
    crate::config::expand_env(&mut config).unwrap();
    let name = std::env::var("PUDDING_TEST_MODEL")
        .unwrap_or_else(|_| config["activeModel"].as_str().unwrap().into());
    let mut model: ModelConfig = serde_json::from_value(
        config["models"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == name || m["model"] == name)
            .unwrap()
            .clone(),
    )
    .unwrap();
    model.id = "fixture".into();
    model.options.max_output_tokens = Some(4096);
    model.options.timeout_ms = model.options.timeout_ms.min(90000);
    let root = std::path::PathBuf::from(std::env::var("PUDDING_TEST_ROOT").unwrap());
    let host = Arc::new(Mutex::new(ClrHost::start(&root)));
    let ready = host.lock().unwrap().ready.clone();
    let proxy = ClrProxy::start(host.clone());
    let unused = ModelFixture::new(|_| panic!("use configured real model"));
    let supervisor = supervisor(&unused);
    let bridge: crate::config::BridgeConfig =
        serde_json::from_value(json!({"enabled":true,"baseUrl":proxy.url,"timeoutMs":15000}))
            .unwrap();
    {
        let mut c = supervisor.config.write().unwrap();
        c.models[0] = model.clone();
        c.bridge = bridge.clone();
        c.plugins.enabled.push("bgi".into());
        c.runtime.memory_enabled = false;
        c.runtime.max_decisions = 28;
        c.runtime.max_tools = 70;
    }
    let mut tools = ToolRegistry::default();
    crate::bridge::register_tools(
        &mut tools,
        Arc::new(crate::bridge::BgiClient::for_tests(bridge)),
    )
    .unwrap();
    let mut internal = ToolRegistry::default();
    let endpoint = host.clone();
    internal.register(crate::extension::FunctionTool::new("bgi.host.request","隔离内部传输",json!({"type":"object","properties":{"method":{"type":"string"},"path":{"type":"string"},"body":{}},"required":["method","path"]}),"fixture:transport",move |a|{
        let r=endpoint.lock().unwrap().request(a["method"].as_str().unwrap(),a["path"].as_str().unwrap(),&a["body"]);
        Ok(if r["status"].as_u64().unwrap()<400{json!({"ok":true,"value":r["value"]})}else{json!({"ok":false,"error":{"kind":"tool","message":r["value"].to_string()}})})
    })).unwrap();
    let client = tokio::task::spawn_blocking(move || {
        let manifest = serde_json::from_value(
            json!({"id":"bridge","transport":"inProcess","server":"bgi","timeoutMs":15000}),
        )
        .unwrap();
        crate::extension::mcp::McpClient::in_process(
            &manifest,
            crate::extension::mcp_server::RegistryServer::new(
                internal,
                "isolated-clr",
                "测试内部传输",
            ),
        )
    })
    .await
    .unwrap()
    .unwrap();
    tools.register_mcp("bgi", "bridge", client);
    let mut skills = SkillRegistry::default();
    skills
        .load(&[(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("bridges/bgi/plugins/bgi/skills"),
            crate::extension::skills::SkillSource::Plugin("bgi".into()),
        )])
        .unwrap();
    for source in crate::extension::skills::EMBEDDED_SKILLS {
        skills.load_embedded(source).unwrap();
    }
    {
        let mut e = supervisor.extensions.write().unwrap();
        e.tools = Arc::new(tools);
        e.skills = Arc::new(skills);
    }
    let prompt = "在当前隔离宿主的 BetterGI 中使用兑换码 GAPTEST123456。禁止任何界面操作。";
    let mut run = supervisor
        .journal
        .create(prompt, "clr-gap", "native-method", 360, None)
        .unwrap();
    let start = std::time::Instant::now();
    let result = supervisor
        .session(&mut run, &CancellationToken::new())
        .await;
    let events = supervisor.journal.events(&run.conversation_id, 0).unwrap();
    let calls = events
        .iter()
        .filter(|e| e.kind == "tool.started" || e.kind == "tool.completed")
        .map(|e| json!({"kind":e.kind,"data":e.data}))
        .collect::<Vec<_>>();
    let state = host
        .lock()
        .unwrap()
        .request("GET", "/fixture/state", &Value::Null)["value"]
        .clone();
    let ui_attempts = calls
        .iter()
        .filter(|e| {
            e["kind"] == "tool.started"
                && e["data"]["arguments"]["methodId"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("bgi.ui."))
        })
        .count();
    let invocation_verified = state["originalMethodCalls"] == 1
        && state["runnerCalls"] == 1
        && state["tokenInjected"] == true
        && state["lockHeld"] == true
        && state["codes"] == json!(["GAPTEST123456"])
        && state["uiAttempted"] == false
        && ui_attempts == 0;
    let conversation_completed =
        result.is_ok() && run.result.as_deref().is_some_and(|s| !s.trim().is_empty());
    let passed = invocation_verified && conversation_completed;
    let path = std::env::var("PUDDING_BENCH_REPORT").unwrap();
    let mut prior = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .map(|mut r| {
            let old = r
                .as_object_mut()
                .unwrap()
                .remove("priorAttempts")
                .unwrap_or(json!([]));
            let mut items = old.as_array().cloned().unwrap_or_default();
            items.push(r);
            items
        })
        .unwrap_or_default();
    let report = json!({"passed":passed,"invocationVerified":invocation_verified,"conversationCompleted":conversation_completed,"model":model.model,"prompt":prompt,"state":run.state,"answer":run.result,"error":result.err().map(|e|e.user_message()),"elapsedSec":start.elapsed().as_secs_f64(),"decisions":run.decisions,"tools":run.tool_calls,"uiToolAttempts":ui_attempts,"fixture":state,"host":ready,"calls":calls,"usage":supervisor.journal.model_usage(&run.id).unwrap(),"priorAttempts":std::mem::take(&mut prior),"scope":"production CLR fallback and Supervisor; isolated host with original type/constructor/method signatures, game side effects replaced by recorder; not real BGI/game"});
    std::fs::write(
        std::env::var("PUDDING_BENCH_REPORT").unwrap(),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    supervisor.shutdown();
    drop(supervisor);
    drop(proxy);
    drop(host);
    println!(
        "CLR gap: passed={passed}, UI attempts={ui_attempts}, decisions={}, tools={}",
        run.decisions, run.tool_calls
    );
    assert!(
        passed,
        "Agent did not invoke original method without UI; inspect report"
    );
}
