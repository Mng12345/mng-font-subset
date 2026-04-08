use allsorts::binary::read::ReadScope;
use allsorts::font::read_cmap_subtable;
use allsorts::font_data::FontData;
use allsorts::subset::{subset, CmapTarget, SubsetProfile};
use allsorts::tables::cmap::Cmap;
use allsorts::tables::FontTableProvider;
use allsorts::tag;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResult {
    pub success: bool,
    pub message: String,
    pub output_path: Option<String>,
}

/// 字体信息
#[derive(Debug, Serialize, Deserialize)]
pub struct FontInfo {
    pub index: u32,
    pub family_name: Option<String>,
    pub post_script_name: Option<String>,
    pub num_glyphs: u16,
    pub units_per_em: u16,
}

/// 从文本中提取所有唯一的字符
fn extract_unique_chars(text: &str) -> BTreeSet<char> {
    let mut chars: BTreeSet<char> = text.chars().collect();
    // 添加基本控制字符，确保字体正常工作
    chars.insert('\0'); // NULL
    chars.insert('\r'); // CR
    chars.insert('\n'); // LF
    chars.insert(' ');  // Space
    chars
}

/// 使用 allsorts 提取字体子集
fn subset_font_data(font_data: &[u8], chars: &BTreeSet<char>) -> Result<Vec<u8>, String> {
    // 读取字体数据
    let scope = ReadScope::new(font_data);
    let font_file = scope
        .read::<FontData<'_>>()
        .map_err(|e| format!("解析字体文件失败: {:?}", e))?;

    // 获取字体表提供者
    let table_provider = font_file
        .table_provider(0)
        .map_err(|e| format!("获取字体表失败: {:?}", e))?;

    // 读取 cmap 表以获取字符到字形 ID 的映射
    let cmap_data = table_provider
        .table_data(tag::CMAP)
        .map_err(|e| format!("获取 cmap 表失败: {:?}", e))?
        .ok_or_else(|| "字体缺少 cmap 表".to_string())?;

    let cmap = ReadScope::new(&cmap_data)
        .read::<Cmap<'_>>()
        .map_err(|e| format!("解析 cmap 表失败: {:?}", e))?;

    // 使用 read_cmap_subtable 选择合适的 cmap 子表
    let (_, cmap_subtable) = read_cmap_subtable(&cmap)
        .map_err(|e| format!("读取 cmap 子表失败: {:?}", e))?
        .ok_or_else(|| "没有找到合适的 cmap 子表".to_string())?;

    // 将字符转换为字形 ID
    let mut glyph_ids: BTreeSet<u16> = BTreeSet::new();
    // 必须包含 .notdef 字形 (ID 0)
    glyph_ids.insert(0);

    // 使用 cmap 子表映射字符到字形 ID
    for ch in chars {
        if let Ok(Some(glyph_id)) = cmap_subtable.map_glyph(*ch as u32) {
            glyph_ids.insert(glyph_id);
        }
    }

    let glyph_ids_vec: Vec<u16> = glyph_ids.into_iter().collect();

    println!("[Subset] 需要保留的 glyph 数量: {}", glyph_ids_vec.len());

    // 使用 SubsetProfile::Minimal 进行子集提取
    let profile = SubsetProfile::Minimal;

    // 执行子集提取，使用 CmapTarget::Unicode
    let subset_data = subset(&table_provider, &glyph_ids_vec, &profile, CmapTarget::Unicode)
        .map_err(|e| format!("子集提取失败: {:?}", e))?;

    Ok(subset_data)
}

