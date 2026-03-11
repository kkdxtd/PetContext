use base64::Engine;
use image::imageops::FilterType;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// VLM 提取的单个待办事项
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TodoItem {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
}

/// 自定义反序列化：兼容旧的 Vec<String> 和新的 Vec<TodoItem>
fn deserialize_todos<'de, D>(deserializer: D) -> Result<Vec<TodoItem>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct TodosVisitor;

    impl<'de> de::Visitor<'de> for TodosVisitor {
        type Value = Vec<TodoItem>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an array of strings or TodoItem objects")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<TodoItem>, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut items = Vec::new();
            while let Some(val) = seq.next_element::<serde_json::Value>()? {
                match val {
                    serde_json::Value::String(s) => {
                        items.push(TodoItem { text: s, deadline: None });
                    }
                    serde_json::Value::Object(_) => {
                        let item: TodoItem = serde_json::from_value(val)
                            .map_err(de::Error::custom)?;
                        items.push(item);
                    }
                    _ => {} // 忽略非法类型
                }
            }
            Ok(items)
        }
    }

    deserializer.deserialize_seq(TodosVisitor)
}

/// 截图轮换索引 0/1/2
static SCREENSHOT_IDX: AtomicUsize = AtomicUsize::new(0);

const MAX_SCREENSHOTS: usize = 3;
const MAX_IMAGE_DIM: u32 = 1920;

// ========== 数据结构 ==========

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VlmData {
    #[serde(default)]
    pub activity: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default, deserialize_with = "deserialize_todos")]
    pub todos: Vec<TodoItem>,
    #[serde(default)]
    pub suggestion: String,
    #[serde(default)]
    pub mood: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct VlmResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<VlmData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_path: Option<String>,
}

// ========== 截图 ==========

/// 截取主显示器，保存为 JPEG，返回 (文件路径, base64 字符串)
pub fn capture_screen(save_dir: &Path) -> Result<(PathBuf, String), String> {
    // 确保保存目录存在
    std::fs::create_dir_all(save_dir)
        .map_err(|e| format!("创建截图目录失败: {}", e))?;

    // 获取主显示器
    let monitors = xcap::Monitor::all().map_err(|e| format!("获取显示器失败: {}", e))?;
    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary())
        .or_else(|| xcap::Monitor::all().ok()?.into_iter().next())
        .ok_or_else(|| "未找到可用显示器".to_string())?;

    // 截屏
    let img = monitor
        .capture_image()
        .map_err(|e| format!("截屏失败: {}", e))?;

    // 缩放（如超过 MAX_IMAGE_DIM）
    let (w, h) = (img.width(), img.height());
    let img = if w > MAX_IMAGE_DIM || h > MAX_IMAGE_DIM {
        let ratio = MAX_IMAGE_DIM as f64 / w.max(h) as f64;
        let new_w = (w as f64 * ratio) as u32;
        let new_h = (h as f64 * ratio) as u32;
        image::DynamicImage::ImageRgba8(img)
            .resize_exact(new_w, new_h, FilterType::Triangle)
    } else {
        image::DynamicImage::ImageRgba8(img)
    };

    // 编码为 JPEG bytes
    let mut jpeg_buf = Cursor::new(Vec::new());
    img.write_to(&mut jpeg_buf, image::ImageFormat::Jpeg)
        .map_err(|e| format!("JPEG 编码失败: {}", e))?;
    let jpeg_bytes = jpeg_buf.into_inner();

    // 轮换保存：screenshot_0.jpg, screenshot_1.jpg, screenshot_2.jpg
    let idx = SCREENSHOT_IDX.fetch_add(1, Ordering::Relaxed) % MAX_SCREENSHOTS;
    let filename = format!("screenshot_{}.jpg", idx);
    let filepath = save_dir.join(&filename);
    std::fs::write(&filepath, &jpeg_bytes)
        .map_err(|e| format!("保存截图失败: {}", e))?;

    // base64 编码
    let b64 = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);

    Ok((filepath, b64))
}

// ========== VLM API 调用 ==========

