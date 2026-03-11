use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReportData {
    #[serde(rename = "type")]
    pub report_type: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub month: Option<String>,
    #[serde(default)]
    pub mouse_clicks: Option<u64>,
    #[serde(default)]
    pub key_strokes: Option<u64>,
    #[serde(default)]
    pub tasks_completed: Option<u32>,
    #[serde(default)]
    pub total_tasks: Option<u32>,
    #[serde(default)]
    pub xp_earned: Option<u32>,
    #[serde(default)]
    pub total_mouse_clicks: Option<u64>,
    #[serde(default)]
    pub total_key_strokes: Option<u64>,
    #[serde(default)]
    pub total_tasks_completed: Option<u32>,
    #[serde(default)]
    pub total_xp_earned: Option<u32>,
    #[serde(default)]
    pub mood_distribution: Option<HashMap<String, u32>>,
    #[serde(default)]
    pub llm_narrative: Option<String>,
    #[serde(default)]
    pub daily_breakdown: Option<Vec<DailyBreakdownItem>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DailyBreakdownItem {
    pub date: String,
    #[serde(default)]
    pub mouse_clicks: u64,
    #[serde(default)]
    pub key_strokes: u64,
    #[serde(default)]
    pub tasks_completed: u32,
    #[serde(default)]
    pub xp_earned: u32,
}

impl ReportData {
    fn title(&self) -> String {
        if self.report_type == "daily" {
            format!("{} 日报", self.date.as_deref().unwrap_or(""))
        } else {
            format!("{} 月报", self.month.as_deref().unwrap_or(""))
        }
    }

    fn mouse(&self) -> u64 {
        self.mouse_clicks.or(self.total_mouse_clicks).unwrap_or(0)
    }

    fn keys(&self) -> u64 {
        self.key_strokes.or(self.total_key_strokes).unwrap_or(0)
    }

    fn completed(&self) -> u32 {
        self.tasks_completed.or(self.total_tasks_completed).unwrap_or(0)
    }

    fn total(&self) -> u32 {
        self.total_tasks.unwrap_or(0)
    }

    fn xp(&self) -> u32 {
        self.xp_earned.or(self.total_xp_earned).unwrap_or(0)
    }
}

/// 加载系统中文字体
fn load_chinese_font_bytes() -> Result<Vec<u8>, String> {
    // Windows 优先使用微软雅黑
    let candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simsun.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        // Linux
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ];

    for path in &candidates {
        if let Ok(bytes) = fs::read(path) {
            return Ok(bytes);
        }
    }

    Err("未找到系统中文字体，请确保系统已安装中文字体".to_string())
}

// ==================== PDF 导出 ====================

#[tauri::command]
pub async fn export_report_pdf(
    report_json: String,
    save_path: String,
) -> Result<String, String> {
    let report: ReportData =
        serde_json::from_str(&report_json).map_err(|e| format!("解析报告数据失败: {}", e))?;

    let font_bytes = load_chinese_font_bytes()?;

    // 在阻塞线程中生成 PDF（printpdf 是同步的）
    let path = save_path.clone();
    tokio::task::spawn_blocking(move || generate_pdf(&report, &font_bytes, &path))
        .await
        .map_err(|e| format!("PDF 生成任务异常: {}", e))?
}