/// 从 TTC 字体集合中提取指定索引的字体数据
/// TTC 格式: 多个字体共享表数据，通过 Offset 表指向不同位置
fn extract_font_from_ttc(font_data: &[u8], font_index: u32) -> Result<Vec<u8>, String> {
    // 使用 ttf-parser 验证索引有效
    let num_fonts = ttf_parser::fonts_in_collection(font_data)
        .ok_or_else(|| "不是有效的 TTC 字体文件".to_string())?;

    if font_index >= num_fonts {
        return Err(format!(
            "字体索引 {} 超出范围 (0-{})",
            font_index,
            num_fonts - 1
        ));
    }

    // 解析 TTC 头部获取各字体的偏移量表位置
    // TTC Header 格式:
    // - Tag: "ttcf" (4 bytes)
    // - Version: 4 bytes
    // - numFonts: 4 bytes
    // - offset[n]: 每个字体的 Offset 表偏移量 (4 bytes each)
    if font_data.len() < 12 {
        return Err("TTC 文件太小".to_string());
    }

    let tag = u32::from_be_bytes([font_data[0], font_data[1], font_data[2], font_data[3]]);
    if tag != u32::from_be_bytes(*b"ttcf") {
        return Err("不是有效的 TTC 文件".to_string());
    }

    // 读取版本号
    let _version = u32::from_be_bytes([font_data[4], font_data[5], font_data[6], font_data[7]]);

    // 读取字体数量
    let num_fonts_in_file =
        u32::from_be_bytes([font_data[8], font_data[9], font_data[10], font_data[11]]);

    if num_fonts_in_file != num_fonts {
        return Err("TTC 头部信息不一致".to_string());
    }

    // 读取指定字体的 Offset 表偏移量
    let offset_table_offset_pos = 12 + (font_index as usize) * 4;
    if font_data.len() < offset_table_offset_pos + 4 {
        return Err("TTC 文件损坏".to_string());
    }

    let offset_table_offset = u32::from_be_bytes([
        font_data[offset_table_offset_pos],
        font_data[offset_table_offset_pos + 1],
        font_data[offset_table_offset_pos + 2],
        font_data[offset_table_offset_pos + 3],
    ]) as usize;

    // 解析 Offset 表
    if font_data.len() < offset_table_offset + 12 {
        return Err("TTC Offset 表位置无效".to_string());
    }

    let sfnt_version = u32::from_be_bytes([
        font_data[offset_table_offset],
        font_data[offset_table_offset + 1],
        font_data[offset_table_offset + 2],
        font_data[offset_table_offset + 3],
    ]);

    let num_tables = u16::from_be_bytes([
        font_data[offset_table_offset + 4],
        font_data[offset_table_offset + 5],
    ]);

    let search_range = u16::from_be_bytes([
        font_data[offset_table_offset + 6],
        font_data[offset_table_offset + 7],
    ]);

    let entry_selector = u16::from_be_bytes([
        font_data[offset_table_offset + 8],
        font_data[offset_table_offset + 9],
    ]);

    let range_shift = u16::from_be_bytes([
        font_data[offset_table_offset + 10],
        font_data[offset_table_offset + 11],
    ]);

    // 读取表记录 (Table Record)
    // 每个表记录: tag (4) + checksum (4) + offset (4) + length (4) = 16 bytes
    let table_record_start = offset_table_offset + 12;
    let mut tables: Vec<(u32, u32, u32, u32)> = Vec::with_capacity(num_tables as usize);

    for i in 0..num_tables {
        let record_pos = table_record_start + (i as usize) * 16;
        if font_data.len() < record_pos + 16 {
            return Err("TTC 表记录位置无效".to_string());
        }

        let tag = u32::from_be_bytes([
            font_data[record_pos],
            font_data[record_pos + 1],
            font_data[record_pos + 2],
            font_data[record_pos + 3],
        ]);
        let checksum = u32::from_be_bytes([
            font_data[record_pos + 4],
            font_data[record_pos + 5],
            font_data[record_pos + 6],
            font_data[record_pos + 7],
        ]);
        let offset = u32::from_be_bytes([
            font_data[record_pos + 8],
            font_data[record_pos + 9],
            font_data[record_pos + 10],
            font_data[record_pos + 11],
        ]);
        let length = u32::from_be_bytes([
            font_data[record_pos + 12],
            font_data[record_pos + 13],
            font_data[record_pos + 14],
            font_data[record_pos + 15],
        ]);

        tables.push((tag, checksum, offset, length));
    }

    // 构建单个 TTF 文件
    build_ttf_from_tables(
        sfnt_version,
        search_range,
        entry_selector,
        range_shift,
        &tables,
        font_data,
    )
}

