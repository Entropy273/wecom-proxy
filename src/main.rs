use axum::{
    extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

#[derive(Debug)]
struct WecomAppConfig {
    auth_key: String,
    wecom_cid: String,
    wecom_secret: String,
    wecom_aid: String,
    wecom_touid: String,
}

impl WecomAppConfig {
    fn from_env() -> Result<Self, String> {
        Ok(Self {
            auth_key: env::var("AUTH_KEY").map_err(|_| "AUTH_KEY not found".to_string())?,
            wecom_cid: env::var("WECOM_CID").map_err(|_| "WECOM_CID not found".to_string())?,
            wecom_secret: env::var("WECOM_SECRET").map_err(|_| "WECOM_SECRET not found".to_string())?,
            wecom_aid: env::var("WECOM_AID").map_err(|_| "WECOM_AID not found".to_string())?,
            wecom_touid: env::var("WECOM_TOUID").unwrap_or_else(|_| "@all".to_string()),
        })
    }
}

#[derive(Debug, Clone)]
struct AppState {
    client: Client,
    config: Arc<WecomAppConfig>,
}

/// Request to send message to Wecom. From user to wecom-proxy
#[derive(Deserialize, Debug)]
struct WecomProxyAppMsgReq {
    auth_key: String,
    msg: String,
}

/// Request to send message to Wecom. From wecom-porxy to Wecom app backend
#[derive(Serialize, Deserialize)]
struct WecomAppMsgReq {
    touser: String,
    agentid: String,
    msgtype: String,
    #[serde(rename = "duplicate_check_interval")]
    duplicate_check_interval: u64,
    text: WecomAppMsgContent,
}

/// Content of the message, part of WecomAppMsgReq
#[derive(Serialize, Deserialize)]
struct WecomAppMsgContent {
    content: String,
}

async fn get_wecom_access_token(client: &Client, config: &WecomAppConfig) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
        config.wecom_cid, config.wecom_secret
    );

    let resp = client.get(&url).send().await?.json::<serde_json::Value>().await?;
    Ok(resp["access_token"].as_str().unwrap_or("").to_string())
}

fn build_wecom_app_msg_req(msg: &str, config: &WecomAppConfig) -> WecomAppMsgReq {
    WecomAppMsgReq {
        touser: config.wecom_touid.clone(),
        agentid: config.wecom_aid.clone(),
        msgtype: "text".to_string(),
        duplicate_check_interval: 600,
        text: WecomAppMsgContent {
            content: msg.to_string(),
        },
    }
}

async fn send_wecom_app_msg(client: &Client, token: &str, data: &WecomAppMsgReq) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
        token
    );

    let resp = client
        .post(&url)
        .json(data)
        .send()
        .await?
        .text()
        .await?;

    Ok(resp)
}

async fn handle_wecom_request(params: WecomProxyAppMsgReq, state: Arc<AppState>) -> impl IntoResponse {
    if params.auth_key != state.config.auth_key {
        return (StatusCode::UNAUTHORIZED, "Wrong auth_key").into_response();
    }

    let token = match get_wecom_access_token(&state.client, &state.config).await {
        Ok(token) => token,
        Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    };

    let req_data = build_wecom_app_msg_req(&params.msg, &state.config);
    match send_wecom_app_msg(&state.client, &token, &req_data).await {
        Ok(response) => response.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn wecom_get_handler(
    State(state): State<Arc<AppState>>,
    Query(query_params): Query<WecomProxyAppMsgReq>,
) -> impl IntoResponse {
    handle_wecom_request(query_params, state).await
}

async fn wecom_post_handler(
    State(state): State<Arc<AppState>>,
    Json(form_params): Json<WecomProxyAppMsgReq>,
) -> impl IntoResponse {
    handle_wecom_request(form_params, state).await
}

#[tokio::main]
async fn main() {
    let config = WecomAppConfig::from_env().expect("Failed to load configuration");
    let client = Client::new();
    let state= Arc::new(AppState {
        client,
        config: Arc::new(config),
    });

    let app = Router::new()
        .route("/wecom", get(wecom_get_handler))
        .route("/wecom", post(wecom_post_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}