fn generate_pdf(report: &ReportData, font_bytes: &[u8], save_path: &str) -> Result<String, String> {
    use printpdf::*;

    let title = report.title();
    let (doc, page1, layer1) = PdfDocument::new(&title, Mm(210.0), Mm(297.0), "Layer 1");

    let font = doc
        .add_external_font(std::io::Cursor::new(font_bytes))
        .map_err(|e| format!("加载字体失败: {}", e))?;

    let layer = doc.get_page(page1).get_layer(layer1);

    let mut y = 270.0; // 从顶部开始
    let left = 20.0;
    let line_height = 7.0;

    // 标题
    layer.use_text(&title, 18.0, Mm(left), Mm(y), &font);
    y -= 12.0;

    // 分隔线
    let line_points = vec![
        (Point::new(Mm(left), Mm(y)), false),
        (Point::new(Mm(190.0), Mm(y)), false),
    ];
    let line = Line { points: line_points, is_closed: false };
    layer.set_outline_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.7, None)));
    layer.set_outline_thickness(0.5);
    layer.add_line(line);
    y -= 10.0;

    // 统计数据
    layer.use_text("统计概览", 14.0, Mm(left), Mm(y), &font);
    y -= line_height * 1.5;

    let stats_lines = vec![
        format!("鼠标点击：{} 次", report.mouse()),
        format!("键盘敲击：{} 次", report.keys()),
        format!("完成任务：{}/{}", report.completed(), report.total()),
        format!("获得 XP：{}", report.xp()),
    ];

    for line_text in &stats_lines {
        layer.use_text(line_text, 11.0, Mm(left + 4.0), Mm(y), &font);
        y -= line_height;
    }
    y -= 4.0;

    // 心情分布
    if let Some(ref mood) = report.mood_distribution {
        if !mood.is_empty() {
            layer.use_text("心情分布", 14.0, Mm(left), Mm(y), &font);
            y -= line_height * 1.5;

            for (k, v) in mood {
                let text = format!("{}：{} 次", k, v);
                layer.use_text(&text, 11.0, Mm(left + 4.0), Mm(y), &font);
                y -= line_height;
            }
            y -= 4.0;
        }
    }

    // LLM 叙述
    if let Some(ref narrative) = report.llm_narrative {
        if !narrative.is_empty() {
            layer.use_text("猫猫总结", 14.0, Mm(left), Mm(y), &font);
            y -= line_height * 1.5;

            // 简单分行（每行约 40 字）
            let chars: Vec<char> = narrative.chars().collect();
            let chars_per_line = 40;
            let mut i = 0;
            while i < chars.len() {
                let end = (i + chars_per_line).min(chars.len());
                // 尝试在标点处断行
                let mut break_at = end;
                if end < chars.len() {
                    for j in (i..end).rev() {
                        let c = chars[j];
                        if c == '，' || c == '。' || c == '！' || c == '？' || c == '；'
                            || c == '、' || c == '\n'
                        {
                            break_at = j + 1;
                            break;
                        }
                    }
                }
                let line_str: String = chars[i..break_at].iter().collect();
                layer.use_text(&line_str, 11.0, Mm(left + 4.0), Mm(y), &font);
                y -= line_height;
                i = break_at;

                // 如果超出页面底部，就停止（简单处理）
                if y < 20.0 {
                    break;
                }
            }
            y -= 4.0;
        }
    }

    // 月报：每日明细表格
    if let Some(ref breakdown) = report.daily_breakdown {
        if !breakdown.is_empty() && y > 40.0 {
            layer.use_text("每日明细", 14.0, Mm(left), Mm(y), &font);
            y -= line_height * 1.5;

            // 表头
            let header = format!(
                "{:<12} {:>8} {:>8} {:>6} {:>6}",
                "日期", "鼠标", "键盘", "任务", "XP"
            );
            layer.use_text(&header, 10.0, Mm(left + 4.0), Mm(y), &font);
            y -= line_height;

            for item in breakdown {
                if y < 20.0 {
                    break;
                }
                let row = format!(
                    "{:<12} {:>8} {:>8} {:>6} {:>6}",
                    item.date, item.mouse_clicks, item.key_strokes, item.tasks_completed, item.xp_earned
                );
                layer.use_text(&row, 10.0, Mm(left + 4.0), Mm(y), &font);
                y -= line_height;
            }
        }
    }

    // 保存
    let pdf_bytes = doc.save_to_bytes().map_err(|e| format!("PDF 序列化失败: {}", e))?;
    fs::write(save_path, pdf_bytes).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(save_path.to_string())
}

// ==================== Word 导出 ====================

#[tauri::command]
pub async fn export_report_word(
    report_json: String,
    save_path: String,
) -> Result<String, String> {
    let report: ReportData =
        serde_json::from_str(&report_json).map_err(|e| format!("解析报告数据失败: {}", e))?;

    let path = save_path.clone();
    tokio::task::spawn_blocking(move || generate_word(&report, &path))
        .await
        .map_err(|e| format!("Word 生成任务异常: {}", e))?
}