/// 从表数据构建 TTF 文件
fn build_ttf_from_tables(
    sfnt_version: u32,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
    tables: &[(u32, u32, u32, u32)],
    source_data: &[u8],
) -> Result<Vec<u8>, String> {
    let num_tables = tables.len() as u16;

    // 计算表数据起始位置
    let table_record_size = 16 * tables.len();
    let header_size = 12 + table_record_size;

    // 计算每个表的新偏移量
    let mut new_tables: Vec<(u32, u32, u32, Vec<u8>)> = Vec::with_capacity(tables.len());
    let mut current_offset = header_size as u32;

    for (tag, checksum, offset, length) in tables {
        let offset = *offset as usize;
        let length = *length as usize;

        if source_data.len() < offset + length {
            return Err(format!(
                "表数据超出文件范围: tag={:08X}, offset={}, length={}",
                tag, offset, length
            ));
        }

        let table_data = source_data[offset..offset + length].to_vec();

        // 重新计算 checksum（TTC中的原始checksum可能基于共享表计算，不适用于提取后的独立字体）
        let tag_bytes = tag.to_be_bytes();
        let tag_str = String::from_utf8_lossy(&tag_bytes);
        let calculated_checksum = calc_checksum_debug(&table_data, &tag_str);
        println!(
            "[TTC Extract] 表 {}: 原始checksum=0x{:08X}, 计算checksum=0x{:08X}, offset={}, length={}",
            tag_str, checksum, calculated_checksum, offset, length
        );

        // 4字节对齐
        let padded_offset = current_offset;
        let padded_length = ((length + 3) / 4) * 4;

        // 使用重新计算的 checksum，确保提取后的字体能通过严格验证
        new_tables.push((*tag, calculated_checksum, padded_offset, table_data));
        current_offset += padded_length as u32;
    }

    // 按 tag 排序表记录
    new_tables.sort_by_key(|(tag, _, _, _)| *tag);

    // 构建输出数据
    let total_size = current_offset as usize;
    let mut output = Vec::with_capacity(total_size);

    // 写入字体头
    output.extend_from_slice(&sfnt_version.to_be_bytes());
    output.extend_from_slice(&num_tables.to_be_bytes());
    output.extend_from_slice(&search_range.to_be_bytes());
    output.extend_from_slice(&entry_selector.to_be_bytes());
    output.extend_from_slice(&range_shift.to_be_bytes());

    // 写入表记录
    for (tag, checksum, offset, data) in &new_tables {
        output.extend_from_slice(&tag.to_be_bytes());
        output.extend_from_slice(&checksum.to_be_bytes());
        output.extend_from_slice(&offset.to_be_bytes());
        output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    }

    // 写入表数据
    for (_, _, _, data) in &new_tables {
        output.extend_from_slice(data);
        // 填充到4字节对齐
        let padding = (4 - (data.len() % 4)) % 4;
        for _ in 0..padding {
            output.push(0);
        }
    }

    // 更新 head 表的 checksumAdjustment
    update_head_checksum(&mut output)?;

    Ok(output)
}

/// 计算校验和
fn calc_checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    let mut i = 0;

    // 按 4 字节字累加
    while i + 4 <= data.len() {
        let word = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        sum = sum.wrapping_add(word);
        i += 4;
    }

    // 处理剩余字节
    if i < data.len() {
        let mut remaining = [0u8; 4];
        remaining[..data.len() - i].copy_from_slice(&data[i..]);
        let word = u32::from_be_bytes(remaining);
        sum = sum.wrapping_add(word);
    }

    sum
}

/// 计算校验和（详细日志版）
fn calc_checksum_debug(data: &[u8], tag: &str) -> u32 {
    let mut sum: u32 = 0;
    let mut i = 0;

    println!("[Checksum Debug] {}: 数据长度 = {} bytes", tag, data.len());

    // 按 4 字节字累加
    while i + 4 <= data.len() {
        let word = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        sum = sum.wrapping_add(word);
        i += 4;
    }

    // 处理剩余字节
    if i < data.len() {
        let mut remaining = [0u8; 4];
        remaining[..data.len() - i].copy_from_slice(&data[i..]);
        let word = u32::from_be_bytes(remaining);
        sum = sum.wrapping_add(word);
        println!("[Checksum Debug] {}: 有 {} 字节剩余", tag, data.len() - i + 4);
    }

    println!("[Checksum Debug] {}: checksum = 0x{:08X}", tag, sum);
    sum
}

