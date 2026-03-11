use rdev::{Event, EventType, listen};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use tauri::{command, AppHandle, Emitter, Runtime};

#[derive(Debug, Clone, Serialize)]
pub enum DeviceEventKind {
    MousePress,
    MouseRelease,
    MouseMove,
    KeyboardPress,
    KeyboardRelease,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceEvent {
    kind: DeviceEventKind,
    value: Value,
}

static IS_LISTENING: AtomicBool = AtomicBool::new(false);

/// 鼠标点击累计计数器
static MOUSE_CLICK_COUNT: AtomicU64 = AtomicU64::new(0);
/// 键盘敲击累计计数器
static KEY_STROKE_COUNT: AtomicU64 = AtomicU64::new(0);

/// 原子读取并清零输入计数，返回 [mouse_clicks, key_strokes]
#[command]
pub async fn get_and_reset_input_counts() -> Vec<u64> {
    let mouse = MOUSE_CLICK_COUNT.swap(0, Ordering::Relaxed);
    let keys = KEY_STROKE_COUNT.swap(0, Ordering::Relaxed);
    vec![mouse, keys]
}

#[command]
pub async fn start_device_listening<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    if IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(());
    }

    IS_LISTENING.store(true, Ordering::SeqCst);

    let handle = app_handle.clone();
    
    thread::spawn(move || {
        let callback = move |event: Event| {
            let device_event = match event.event_type {
                EventType::ButtonPress(button) => {
                    MOUSE_CLICK_COUNT.fetch_add(1, Ordering::Relaxed);
                    DeviceEvent {
                        kind: DeviceEventKind::MousePress,
                        value: json!(format!("{:?}", button)),
                    }
                },
                EventType::ButtonRelease(button) => DeviceEvent {
                    kind: DeviceEventKind::MouseRelease,
                    value: json!(format!("{:?}", button)),
                },
                EventType::MouseMove { x, y } => DeviceEvent {
                    kind: DeviceEventKind::MouseMove,
                    value: json!({ "x": x, "y": y }),
                },
                EventType::KeyPress(key) => {
                    KEY_STROKE_COUNT.fetch_add(1, Ordering::Relaxed);
                    DeviceEvent {
                        kind: DeviceEventKind::KeyboardPress,
                        value: json!(format!("{:?}", key)),
                    }
                },
                EventType::KeyRelease(key) => DeviceEvent {
                    kind: DeviceEventKind::KeyboardRelease,
                    value: json!(format!("{:?}", key)),
                },
                _ => return,
            };

            let _ = handle.emit("device-changed", device_event);
        };

        if let Err(err) = listen(callback) {
            log::error!("Failed to listen device: {:?}", err);
        }
    });

    Ok(())
}
