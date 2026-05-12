use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use super::cookie;
use super::models::{BilibiliCookies, QrcodeData};
use super::wbi::WbiSigner;

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 1000;

const BILIBILI_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

#[derive(Debug)]
pub enum BiliApiError {
    Network(String),
    RateLimit(String),
    CookieExpired(String),
    ApiError { code: i64, message: String },
    Parse(String),
}

impl std::fmt::Display for BiliApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "网络错误: {}", e),
            Self::RateLimit(e) => write!(f, "请求频率过高，请稍后重试: {}", e),
            Self::CookieExpired(e) => write!(f, "登录已过期，请重新扫码登录: {}", e),
            Self::ApiError { code, message } => write!(f, "B 站 API 错误 ({}): {}", code, message),
            Self::Parse(e) => write!(f, "数据解析错误: {}", e),
        }
    }
}

impl From<BiliApiError> for String {
    fn from(e: BiliApiError) -> String {
        e.to_string()
    }
}

fn classify_api_error(code: i64, message: &str) -> BiliApiError {
    match code {
        -101 | -111 => BiliApiError::CookieExpired(message.to_string()),
        -412 | -401 => BiliApiError::RateLimit(message.to_string()),
        _ => BiliApiError::ApiError {
            code,
            message: message.to_string(),
        },
    }
}

async fn get_json_with_retry(
    client: &Client,
    url: &str,
) -> Result<serde_json::Value, BiliApiError> {
    let mut last_err = BiliApiError::Network("未开始请求".to_string());

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let delay = BASE_RETRY_DELAY_MS * 2u64.pow(attempt - 1);
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        match client.get(url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    || status.as_u16() == 412
                {
                    last_err = BiliApiError::RateLimit(format!("HTTP {}", status));
                    continue;
                }
                if status.is_server_error() {
                    last_err = BiliApiError::Network(format!("HTTP {}", status));
                    continue;
                }
                return resp.json().await.map_err(|e| BiliApiError::Parse(e.to_string()));
            }
            Err(e) => {
                if e.is_timeout() || e.is_connect() {
                    last_err = BiliApiError::Network(e.to_string());
                    continue;
                }
                return Err(BiliApiError::Network(e.to_string()));
            }
        }
    }

    Err(last_err)
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(BILIBILI_UA));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.bilibili.com/"),
    );
    headers
}

pub fn build_client() -> Result<Client, String> {
    Client::builder()
        .default_headers(default_headers())
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

pub fn build_authed_client(cookies: &BilibiliCookies) -> Result<Client, String> {
    let cookie_str = format!(
        "SESSDATA={}; bili_jct={}; DedeUserID={}; DedeUserID__ckMd5={}",
        cookies.sessdata, cookies.bili_jct, cookies.dede_user_id, cookies.dede_user_id_ckmd5
    );

    let mut headers = default_headers();
    headers.insert(
        reqwest::header::COOKIE,
        HeaderValue::from_str(&cookie_str)
            .map_err(|e| format!("设置 Cookie header 失败: {}", e))?,
    );

    Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("创建认证客户端失败: {}", e))
}

// --- QR 码登录 ---

pub async fn generate_qrcode(client: &Client) -> Result<QrcodeData, String> {
    let resp: serde_json::Value = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .send()
        .await
        .map_err(|e| format!("请求 QR 码失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析 QR 码响应失败: {}", e))?;

    if resp["code"].as_i64() != Some(0) {
        return Err(format!("生成 QR 码失败: {}", resp["message"]));
    }

    Ok(QrcodeData {
        url: resp["data"]["url"]
            .as_str()
            .ok_or("缺少 QR 码 URL")?
            .to_string(),
        qrcode_key: resp["data"]["qrcode_key"]
            .as_str()
            .ok_or("缺少 qrcode_key")?
            .to_string(),
    })
}

#[derive(Debug)]
pub struct PollResult {
    pub code: i64,
    pub message: String,
    pub login_url: Option<String>,
    pub refresh_token: Option<String>,
    pub set_cookies: Vec<String>,
}

pub async fn poll_qrcode(client: &Client, qrcode_key: &str) -> Result<PollResult, String> {
    let response = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll")
        .query(&[("qrcode_key", qrcode_key)])
        .send()
        .await
        .map_err(|e| format!("轮询 QR 码状态失败: {}", e))?;

    let set_cookies: Vec<String> = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .collect();

    let resp: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析轮询响应失败: {}", e))?;

    let data = &resp["data"];
    Ok(PollResult {
        code: data["code"].as_i64().unwrap_or(-1),
        message: data["message"].as_str().unwrap_or("未知状态").to_string(),
        login_url: data["url"].as_str().map(|s| s.to_string()),
        refresh_token: data["refresh_token"].as_str().map(|s| s.to_string()),
        set_cookies,
    })
}