/// 更新 head 表的校验和调整值
fn update_head_checksum(font_data: &mut [u8]) -> Result<(), String> {
    let num_tables = u16::from_be_bytes([font_data[4], font_data[5]]) as usize;
    println!("[Head Checksum] 字体表数量: {}", num_tables);

    // 首先找到 head 表的位置
    let mut head_record_offset = None;
    let mut head_table_offset = None;
    let mut head_table_length = None;

    for i in 0..num_tables {
        let record_offset = 12 + i * 16;
        let tag = u32::from_be_bytes([
            font_data[record_offset],
            font_data[record_offset + 1],
            font_data[record_offset + 2],
            font_data[record_offset + 3],
        ]);

        if tag == u32::from_be_bytes(*b"head") {
            let offset = u32::from_be_bytes([
                font_data[record_offset + 8],
                font_data[record_offset + 9],
                font_data[record_offset + 10],
                font_data[record_offset + 11],
            ]) as usize;
            let length = u32::from_be_bytes([
                font_data[record_offset + 12],
                font_data[record_offset + 13],
                font_data[record_offset + 14],
                font_data[record_offset + 15],
            ]) as usize;

            head_record_offset = Some(record_offset);
            head_table_offset = Some(offset);
            head_table_length = Some(length);
            break;
        }
    }

    let (record_offset, table_offset, table_length) = match (head_record_offset, head_table_offset, head_table_length) {
        (Some(r), Some(t), Some(l)) => (r, t, l),
        _ => {
            println!("[Head Checksum] 警告: 未找到 head 表");
            return Ok(());
        }
    };

    println!("[Head Checksum] 找到 head 表: record_offset={}, table_offset={}, length={}",
             record_offset, table_offset, table_length);

    // 读取原始的 checksumAdjustment
    let checksum_offset = table_offset + 8;
    let original_adjustment = u32::from_be_bytes([
        font_data[checksum_offset],
        font_data[checksum_offset + 1],
        font_data[checksum_offset + 2],
        font_data[checksum_offset + 3],
    ]);
    println!("[Head Checksum] 原始 checksumAdjustment: 0x{:08X}", original_adjustment);

    // 先将 checksumAdjustment 设为 0
    font_data[checksum_offset] = 0;
    font_data[checksum_offset + 1] = 0;
    font_data[checksum_offset + 2] = 0;
    font_data[checksum_offset + 3] = 0;

    // 计算整个字体的校验和
    let full_checksum = calc_checksum_debug(font_data, "full_font");
    let adjustment = 0xB1B0AFBAu32.wrapping_sub(full_checksum);
    println!("[Head Checksum] 计算得到的 adjustment: 0x{:08X}", adjustment);

    // 写回校验和调整值
    font_data[checksum_offset..checksum_offset + 4]
        .copy_from_slice(&adjustment.to_be_bytes());

    // 重新计算 head 表的 checksum（更新表记录中的 checksum）
    let head_table_data = &font_data[table_offset..table_offset + table_length];
    let new_head_checksum = calc_checksum(head_table_data);
    println!("[Head Checksum] 新的 head 表 checksum: 0x{:08X}", new_head_checksum);

    // 更新表记录中的 checksum
    font_data[record_offset + 4..record_offset + 8].copy_from_slice(&new_head_checksum.to_be_bytes());
    println!("[Head Checksum] 已更新表记录中的 checksum");

    Ok(())
}

