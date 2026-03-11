mod device;
mod llm;
mod report;
mod vlm;

use device::{start_device_listening, get_and_reset_input_counts};
use llm::{
    breakdown_task as llm_breakdown_task,
    call_llm as llm_call_llm,
    chat_with_model as llm_chat_with_model,
    test_model_connection as llm_test_model_connection,
    Message,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{generate_handler, Emitter, Manager};

/// 全局久坐提醒间隔（秒），前端可通过命令修改
static REMINDER_INTERVAL_SECS: AtomicU64 = AtomicU64::new(45 * 60); // 默认 45 分钟

#[tauri::command]
async fn call_llm(
    api_url: String,
    api_key: String,
    model: String,
    messages: Vec<Message>,
) -> Result<String, String> {
    llm_call_llm(&api_url, &api_key, &model, messages).await
}

#[tauri::command]
async fn breakdown_task(
    api_url: String,
    api_key: String,
    model: String,
    task: String,
) -> Result<llm::TaskBreakdownResult, String> {
    llm_breakdown_task(&api_url, &api_key, &model, &task).await
}

#[tauri::command]
async fn chat_with_model(
    api_url: String,
    api_key: String,
    model: String,
    message: String,
) -> Result<String, String> {
    llm_chat_with_model(&api_url, &api_key, &model, &message).await
}

#[tauri::command]
async fn test_model_connection(
    api_url: String,
    api_key: String,
    model: String,
) -> Result<bool, String> {
    llm_test_model_connection(&api_url, &api_key, &model).await
}

#[tauri::command]
async fn set_reminder_interval(minutes: u64) -> Result<(), String> {
    if minutes < 1 || minutes > 480 {
        return Err("间隔需在 1-480 分钟之间".to_string());
    }
    REMINDER_INTERVAL_SECS.store(minutes * 60, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn get_reminder_interval() -> u64 {
    REMINDER_INTERVAL_SECS.load(Ordering::Relaxed) / 60
}

// ======== VLM 截屏分析（Rust 原生） ========

/// 自动截屏分析开关
static AUTO_CAPTURE_ENABLED: AtomicBool = AtomicBool::new(false);
/// 自动截屏间隔（秒），默认 10 秒
static AUTO_CAPTURE_INTERVAL_SECS: AtomicU64 = AtomicU64::new(10);

/// 截屏 + 调用 VLM 分析（Rust 原生实现）
#[tauri::command]
async fn capture_and_analyze(
    app: tauri::AppHandle,
    api_url: String,
    api_key: String,
    model: String,
    prompt: Option<String>,
) -> Result<vlm::VlmResult, String> {
    // 截图保存目录：app_data_dir/screenshots/
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    let screenshot_dir = data_dir.join("screenshots");

    // 截屏（在阻塞线程中执行，因为 xcap 是同步的）
    let dir = screenshot_dir.clone();
    let (filepath, b64) = tokio::task::spawn_blocking(move || vlm::capture_screen(&dir))
        .await
        .map_err(|e| format!("截屏任务异常: {}", e))?
        ?;

    // 调用 VLM API
    let user_prompt = prompt.unwrap_or_else(|| "请分析我的屏幕截图，告诉我在做什么。".to_string());
    let data = vlm::call_vlm(&api_url, &api_key, &model, &b64, &user_prompt).await?;

    Ok(vlm::VlmResult {
        success: true,
        data: Some(data),
        error: None,
        screenshot_path: Some(filepath.to_string_lossy().to_string()),
    })
}

/// 设置自动截屏间隔（秒，1-60）
#[tauri::command]
async fn set_auto_capture_interval(seconds: u64) -> Result<(), String> {
    if seconds < 1 || seconds > 60 {
        return Err("间隔需在 1-60 秒之间".to_string());
    }
    AUTO_CAPTURE_INTERVAL_SECS.store(seconds, Ordering::Relaxed);
    Ok(())
}

/// 获取自动截屏间隔（秒）
#[tauri::command]
async fn get_auto_capture_interval() -> u64 {
    AUTO_CAPTURE_INTERVAL_SECS.load(Ordering::Relaxed)
}

/// 开启/关闭自动截屏
#[tauri::command]
async fn set_auto_capture_enabled(enabled: bool) {
    AUTO_CAPTURE_ENABLED.store(enabled, Ordering::Relaxed);
}

/// 获取自动截屏开关状态
#[tauri::command]
async fn get_auto_capture_enabled() -> bool {
    AUTO_CAPTURE_ENABLED.load(Ordering::Relaxed)
}

/// 启动自动截屏后台循环
fn start_auto_capture_loop(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let interval = AUTO_CAPTURE_INTERVAL_SECS.load(Ordering::Relaxed);
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

            if !AUTO_CAPTURE_ENABLED.load(Ordering::Relaxed) {
                continue;
            }

            // 通知前端触发分析（VLM 配置在前端 localStorage）
            let _ = app_handle.emit("vlm-auto-trigger", ());
        }
    });
}