pub fn parse_login_result(poll: &PollResult) -> Result<(BilibiliCookies, Option<String>), String> {
    if let Some(url) = poll.login_url.as_ref() {
        if !url.is_empty() {
            if let Ok(cookies) = cookie::parse_login_url(url) {
                return Ok((cookies, poll.refresh_token.clone()));
            }
        }
    }

    parse_cookies_from_headers(&poll.set_cookies)
        .map(|cookies| (cookies, poll.refresh_token.clone()))
        .map_err(|e| format!("无法从登录响应提取 Cookie: {}", e))
}

fn parse_cookies_from_headers(set_cookies: &[String]) -> Result<BilibiliCookies, String> {
    let mut sessdata = None;
    let mut bili_jct = None;
    let mut dede_user_id = None;
    let mut dede_user_id_ckmd5 = None;

    for header in set_cookies {
        let parts: Vec<&str> = header.splitn(2, ';').collect();
        let kv = parts[0].trim();
        if let Some((key, value)) = kv.split_once('=') {
            match key.trim() {
                "SESSDATA" => sessdata = Some(value.to_string()),
                "bili_jct" => bili_jct = Some(value.to_string()),
                "DedeUserID" => dede_user_id = Some(value.to_string()),
                "DedeUserID__ckMd5" => dede_user_id_ckmd5 = Some(value.to_string()),
                _ => {}
            }
        }
    }

    Ok(BilibiliCookies {
        sessdata: sessdata.ok_or("响应头缺少 SESSDATA")?,
        bili_jct: bili_jct.ok_or("响应头缺少 bili_jct")?,
        dede_user_id: dede_user_id.ok_or("响应头缺少 DedeUserID")?,
        dede_user_id_ckmd5: dede_user_id_ckmd5.ok_or("响应头缺少 DedeUserID__ckMd5")?,
    })
}

// --- UP 主信息 ---