/// 获取字体信息（支持 TTC 多字体）
#[tauri::command]
fn get_font_info(font_path: String) -> Result<serde_json::Value, String> {
    if !Path::new(&font_path).exists() {
        return Err("字体文件不存在".to_string());
    }

    let font_data = fs::read(&font_path).map_err(|e| format!("读取字体文件失败: {}", e))?;

    // 获取字体集合中的字体数量
    let num_fonts = ttf_parser::fonts_in_collection(&font_data).unwrap_or(1);

    let mut fonts_info = Vec::new();

    // 获取集合中每个字体的信息
    for font_index in 0..num_fonts {
        match ttf_parser::Face::parse(&font_data, font_index) {
            Ok(face) => {
                // 名称 ID 常量
                const FAMILY_NAME: u16 = 1;       // 字体家族名称
                const SUBFAMILY_NAME: u16 = 2;    // 字体子家族名称 (Regular, Bold 等)
                const FULL_NAME: u16 = 4;         // 完整名称
                const POST_SCRIPT_NAME: u16 = 6;  // PostScript 名称

                let family_name = face
                    .names()
                    .into_iter()
                    .find(|name| name.name_id == FAMILY_NAME)
                    .and_then(|name| name.to_string());

                let subfamily_name = face
                    .names()
                    .into_iter()
                    .find(|name| name.name_id == SUBFAMILY_NAME)
                    .and_then(|name| name.to_string());

                let full_name = face
                    .names()
                    .into_iter()
                    .find(|name| name.name_id == FULL_NAME)
                    .and_then(|name| name.to_string());

                let post_script_name = face
                    .names()
                    .into_iter()
                    .find(|name| name.name_id == POST_SCRIPT_NAME)
                    .and_then(|name| name.to_string());

                let num_glyphs = face.number_of_glyphs();
                let units_per_em = face.units_per_em();

                fonts_info.push(serde_json::json!({
                    "index": font_index,
                    "family_name": family_name,
                    "subfamily_name": subfamily_name,
                    "full_name": full_name,
                    "post_script_name": post_script_name,
                    "num_glyphs": num_glyphs,
                    "units_per_em": units_per_em,
                }));
            }
            Err(e) => {
                fonts_info.push(serde_json::json!({
                    "index": font_index,
                    "error": format!("{:?}", e),
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "is_collection": num_fonts > 1,
        "num_fonts": num_fonts,
        "fonts": fonts_info,
        "file_size": font_data.len(),
    }))
}

/// 提取字体子集（支持指定字体索引）
#[tauri::command]
fn extract_font_subset(
    font_path: String,
    text: String,
    output_path: String,
    font_index: Option<u32>,
) -> Result<ExtractResult, String> {
    // 检查输入文件是否存在
    if !Path::new(&font_path).exists() {
        return Ok(ExtractResult {
            success: false,
            message: format!("字体文件不存在: {}", font_path),
            output_path: None,
        });
    }

    // 读取字体文件
    let font_data = fs::read(&font_path).map_err(|e| format!("读取字体文件失败: {}", e))?;
    let original_size = font_data.len();

    // 提取需要的字符
    let chars = extract_unique_chars(&text);

    if chars.is_empty() {
        return Ok(ExtractResult {
            success: false,
            message: "没有需要提取的字符".to_string(),
            output_path: None,
        });
    }

    // 获取字体集合中的字体数量
    let num_fonts = ttf_parser::fonts_in_collection(&font_data).unwrap_or(1);
    let target_index = font_index.unwrap_or(0);

    if target_index >= num_fonts {
        return Ok(ExtractResult {
            success: false,
            message: format!("字体索引 {} 超出范围 (0-{})", target_index, num_fonts - 1),
            output_path: None,
        });
    }

    // 确保输出目录存在
    let output_path_obj = Path::new(&output_path);
    if let Some(parent) = output_path_obj.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    // 步骤1: 如果是 TTC，先提取指定索引的字体
    let single_font_data = if num_fonts > 1 {
        println!("[Extract] TTC 字体，共 {} 个字体，提取索引 {}", num_fonts, target_index);
        extract_font_from_ttc(&font_data, target_index)?
    } else {
        font_data
    };

    // 步骤2: 对字体进行子集提取
    println!("[Extract] 开始子集提取，字体数据大小: {} bytes", single_font_data.len());
    let subset_font_data = subset_font_data(&single_font_data, &chars)?;
    let new_size = subset_font_data.len();

    // 写入输出文件
    fs::write(&output_path, &subset_font_data)
        .map_err(|e| format!("写入输出文件失败: {}", e))?;

    // 计算压缩率
    let reduction = if original_size > 0 {
        ((original_size - new_size) as f64 / original_size as f64 * 100.0) as u32
    } else {
        0
    };

    let message = if num_fonts > 1 {
        format!(
            "字体提取成功！从 TTC 中提取第 {} 个字体，原始大小: {} KB, 新大小: {} KB, 减少: {}%",
            target_index,
            original_size / 1024,
            new_size / 1024,
            reduction
        )
    } else {
        format!(
            "字体提取成功！原始大小: {} KB, 新大小: {} KB, 减少: {}%",
            original_size / 1024,
            new_size / 1024,
            reduction
        )
    };

    Ok(ExtractResult {
        success: true,
        message,
        output_path: Some(output_path),
    })
}

/// 获取用于预览的字体数据
#[tauri::command]
fn get_font_data_for_preview(
    font_path: String,
    font_index: Option<u32>,
) -> Result<Vec<u8>, String> {
    if !Path::new(&font_path).exists() {
        return Err("字体文件不存在".to_string());
    }

    let font_data = fs::read(&font_path).map_err(|e| format!("读取字体文件失败: {}", e))?;

    // 获取字体集合中的字体数量
    let num_fonts = ttf_parser::fonts_in_collection(&font_data).unwrap_or(1);
    let target_index = font_index.unwrap_or(0);

    // 如果不是 TTC 字体，直接返回原始数据
    if num_fonts <= 1 {
        return Ok(font_data);
    }

    // 对于 TTC 字体，提取指定索引的字体
    extract_font_from_ttc(&font_data, target_index)
        .map_err(|e| format!("提取预览字体失败: {}", e))
}

/// 前端调用此命令在终端打印日志
#[tauri::command]
fn log_to_terminal(message: String) {
    println!("[Frontend] {}", message);
}

/// 打开文件夹
#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    let path_obj = std::path::Path::new(&path);
    if !path_obj.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let folder_path = if path_obj.is_file() {
        path_obj
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path_obj.to_path_buf())
    } else {
        path_obj.to_path_buf()
    };

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            extract_font_subset,
            get_font_info,
            get_font_data_for_preview,
            log_to_terminal,
            open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
