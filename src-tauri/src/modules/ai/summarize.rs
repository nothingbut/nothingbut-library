use crate::errors::AppResult;
use super::{models::SummaryLength, OllamaClient};

const SUMMARY_PROMPT_TEMPLATE: &str = r#"你是一位专业的小说分析助手。请为以下内容生成一个{length}摘要。

要求：
- 简洁明了，突出重点
- 保留关键情节和人物
- 不要添加原文没有的内容
- 使用第三人称叙述

内容：
{content}

摘要："#;

pub fn build_summary_prompt(content: &str, length: SummaryLength) -> String {
    let length_desc = length.description();
    SUMMARY_PROMPT_TEMPLATE
        .replace("{length}", length_desc)
        .replace("{content}", content)
}

pub async fn generate_summary(
    client: &OllamaClient,
    content: &str,
    length: SummaryLength,
    model: &str,
) -> AppResult<String> {
    let prompt = build_summary_prompt(content, length);
    client.generate(&prompt, model).await
}