const VLM_SYSTEM_PROMPT: &str = r#"你是用户的桌面活动分析助手。根据用户的屏幕截图，用简洁的中文描述：
1. 用户当前在做什么（应用/网页/文档）
2. 屏幕上的关键信息摘要
3. 如果发现待办事项、截止日期或重要信息，提取出来。每个待办包含文本和截止日期（如果能从屏幕内容推断）
4. 给用户一个简短的状态建议

以 JSON 格式输出：
{"activity": "当前活动描述", "summary": "关键信息摘要", "todos": [{"text": "待办内容", "deadline": "2025-03-15T18:00:00"}, {"text": "无截止日期的待办", "deadline": null}], "suggestion": "简短建议", "mood": "工作/学习/娱乐/休息"}

注意：
- todos 数组中每个元素必须是对象，包含 text（待办内容）和 deadline（ISO 8601 格式的截止时间，无法确定则为 null）
- 如果屏幕上有"明天""下周""后天"等相对日期，请结合用户提供的当前日期换算为绝对日期
- 只返回JSON"#;

pub async fn call_vlm(
    api_url: &str,
    api_key: &str,
    model: &str,
    image_base64: &str,
    prompt: &str,
) -> Result<VlmData, String> {
    let client = Client::new();

    // 注入当前日期，供 VLM 换算相对日期
    let now = chrono::Local::now();
    let date_prefix = format!(
        "【当前时间：{}，星期{}】\n",
        now.format("%Y-%m-%d %H:%M"),
        match now.format("%u").to_string().as_str() {
            "1" => "一", "2" => "二", "3" => "三", "4" => "四",
            "5" => "五", "6" => "六", "7" => "日", _ => ""
        }
    );
    let full_prompt = format!("{}{}", date_prefix, prompt);

    // 构建多模态消息（使用 serde_json::Value 以支持 content 数组）
    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": VLM_SYSTEM_PROMPT
            },
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": full_prompt },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/jpeg;base64,{}", image_base64)
                        }
                    }
                ]
            }
        ],
        "max_tokens": 1024
    });

    let response = client
        .post(&format!("{}/chat/completions", api_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "VLM 请求超时（60s）".to_string()
            } else {
                format!("VLM 请求失败: {}", e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("VLM API 错误 {}: {}", status, body));
    }

    #[derive(Deserialize)]
    struct Resp {
        choices: Vec<RespChoice>,
    }
    #[derive(Deserialize)]
    struct RespChoice {
        message: RespMsg,
    }
    #[derive(Deserialize)]
    struct RespMsg {
        content: String,
    }

    let resp: Resp = response
        .json()
        .await
        .map_err(|e| format!("解析 VLM 响应失败: {}", e))?;

    let content = resp
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    parse_vlm_json(&content)
}

/// 从 VLM 返回文本中提取 JSON
fn parse_vlm_json(text: &str) -> Result<VlmData, String> {
    let mut cleaned = text.trim().to_string();

    // 去掉 <think>...</think>
    if let Some(pos) = cleaned.find("</think>") {
        cleaned = cleaned[pos + 8..].trim().to_string();
    }

    // 去掉 markdown ```json ... ```
    if cleaned.starts_with("```") {
        cleaned = cleaned
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .to_string();
        if let Some(pos) = cleaned.rfind("```") {
            cleaned = cleaned[..pos].to_string();
        }
        cleaned = cleaned.trim().to_string();
    }

    // 尝试直接解析
    if let Ok(data) = serde_json::from_str::<VlmData>(&cleaned) {
        return Ok(data);
    }

    // 尝试找 {...} 块
    if let (Some(start), Some(end)) = (cleaned.find('{'), cleaned.rfind('}')) {
        let json_str = &cleaned[start..=end];
        if let Ok(data) = serde_json::from_str::<VlmData>(json_str) {
            return Ok(data);
        }
    }

    // 兜底：把原始文本作为 activity 返回
    Ok(VlmData {
        activity: cleaned,
        summary: String::new(),
        todos: Vec::new(),
        suggestion: String::new(),
        mood: "未知".to_string(),
    })
}
