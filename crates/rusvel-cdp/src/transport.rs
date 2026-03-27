//! HTTP discovery (`/json/list`) and CDP WebSocket command helpers.

use std::borrow::Cow;

use futures::{SinkExt, StreamExt};
use rusvel_core::{Result, RusvelError};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

/// Normalize user input into an HTTP origin usable for `GET /json/list`.
pub fn http_base(endpoint: &str) -> Cow<'_, str> {
    let e = endpoint.trim();
    if let Some(rest) = e.strip_prefix("ws://") {
        Cow::Owned(format!("http://{rest}"))
    } else if let Some(rest) = e.strip_prefix("wss://") {
        Cow::Owned(format!("https://{rest}"))
    } else if e.starts_with("http://") || e.starts_with("https://") {
        Cow::Borrowed(e)
    } else {
        Cow::Owned(format!("http://{e}"))
    }
}

#[derive(Debug, Deserialize)]
pub struct ChromeTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub web_socket_debugger_url: Option<String>,
}

pub async fn fetch_targets(http_base: &str) -> Result<Vec<ChromeTarget>> {
    let url = format!("{}/json/list", http_base.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| RusvelError::Internal(e.to_string()))?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| RusvelError::Internal(e.to_string()))?;
    let text = resp
        .text()
        .await
        .map_err(|e| RusvelError::Internal(e.to_string()))?;
    let list: Vec<ChromeTarget> =
        serde_json::from_str(&text).map_err(|e| RusvelError::Serialization(e.to_string()))?;
    Ok(list)
}

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

pub async fn cdp_evaluate(ws_url: &str, expression: &str) -> Result<serde_json::Value> {
    let (mut ws, _) = connect_async(ws_url)
        .await
        .map_err(|e| RusvelError::Internal(format!("cdp connect: {e}")))?;
    let mut next_id: u64 = 1;
    let id_run = send_cmd(&mut ws, &mut next_id, "Runtime.enable", json!({})).await?;
    let _ = read_until_id(&mut ws, id_run).await?;
    let id_eval = send_cmd(
        &mut ws,
        &mut next_id,
        "Runtime.evaluate",
        json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true,
        }),
    )
    .await?;
    let result = read_until_id(&mut ws, id_eval).await?;
    let _ = ws.close(None).await;
    parse_evaluate_value(&result)
}

pub async fn cdp_navigate(ws_url: &str, page_url: &str) -> Result<()> {
    let (mut ws, _) = connect_async(ws_url)
        .await
        .map_err(|e| RusvelError::Internal(format!("cdp connect: {e}")))?;
    let mut next_id: u64 = 1;
    let id_page = send_cmd(&mut ws, &mut next_id, "Page.enable", json!({})).await?;
    let _ = read_until_id(&mut ws, id_page).await?;
    let id_nav = send_cmd(
        &mut ws,
        &mut next_id,
        "Page.navigate",
        json!({ "url": page_url }),
    )
    .await?;
    let _ = read_until_id(&mut ws, id_nav).await?;
    let _ = ws.close(None).await;
    Ok(())
}

async fn send_cmd(
    ws: &mut WsStream,
    next_id: &mut u64,
    method: &str,
    params: serde_json::Value,
) -> Result<u64> {
    let id = *next_id;
    *next_id += 1;
    let msg = json!({ "id": id, "method": method, "params": params });
    let text =
        serde_json::to_string(&msg).map_err(|e| RusvelError::Serialization(e.to_string()))?;
    ws.send(Message::Text(text.into()))
        .await
        .map_err(|e| RusvelError::Internal(e.to_string()))?;
    Ok(id)
}

async fn read_until_id(ws: &mut WsStream, want: u64) -> Result<serde_json::Value> {
    loop {
        let frame = match ws.next().await {
            Some(Ok(m)) => m,
            Some(Err(e)) => return Err(RusvelError::Internal(e.to_string())),
            None => return Err(RusvelError::Internal("cdp websocket closed".into())),
        };
        let Message::Text(text) = frame else {
            continue;
        };
        let v: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| RusvelError::Serialization(e.to_string()))?;
        if v.get("id").and_then(serde_json::Value::as_u64) == Some(want) {
            if let Some(err) = v.get("error") {
                return Err(RusvelError::Internal(err.to_string()));
            }
            return Ok(v.get("result").cloned().unwrap_or(serde_json::Value::Null));
        }
    }
}

fn parse_evaluate_value(result: &serde_json::Value) -> Result<serde_json::Value> {
    if let Some(ex) = result.get("exceptionDetails") {
        return Err(RusvelError::Internal(ex.to_string()));
    }
    result
        .get("result")
        .and_then(|r| r.get("value"))
        .cloned()
        .ok_or_else(|| RusvelError::Internal("Runtime.evaluate missing value".into()))
}