#[tauri::command]
async fn create_chat_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("chat") {
        let _ = win.set_focus();
        return Ok(());
    }

    let main_window = app
        .get_webview_window("main")
        .ok_or("无法获取主窗口")?;

    let main_pos = main_window
        .outer_position()
        .map_err(|e| format!("获取窗口位置失败: {}", e))?;

    let main_size = main_window
        .outer_size()
        .map_err(|e| format!("获取窗口大小失败: {}", e))?;

    let chat_w: i32 = 400;
    let chat_h: i32 = 500;
    let gap: i32 = 10;

    let main_x = main_pos.x;
    let main_y = main_pos.y;
    let main_w = main_size.width as i32;
    let main_h = main_size.height as i32;

    // 获取主窗口所在显示器的屏幕边界
    let (screen_x, screen_y, screen_w, screen_h) = match main_window.current_monitor() {
        Ok(Some(monitor)) => {
            let pos = monitor.position();
            let size = monitor.size();
            (pos.x, pos.y, size.width as i32, size.height as i32)
        }
        _ => (0, 0, 1920, 1080), // 兜底默认值
    };

    // 计算主窗口四个方向的可用空间
    let space_right = (screen_x + screen_w) - (main_x + main_w);
    let space_left = main_x - screen_x;
    let space_bottom = (screen_y + screen_h) - (main_y + main_h);
    let space_top = main_y - screen_y;

    // 优先水平方向（右 > 左），其次垂直方向（下 > 上）
    // 选择能容纳聊天窗口且空间最大的方向
    let (mut chat_x, mut chat_y) = if space_right >= chat_w + gap {
        // 右侧空间足够
        (main_x + main_w + gap, main_y)
    } else if space_left >= chat_w + gap {
        // 左侧空间足够
        (main_x - chat_w - gap, main_y)
    } else if space_bottom >= chat_h + gap {
        // 下方空间足够
        (main_x, main_y + main_h + gap)
    } else if space_top >= chat_h + gap {
        // 上方空间足够
        (main_x, main_y - chat_h - gap)
    } else {
        // 都不够，选空间最大的方向放置
        let spaces = [
            (space_right, 0),  // 右
            (space_left, 1),   // 左
            (space_bottom, 2), // 下
            (space_top, 3),    // 上
        ];
        let best = spaces.iter().max_by_key(|(s, _)| *s).unwrap().1;
        match best {
            0 => (main_x + main_w + gap, main_y),
            1 => (main_x - chat_w - gap, main_y),
            2 => (main_x, main_y + main_h + gap),
            _ => (main_x, main_y - chat_h - gap),
        }
    };

    // 确保窗口不超出屏幕边界
    chat_x = chat_x.max(screen_x).min(screen_x + screen_w - chat_w);
    chat_y = chat_y.max(screen_y).min(screen_y + screen_h - chat_h);

    tauri::WebviewWindowBuilder::new(
        &app,
        "chat",
        tauri::WebviewUrl::App("chat.html".into()),
    )
    .title("PetContext 对话")
    .inner_size(400.0, 500.0)
    .position(chat_x as f64, chat_y as f64)
    .resizable(true)
    .focused(true)
    .build()
    .map_err(|e| format!("创建窗口失败: {}", e))?;

    Ok(())
}

/// 启动久坐提醒后台循环
fn start_sedentary_reminder(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let tips = [
            "已经坐了好久啦！起来伸个懒腰，活动一下肩膀和脖子吧~",
            "主人该休息啦！去倒杯水，看看窗外的风景~",
            "久坐不好哦！站起来走动走动，让血液流通一下~",
            "休息一下眼睛吧！看看远处的绿色植物，做几次深呼吸~",
            "该活动啦！试试扭扭腰、转转脚踝，简单拉伸一下~",
            "猫猫提醒：坐太久对腰椎不好！起来走几步再回来~",
            "摸摸你的脖子是不是有点僵？站起来转转头放松一下~",
            "喝水了吗？起来倒杯温水，顺便活动活动腿脚~",
        ];
        let mut tip_idx: usize = 0;

        loop {
            let interval = REMINDER_INTERVAL_SECS.load(Ordering::Relaxed);
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

            let tip = tips[tip_idx % tips.len()];
            tip_idx += 1;

            #[derive(Clone, serde::Serialize)]
            struct BubblePayload {
                text: String,
            }

            let _ = app_handle.emit("cat-bubble", BubblePayload {
                text: tip.to_string(),
            });
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = app.get_webview_window("main");
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_process::init())
        .invoke_handler(generate_handler![
            start_device_listening,
            get_and_reset_input_counts,
            call_llm,
            breakdown_task,
            chat_with_model,
            test_model_connection,
            create_chat_window,
            set_reminder_interval,
            get_reminder_interval,
            capture_and_analyze,
            set_auto_capture_interval,
            get_auto_capture_interval,
            set_auto_capture_enabled,
            get_auto_capture_enabled,
            report::export_report_pdf,
            report::export_report_word
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_decorations(false).unwrap();
            window.set_always_on_top(true).unwrap();

            // 启动久坐提醒
            start_sedentary_reminder(app.handle().clone());

            // 启动自动截屏分析循环
            start_auto_capture_loop(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