#[derive(Debug, Deserialize)]
pub struct UploaderInfo {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

pub async fn get_uploader_info(client: &Client, wbi: &WbiSigner, mid: i64) -> Result<UploaderInfo, String> {
    let mut params: Vec<(String, String)> = vec![
        ("mid".to_string(), mid.to_string()),
        ("platform".to_string(), "web".to_string()),
    ];

    wbi.sign_params(&mut params).await?;

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!(
        "https://api.bilibili.com/x/space/wbi/acc/info?{}",
        query
    );
    let resp = get_json_with_retry(client, &url).await?;

    let code = resp["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        let msg = resp["message"].as_str().unwrap_or("未知错误");
        return Err(classify_api_error(code, msg).into());
    }

    let data = &resp["data"];
    Ok(UploaderInfo {
        mid: data["mid"].as_i64().ok_or("缺少 mid")?,
        name: data["name"].as_str().ok_or("缺少 name")?.to_string(),
        face: data["face"].as_str().unwrap_or("").to_string(),
    })
}

// --- 视频列表 ---

#[derive(Debug)]
pub struct VideoItem {
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub description: String,
    pub cover_url: String,
    pub duration_str: String,
    pub play_count: i64,
    pub created: i64,
}

#[derive(Debug)]
pub struct VideoListPage {
    pub videos: Vec<VideoItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub async fn fetch_video_list(
    client: &Client,
    wbi: &WbiSigner,
    mid: i64,
    page: i64,
    page_size: i64,
    order: &str,
) -> Result<VideoListPage, String> {
    let mut params: Vec<(String, String)> = vec![
        ("mid".to_string(), mid.to_string()),
        ("ps".to_string(), page_size.to_string()),
        ("pn".to_string(), page.to_string()),
        ("order".to_string(), order.to_string()),
        ("platform".to_string(), "web".to_string()),
        ("web_location".to_string(), "1550101".to_string()),
        ("order_avoided".to_string(), "true".to_string()),
    ];

    wbi.sign_params(&mut params).await?;

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!(
        "https://api.bilibili.com/x/space/wbi/arc/search?{}",
        query
    );

    let resp = get_json_with_retry(client, &url).await?;

    let code = resp["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        let msg = resp["message"].as_str().unwrap_or("未知错误");
        return Err(classify_api_error(code, msg).into());
    }

    let data = &resp["data"];
    let vlist = data["list"]["vlist"]
        .as_array()
        .ok_or("缺少视频列表数据")?;

    let videos: Vec<VideoItem> = vlist
        .iter()
        .map(|v| VideoItem {
            aid: v["aid"].as_i64().unwrap_or(0),
            bvid: v["bvid"].as_str().unwrap_or("").to_string(),
            title: v["title"].as_str().unwrap_or("").to_string(),
            description: v["description"].as_str().unwrap_or("").to_string(),
            cover_url: v["pic"].as_str().unwrap_or("").to_string(),
            duration_str: v["length"].as_str().unwrap_or("0:00").to_string(),
            play_count: v["play"].as_i64().unwrap_or(0),
            created: v["created"].as_i64().unwrap_or(0),
        })
        .collect();

    let page_data = &data["page"];
    Ok(VideoListPage {
        videos,
        total: page_data["count"].as_i64().unwrap_or(0),
        page: page_data["pn"].as_i64().unwrap_or(page),
        page_size: page_data["ps"].as_i64().unwrap_or(page_size),
    })
}

pub fn parse_duration_str(s: &str) -> i64 {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        2 => {
            let mins: i64 = parts[0].parse().unwrap_or(0);
            let secs: i64 = parts[1].parse().unwrap_or(0);
            mins * 60 + secs
        }
        3 => {
            let hours: i64 = parts[0].parse().unwrap_or(0);
            let mins: i64 = parts[1].parse().unwrap_or(0);
            let secs: i64 = parts[2].parse().unwrap_or(0);
            hours * 3600 + mins * 60 + secs
        }
        _ => 0,
    }
}

pub async fn check_cookie_valid(client: &Client) -> Result<bool, String> {
    let resp = get_json_with_retry(
        client,
        "https://api.bilibili.com/x/web-interface/nav",
    )
    .await?;

    Ok(resp["data"]["isLogin"].as_bool().unwrap_or(false))
}

// --- Cookie 刷新 ---

pub async fn get_refresh_csrf(client: &Client) -> Result<String, String> {
    let resp = client
        .get("https://www.bilibili.com/correspond/1/{}")
        .send()
        .await
        .map_err(|e| format!("获取 refresh CSRF 失败: {}", e))?
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let re = regex::Regex::new(r#"<div\s+id="1-name">([^<]+)</div>"#)
        .map_err(|e| format!("正则编译失败: {}", e))?;

    re.captures(&resp)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| "无法从页面提取 refresh_csrf".to_string())
}

pub async fn refresh_cookie(
    client: &Client,
    csrf: &str,
    refresh_csrf: &str,
    refresh_token: &str,
) -> Result<RefreshResult, String> {
    let params = [
        ("csrf", csrf),
        ("refresh_csrf", refresh_csrf),
        ("refresh_token", refresh_token),
        ("source", "main_web"),
    ];

    let resp: serde_json::Value = client
        .post("https://passport.bilibili.com/x/passport-login/web/cookie/refresh")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("刷新 cookie 失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析刷新响应失败: {}", e))?;

    let code = resp["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        return Err(format!(
            "刷新 cookie 失败 (code={}): {}",
            code,
            resp["message"].as_str().unwrap_or("未知错误")
        ));
    }

    let data = &resp["data"];
    Ok(RefreshResult {
        new_refresh_token: data["refresh_token"]
            .as_str()
            .ok_or("缺少新 refresh_token")?
            .to_string(),
    })
}

pub async fn confirm_refresh(
    client: &Client,
    csrf: &str,
    old_refresh_token: &str,
) -> Result<(), String> {
    let params = [
        ("csrf", csrf),
        ("refresh_token", old_refresh_token),
    ];

    let resp: serde_json::Value = client
        .post("https://passport.bilibili.com/x/passport-login/web/confirm/refresh")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("确认刷新失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析确认响应失败: {}", e))?;

    let code = resp["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        return Err(format!(
            "确认刷新失败 (code={}): {}",
            code,
            resp["message"].as_str().unwrap_or("未知错误")
        ));
    }

    Ok(())
}

#[derive(Debug)]
pub struct RefreshResult {
    pub new_refresh_token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_str() {
        assert_eq!(parse_duration_str("12:34"), 754);
        assert_eq!(parse_duration_str("1:02:03"), 3723);
        assert_eq!(parse_duration_str("0:00"), 0);
        assert_eq!(parse_duration_str("invalid"), 0);
    }
}
