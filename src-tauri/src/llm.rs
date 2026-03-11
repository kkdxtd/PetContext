use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubTask {
    pub id: u32,
    pub name: String,
    pub xp: u32,
    #[serde(rename = "time")]
    pub duration: String,
    #[serde(default)]
    pub action: String,
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskBreakdownResult {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub task: String,
    #[serde(default)]
    pub energy: String,
    pub sub_tasks: Vec<SubTask>,
    #[serde(default, rename = "reward")]
    pub reward: String,
    #[serde(default, rename = "shield")]
    pub shield: String,
    #[serde(rename = "health_tip")]
    pub health_tip: String,
    #[serde(rename = "total_xp")]
    pub total_xp: u32,
}

pub async fn call_llm(
    api_url: &str,
    api_key: &str,
    model: &str,
    messages: Vec<Message>,
) -> Result<String, String> {
    let client = Client::new();
    
    let request_body = ChatRequest {
        model: model.to_string(),
        messages,
        response_format: Some([("type".to_string(), "json_object".to_string())].into_iter().collect()),
    };
    
    let response = client
        .post(&format!("{}/chat/completions", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "请求超时，请检查网络或稍后重试".to_string()
            } else {
                format!("请求失败: {}", e)
            }
        })?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 返回错误 {}: {}", status, body));
    }
    
    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default())
}

pub fn build_system_prompt() -> String {
    r#"你是一位精通 ADHD 执行功能辅助的猫猫桌宠教练。你的核心能力是把用户模糊的大任务拆解为"原子级"小步骤，让用户无需思考就能开始行动。

## 拆解原则
1. **原子化**：每个步骤必须在 5-15 分钟内可完成，是一个具体的、可视化的物理动作（如"打开XX软件""在搜索栏输入XX"），而非抽象描述。
2. **低启动阻力**：第一步必须极其简单（2分钟以内），降低开始的心理门槛。
3. **多巴胺驱动**：每个步骤用 emoji 开头，描述要有趣、有新奇感，像游戏任务。
4. **正向反馈**：XP 奖励要让人有成就感，完成越难的步骤 XP 越高。

## 输出格式（严格JSON，不要其他文字）
{
  "type": "task_breakdown",
  "task": "用一句话重新表述的核心目标",
  "energy": "low/mid/high（完成整个任务需要的能量等级）",
  "sub_tasks": [
    {
      "id": 1,
      "name": "emoji + 步骤名（简短有趣）",
      "xp": 经验值数字,
      "time": "预估耗时（如5分钟）",
      "action": "具体到手指该放哪的最小启动动作",
      "done": false
    }
  ],
  "reward": "完成全部任务后的一个具体小奖励建议，要有诱惑力",
  "shield": "针对这个任务最可能的分心陷阱，给一个温馨的预防建议",
  "health_tip": "一句轻松的鼓励或健康提醒，口吻像关心主人的猫猫",
  "total_xp": 所有步骤xp之和
}

只返回JSON。"#.to_string()
}

pub async fn test_model_connection(
    api_url: &str,
    api_key: &str,
    model: &str,
) -> Result<bool, String> {
    let client = Client::new();
    
    let test_message = Message {
        role: "user".to_string(),
        content: "hello".to_string(),
    };
    
    let request_body = ChatRequest {
        model: model.to_string(),
        messages: vec![test_message],
        response_format: None,
    };
    
    let response = client
        .post(&format!("{}/chat/completions", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("连接测试失败: {}", e))?;
    
    Ok(response.status().is_success())
}
pub async fn chat_with_model(
    api_url: &str,
    api_key: &str,
    model: &str,
    message: &str,
) -> Result<String, String> {
    let client = Client::new();

    let request_body = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: message.to_string(),
            },
        ],
        response_format: None,
    };

    let response = client
        .post(&format!("{}/chat/completions", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 返回错误: {}", response.status()));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default())
}

pub async fn breakdown_task(
    api_url: &str,
    api_key: &str,
    model: &str,
    task: &str,
) -> Result<TaskBreakdownResult, String> {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: build_system_prompt(),
        },
        Message {
            role: "user".to_string(),
            content: format!("请帮我拆解任务：{}", task),
        },
    ];
    
    let result = call_llm(api_url, api_key, model, messages).await?;

    // 清理可能的 markdown 代码块包裹和思考标签
    let cleaned = result.trim();
    let cleaned = if cleaned.starts_with("```") {
        // 去掉 ```json ... ``` 包裹
        let inner = cleaned.trim_start_matches("```json").trim_start_matches("```");
        inner.trim_end_matches("```").trim()
    } else {
        cleaned
    };
    // 去掉 <think>...</think> 标签
    let cleaned = if let Some(pos) = cleaned.find("</think>") {
        cleaned[pos + 8..].trim()
    } else {
        cleaned
    };

    let breakdown: TaskBreakdownResult = serde_json::from_str(cleaned)
        .map_err(|e| format!("解析JSON失败: {} - 原始内容: {}", e, &result[..result.len().min(200)]))?;
    
    Ok(breakdown)
}
