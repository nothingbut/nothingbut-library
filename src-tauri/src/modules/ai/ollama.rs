use crate::errors::{AppError, AppResult};
use super::models::*;
use reqwest::Client;
use std::time::Duration;

/// Ollama HTTP 客户端
pub struct OllamaClient {
    base_url: String,
    client: Client,
}

impl OllamaClient {
    /// 创建新的 Ollama 客户端
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // 2 分钟超时
            .build()
            .expect("Failed to create HTTP client");

        Self { base_url, client }
    }

    /// 创建默认客户端（localhost:11434）
    pub fn default() -> Self {
        Self::new("http://localhost:11434".to_string())
    }

    /// 检查 Ollama 服务健康状态
    pub async fn health_check(&self) -> AppResult<bool> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// 生成文本（非流式）
    pub async fn generate(&self, prompt: &str, model: &str) -> AppResult<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(GenerateOptions {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: None,
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Module(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        let generate_response: GenerateResponse = response
            .json()
            .await
            .map_err(|e| AppError::Module(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(generate_response.response)
    }

    /// 对话（非流式，用于测试）
    pub async fn chat(&self, messages: Vec<ChatMessage>, model: &str) -> AppResult<String> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
            options: Some(GenerateOptions {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: None,
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("Ollama chat request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Module(format!(
                "Ollama chat API error ({}): {}",
                status, error_text
            )));
        }

        let chat_response: ChatResponseChunk = response
            .json()
            .await
            .map_err(|e| AppError::Module(format!("Failed to parse Ollama chat response: {}", e)))?;

        Ok(chat_response.message.content)
    }

    /// 对话（流式）
    /// 返回一个异步迭代器，逐个返回响应片段
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
    ) -> AppResult<impl futures::Stream<Item = AppResult<String>>> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: true,
            options: Some(GenerateOptions {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: None,
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("Ollama stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(AppError::Module(format!(
                "Ollama stream API error: {}",
                status
            )));
        }

        // 使用 bytes_stream 处理流式响应
        use futures::StreamExt;

        let stream = response.bytes_stream().map(|result| {
            result
                .map_err(|e| AppError::Module(format!("Stream error: {}", e)))
                .and_then(|bytes| {
                    // 解析每一行 JSON
                    let text = String::from_utf8(bytes.to_vec())
                        .map_err(|e| AppError::Module(format!("UTF-8 decode error: {}", e)))?;

                    // Ollama 流式响应是每行一个 JSON 对象
                    let mut content = String::new();
                    for line in text.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        let chunk: ChatResponseChunk = serde_json::from_str(line)
                            .map_err(|e| AppError::Module(format!("JSON parse error: {}", e)))?;

                        content.push_str(&chunk.message.content);

                        if chunk.done {
                            break;
                        }
                    }
                    Ok(content)
                })
        });

        Ok(stream)
    }

    /// 生成嵌入向量
    pub async fn embeddings(&self, text: &str, model: &str) -> AppResult<Vec<f32>> {
        let url = format!("{}/api/embed", self.base_url);

        let request = EmbeddingsRequest {
            model: model.to_string(),
            input: text.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("Ollama embeddings request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Module(format!(
                "Ollama embeddings API error ({}): {}",
                status, error_text
            )));
        }

        let embeddings_response: EmbeddingsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Module(format!("Failed to parse embeddings response: {}", e)))?;

        // 返回第一个嵌入向量（单个文本输入）
        embeddings_response
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Module("No embeddings returned".to_string()))
    }

    /// 批量生成嵌入向量
    pub async fn batch_embeddings(
        &self,
        texts: Vec<String>,
        model: &str,
    ) -> AppResult<Vec<Vec<f32>>> {
        use futures::future;

        // 限制并发数为 5
        let chunks: Vec<_> = texts.chunks(5).collect();
        let mut all_embeddings = Vec::new();

        for chunk in chunks {
            let futures: Vec<_> = chunk
                .iter()
                .map(|text| self.embeddings(text, model))
                .collect();

            let results = future::join_all(futures).await;

            for result in results {
                all_embeddings.push(result?);
            }
        }

        Ok(all_embeddings)
    }

    /// 对话（带工具调用支持）
    pub async fn chat_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<Tool>,
        model: &str,
    ) -> AppResult<ChatResponseWithTools> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequestWithTools {
            model: model.to_string(),
            messages,
            stream: false,
            tools: Some(tools),
            options: Some(GenerateOptions {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: None,
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("Ollama chat with tools request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Module(format!(
                "Ollama chat with tools API error ({}): {}",
                status, error_text
            )));
        }

        let chat_response: ChatResponseWithTools = response
            .json()
            .await
            .map_err(|e| AppError::Module(format!("Failed to parse Ollama chat response: {}", e)))?;

        Ok(chat_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要本地运行 Ollama 服务
    // 可以通过 `cargo test --  --ignored` 运行

    #[tokio::test]
    #[ignore]
    async fn test_health_check() {
        let client = OllamaClient::default();
        let healthy = client.health_check().await.unwrap();
        assert!(healthy, "Ollama service should be running");
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate() {
        let client = OllamaClient::default();
        let response = client
            .generate("Say hello in Chinese", "qwen2.5:7b")
            .await
            .unwrap();

        println!("Generated: {}", response);
        assert!(!response.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_chat() {
        let client = OllamaClient::default();
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: "What is 2+2?".to_string(),
            },
        ];

        let response = client.chat(messages, "qwen2.5:7b").await.unwrap();

        println!("Chat response: {}", response);
        assert!(!response.is_empty());
        assert!(response.contains("4"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_embeddings() {
        let client = OllamaClient::default();
        let embedding = client
            .embeddings("Hello world", "nomic-embed-text")
            .await
            .unwrap();

        println!("Embedding dimensions: {}", embedding.len());
        // nomic-embed-text 输出 768 维向量
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    #[ignore]
    async fn test_batch_embeddings() {
        let client = OllamaClient::default();
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = client
            .batch_embeddings(texts, "nomic-embed-text")
            .await
            .unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in embeddings {
            assert_eq!(emb.len(), 768);
        }
    }
}
