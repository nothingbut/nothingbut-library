use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use reqwest::Client;

const WEREAD_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(WEREAD_UA));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://weread.qq.com/"),
    );
    headers
}

pub fn build_authed_client(cookies_json: &str) -> Result<Client, String> {
    let cookies: std::collections::HashMap<String, String> =
        serde_json::from_str(cookies_json)
            .map_err(|e| format!("解析 cookies 失败: {}", e))?;

    let cookie_str: String = cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ");

    let mut headers = default_headers();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&cookie_str)
            .map_err(|e| format!("设置 Cookie header 失败: {}", e))?,
    );

    Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("创建认证客户端失败: {}", e))
}

pub struct UserInfo {
    pub vid: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

pub async fn check_login(client: &Client, vid: &str) -> Result<Option<UserInfo>, String> {
    let url = format!("https://weread.qq.com/web/user?userVid={}", vid);
    let resp: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("验证登录状态失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let err_code = resp.get("errCode").and_then(|v| v.as_i64());
    match err_code {
        Some(-2012) | Some(-2010) => Ok(None),
        Some(0) | None => {
            let name = resp.get("name").and_then(|v| v.as_str()).unwrap_or("用户");
            let avatar = resp.get("avatar").and_then(|v| v.as_str());
            let user_vid = resp
                .get("userVid")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| vid.to_string());

            Ok(Some(UserInfo {
                vid: user_vid,
                username: name.to_string(),
                avatar_url: avatar.map(|s| s.to_string()),
            }))
        }
        Some(code) => {
            let msg = resp
                .get("errMsg")
                .and_then(|v| v.as_str())
                .unwrap_or("未知错误");
            Err(format!("微信读书 API 错误 ({}): {}", code, msg))
        }
    }
}

#[derive(Debug, Clone)]
pub struct BookMeta {
    pub book_id: String,
    pub title: String,
    pub author: String,
    pub cover: String,
    pub intro: String,
    pub category: String,
    pub word_count: i64,
    pub chapter_count: i64,
    pub chapters: Vec<ChapterMeta>,
}

#[derive(Debug, Clone)]
pub struct ChapterMeta {
    pub chapter_uid: String,
    pub title: String,
    pub level: i64,
    pub word_count: i64,
}

pub async fn fetch_book_detail(client: &Client, book_id: &str) -> Result<BookMeta, String> {
    let info_url = format!("https://weread.qq.com/web/book/info?bookId={}", book_id);
    let info_resp: serde_json::Value = client
        .get(&info_url)
        .send()
        .await
        .map_err(|e| format!("获取书籍信息失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析书籍信息失败: {}", e))?;

    let title = info_resp["title"].as_str().unwrap_or("未知书名").to_string();
    let author = info_resp["author"].as_str().unwrap_or("未知作者").to_string();
    let cover = info_resp["cover"].as_str().unwrap_or("").to_string();
    let intro = info_resp["intro"].as_str().unwrap_or("").to_string();
    let category = info_resp["category"].as_str().unwrap_or("").to_string();
    let word_count = info_resp["totalWords"].as_i64()
        .or_else(|| info_resp["wordCount"].as_i64())
        .unwrap_or(0);

    let chapters = fetch_chapter_infos(client, book_id).await?;
    let chapter_count = chapters.len() as i64;

    Ok(BookMeta {
        book_id: book_id.to_string(),
        title,
        author,
        cover,
        intro,
        category,
        word_count,
        chapter_count,
        chapters,
    })
}

async fn fetch_chapter_infos(client: &Client, book_id: &str) -> Result<Vec<ChapterMeta>, String> {
    let body = serde_json::json!({
        "bookIds": [book_id],
        "synckeys": [0]
    });

    let resp: serde_json::Value = client
        .post("https://weread.qq.com/web/book/publicChapterInfos")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("获取章节列表失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析章节列表失败: {}", e))?;

    let data = resp
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or("章节数据格式错误")?;

    let book_data = data
        .first()
        .ok_or("无章节数据")?;

    let updated = book_data
        .get("updated")
        .and_then(|v| v.as_array())
        .ok_or("章节列表为空")?;

    let chapters: Vec<ChapterMeta> = updated
        .iter()
        .map(|ch| ChapterMeta {
            chapter_uid: ch["chapterUid"]
                .as_u64()
                .map(|u| u.to_string())
                .or_else(|| ch["chapterUid"].as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
            title: ch["title"].as_str().unwrap_or("").to_string(),
            level: ch["level"].as_i64().unwrap_or(1),
            word_count: ch["wordCount"].as_i64().unwrap_or(0),
        })
        .collect();

    Ok(chapters)
}

pub async fn fetch_shelf(client: &Client) -> Result<Vec<BookMeta>, String> {
    let url = "https://weread.qq.com/web/shelf/sync";
    let resp: serde_json::Value = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("获取书架失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析书架响应失败: {}", e))?;

    if let Some(err_code) = resp.get("errCode").and_then(|v| v.as_i64()) {
        if err_code != 0 {
            let msg = resp.get("errMsg").and_then(|v| v.as_str()).unwrap_or("未知错误");
            return Err(format!("微信读书 API 错误 ({}): {}，请重新登录", err_code, msg));
        }
    }

    let books = resp
        .get("books")
        .and_then(|v| v.as_array())
        .ok_or("书架数据格式错误：响应中无 books 字段")?;

    let results: Vec<BookMeta> = books
        .iter()
        .filter_map(|item| {
            let book_id = item["bookId"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| item["bookId"].as_u64().map(|n| n.to_string()))?;
            let title = item["title"].as_str()?.to_string();
            let author = item["author"].as_str().unwrap_or("").to_string();
            let cover = item["cover"].as_str().unwrap_or("").to_string();
            let intro = item["intro"].as_str().unwrap_or("").to_string();
            let category = item["category"].as_str().unwrap_or("").to_string();
            let word_count = item["wordCount"].as_i64().unwrap_or(0);
            let chapter_count = item["lastChapterIdx"].as_i64().unwrap_or(0);

            Some(BookMeta {
                book_id,
                title,
                author,
                cover,
                intro,
                category,
                word_count,
                chapter_count,
                chapters: vec![],
            })
        })
        .collect();

    Ok(results)
}