fn generate_word(report: &ReportData, save_path: &str) -> Result<String, String> {
    use docx_rs::*;

    let title = report.title();

    let mut docx = Docx::new();

    // 标题
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(&title)
                    .size(36) // 18pt = 36 half-points
                    .bold()
                    .fonts(RunFonts::new().east_asia("微软雅黑")),
            )
            .align(AlignmentType::Center),
    );

    // 空行
    docx = docx.add_paragraph(Paragraph::new());

    // 统计概览标题
    docx = docx.add_paragraph(
        Paragraph::new().add_run(
            Run::new()
                .add_text("统计概览")
                .size(28)
                .bold()
                .fonts(RunFonts::new().east_asia("微软雅黑")),
        ),
    );

    let stats = vec![
        format!("鼠标点击：{} 次", report.mouse()),
        format!("键盘敲击：{} 次", report.keys()),
        format!("完成任务：{}/{}", report.completed(), report.total()),
        format!("获得 XP：{}", report.xp()),
    ];

    for line in &stats {
        docx = docx.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(line)
                    .size(22)
                    .fonts(RunFonts::new().east_asia("微软雅黑")),
            ),
        );
    }

    // 心情分布
    if let Some(ref mood) = report.mood_distribution {
        if !mood.is_empty() {
            docx = docx.add_paragraph(Paragraph::new());
            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text("心情分布")
                        .size(28)
                        .bold()
                        .fonts(RunFonts::new().east_asia("微软雅黑")),
                ),
            );

            for (k, v) in mood {
                let text = format!("{}：{} 次", k, v);
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text(&text)
                            .size(22)
                            .fonts(RunFonts::new().east_asia("微软雅黑")),
                    ),
                );
            }
        }
    }

    // LLM 叙述
    if let Some(ref narrative) = report.llm_narrative {
        if !narrative.is_empty() {
            docx = docx.add_paragraph(Paragraph::new());
            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text("猫猫总结")
                        .size(28)
                        .bold()
                        .fonts(RunFonts::new().east_asia("微软雅黑")),
                ),
            );
            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(narrative)
                        .size(22)
                        .fonts(RunFonts::new().east_asia("微软雅黑")),
                ),
            );
        }
    }

    // 月报：每日明细表格
    if let Some(ref breakdown) = report.daily_breakdown {
        if !breakdown.is_empty() {
            docx = docx.add_paragraph(Paragraph::new());
            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text("每日明细")
                        .size(28)
                        .bold()
                        .fonts(RunFonts::new().east_asia("微软雅黑")),
                ),
            );

            // 构建表格
            let header_cells = vec!["日期", "鼠标点击", "键盘敲击", "完成任务", "XP"];
            let mut table = Table::new(vec![]);

            // 表头行
            let mut header_cells_vec = vec![];
            for h in &header_cells {
                header_cells_vec.push(
                    TableCell::new().add_paragraph(
                        Paragraph::new().add_run(
                            Run::new()
                                .add_text(*h)
                                .size(20)
                                .bold()
                                .fonts(RunFonts::new().east_asia("微软雅黑")),
                        ),
                    ),
                );
            }
            table = table.add_row(TableRow::new(header_cells_vec));

            // 数据行
            for item in breakdown {
                let cells = vec![
                    item.date.clone(),
                    item.mouse_clicks.to_string(),
                    item.key_strokes.to_string(),
                    item.tasks_completed.to_string(),
                    item.xp_earned.to_string(),
                ];
                let mut row_cells = vec![];
                for c in &cells {
                    row_cells.push(
                        TableCell::new().add_paragraph(
                            Paragraph::new().add_run(
                                Run::new()
                                    .add_text(c.as_str())
                                    .size(20)
                                    .fonts(RunFonts::new().east_asia("微软雅黑")),
                            ),
                        ),
                    );
                }
                table = table.add_row(TableRow::new(row_cells));
            }

            docx = docx.add_table(table);
        }
    }

    // 保存
    let file = fs::File::create(save_path).map_err(|e| format!("创建文件失败: {}", e))?;
    docx.build()
        .pack(file)
        .map_err(|e| format!("写入 Word 文件失败: {}", e))?;

    Ok(save_path.to_string())
}
