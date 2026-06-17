use chrono::NaiveDate;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::{rng, Rng};

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

use crate::utils::areas::get_full_area_info_str;
use crate::utils::areas::{random_area, random_region_by_code};

pub const MAX_IDGEN_COUNT: u32 = 10_000_000;
pub const MAX_EXCEL_ROWS: u32 = 1_048_575;

#[derive(clap::Args)]
pub struct IdOpts {
    #[arg(
        short = 'n',
        long,
        default_value_t = 1,
        help = "生成数量",
        long_help = "生成身份证号数量"
    )]
    count: u32,

    #[arg(long = "region", help = "地区, 例如 110101", required = false)]
    region: Option<String>,

    #[arg(long = "birth", help = "出生日期")]
    birth: Option<String>,

    #[arg(long, default_value = "1970-01-01")]
    min_birth: String,

    #[arg(long, default_value = "2010-12-31")]
    max_birth: String,

    #[arg(value_enum, short = 'g', long = "gender", default_value_t = Gender::Any, help = "性别")]
    gender: Gender,

    #[arg(short = 'o', long = "output.csv", help = "输出文件")]
    output: Option<String>,

    #[arg(value_enum, short = 't', long = "type", default_value_t = OutputType::Text, help = "输出类型")]
    output_type: OutputType,
}

pub fn run_gen_id(opts: IdOpts) -> Result<(), IdError> {
    let records = generate_ids(IdGenerateRequest {
        count: Some(opts.count),
        region: opts.region,
        birth: opts.birth,
        min_birth: Some(opts.min_birth),
        max_birth: Some(opts.max_birth),
        gender: Some(opts.gender),
    })?;

    // 根据输出类型输出不同格式
    if let Some(output) = &opts.output {
        write_to_file(&records, output, &opts.output_type)?;
    } else {
        print_console(&records);
    }
    Ok(())
}

fn print_console(records: &[IdRecord]) {
    for record in records {
        println!(
            "姓名: {}\t 性别: {}\t 身份证号: {}\t 地址:{}",
            record.name,
            convert_gender_name(&record.gender),
            record.id_number,
            record.address
        );
    }
}

fn write_to_file(
    records: &[IdRecord],
    path: &str,
    output_type: &OutputType,
) -> Result<(), IdError> {
    let mut file = File::create(path)?;
    match output_type {
        OutputType::Text => {
            writeln!(file, "姓名\t性别\t身份证号\t地址")?;
            for record in records {
                writeln!(
                    file,
                    "{}",
                    format!(
                        "{}\t{}\t{}\t{}",
                        record.name,
                        convert_gender_name(&record.gender),
                        record.id_number,
                        record.address
                    )
                )?
            }
        }
        OutputType::Csv => {
            writeln!(file, "姓名,性别,身份证号,地址")?;
            for record in records {
                writeln!(
                    file,
                    "{},{},{},{}",
                    record.name,
                    convert_gender_name(&record.gender),
                    record.id_number,
                    record.address
                )?;
            }
        }
        OutputType::Json => {
            let json = serde_json::to_string_pretty(records)?;
            file.write_all(json.as_bytes())?;
        }
        OutputType::Excel => {
            write_excel_file(records, path)?;
        }
    }
    Ok(())
}

fn write_excel_file(records: &[IdRecord], path: &str) -> Result<(), IdError> {
    write_styled_excel_file(records, path)
}

fn write_styled_excel_file(records: &[IdRecord], path: &str) -> Result<(), IdError> {
    use chrono::Local;
    use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    let border_color = Color::RGB(0xC8D3E1);

    let title_format = Format::new()
        .set_bold()
        .set_font_name("Microsoft YaHei")
        .set_font_size(18)
        .set_font_color(Color::RGB(0x002F6C))
        .set_background_color(Color::RGB(0xEAF4FB))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let meta_format = Format::new()
        .set_font_name("Microsoft YaHei")
        .set_font_size(12)
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter);
    let header_format = Format::new()
        .set_bold()
        .set_font_name("Microsoft YaHei")
        .set_font_size(12)
        .set_background_color(Color::RGB(0x1F4E79))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin)
        .set_border_color(border_color)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let center_format = Format::new()
        .set_font_name("Microsoft YaHei")
        .set_font_size(11)
        .set_border(FormatBorder::Thin)
        .set_border_color(border_color)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let id_format = Format::new()
        .set_font_name("Consolas")
        .set_font_size(11)
        .set_font_color(Color::RGB(0x0000FF))
        .set_num_format("@")
        .set_border(FormatBorder::Thin)
        .set_border_color(border_color)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let address_format = Format::new()
        .set_font_name("Microsoft YaHei")
        .set_font_size(11)
        .set_border(FormatBorder::Thin)
        .set_border_color(border_color)
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter);

    sheet.set_column_width(0, 14)?;
    sheet.set_column_width(1, 28)?;
    sheet.set_column_width(2, 16)?;
    sheet.set_column_width(3, 12)?;
    sheet.set_column_width(4, 46)?;
    sheet.set_row_height(0, 32)?;
    sheet.set_row_height(1, 24)?;
    sheet.set_row_height(2, 28)?;

    sheet.merge_range(0, 0, 0, 4, "身份证生成结果", &title_format)?;
    let generated_at = Local::now().format("%Y/%-m/%-d %H:%M:%S");
    let meta_text = format!("生成时间： {}    记录数： {}", generated_at, records.len());
    sheet.merge_range(
        1,
        0,
        1,
        4,
        &meta_text,
        &meta_format,
    )?;

    let headers = ["姓名", "身份证号", "生日", "性别", "地址"];
    for (col, header) in headers.iter().enumerate() {
        sheet.write_string_with_format(2, col as u16, *header, &header_format)?;
    }

    for (index, record) in records.iter().enumerate() {
        let row = (index + 3) as u32;
        sheet.set_row_height(row, 24)?;
        sheet.write_string_with_format(row, 0, &record.name, &center_format)?;
        sheet.write_string_with_format(row, 1, &record.id_number, &id_format)?;
        sheet.write_string_with_format(row, 2, &record.birthday, &center_format)?;
        sheet.write_string_with_format(row, 3, convert_gender_name(&record.gender), &center_format)?;
        sheet.write_string_with_format(row, 4, &record.address, &address_format)?;
    }

    sheet.set_freeze_panes(3, 0)?;
    sheet.autofilter(2, 0, records.len() as u32 + 2, 4)?;
    workbook.save(path)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum IdError {
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("region must be 2, 4, or 6 digits")]
    InvalidRegion,
    #[error("excel export supports at most 1048575 records")]
    ExcelRowLimit,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Excel(#[from] rust_xlsxwriter::XlsxError),
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Any,
    Male,
    Female,
}

fn convert_gender_name(gender: &str) -> &'static str {
    if gender.eq("male") {
        "男"
    } else {
        "女"
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    Text,
    Csv,
    Json,
    Excel,
}

#[derive(Debug, Deserialize)]
pub struct IdGenerateRequest {
    pub count: Option<u32>,
    pub region: Option<String>,
    pub birth: Option<String>,
    pub min_birth: Option<String>,
    pub max_birth: Option<String>,
    pub gender: Option<Gender>,
}

#[derive(Debug, Serialize)]
pub struct IdRecord {
    pub name: String,
    pub id_number: String,
    pub region: String,
    pub birthday: String,
    pub gender: String,
    pub address: String,
}

pub fn generate_ids(request: IdGenerateRequest) -> Result<Vec<IdRecord>, IdError> {
    let count = request.count.unwrap_or(1).clamp(1, MAX_IDGEN_COUNT);
    let min_birth = request
        .min_birth
        .unwrap_or_else(|| "1970-01-01".to_string());
    let max_birth = request
        .max_birth
        .unwrap_or_else(|| "2010-12-31".to_string());
    let min_date = parse_date(&min_birth)?;
    let max_date = parse_date(&max_birth)?;
    let fixed_birth = match request.birth {
        Some(b) if !b.trim().is_empty() => Some(parse_date(&b)?),
        _ => None,
    };
    let region = request
        .region
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let gender = request.gender.unwrap_or(Gender::Any);

    let mut records = Vec::with_capacity(count as usize);
    for _ in 0..count {
        records.push(generate_id(
            region,
            fixed_birth,
            min_date,
            max_date,
            gender,
        )?);
    }

    Ok(records)
}

pub fn write_generated_ids<W: Write>(
    request: IdGenerateRequest,
    output_type: OutputType,
    mut writer: W,
) -> Result<u32, IdError> {
    validate_id_download_request(&request, output_type)?;

    let count = request.count.unwrap_or(1).clamp(1, MAX_IDGEN_COUNT);
    let min_birth = request
        .min_birth
        .unwrap_or_else(|| "1970-01-01".to_string());
    let max_birth = request
        .max_birth
        .unwrap_or_else(|| "2010-12-31".to_string());
    let min_date = parse_date(&min_birth)?;
    let max_date = parse_date(&max_birth)?;
    let fixed_birth = match request.birth {
        Some(b) if !b.trim().is_empty() => Some(parse_date(&b)?),
        _ => None,
    };
    let region = request
        .region
        .filter(|value| !value.trim().is_empty());
    let gender = request.gender.unwrap_or(Gender::Any);

    match output_type {
        OutputType::Text => {
            writeln!(writer, "姓名\t性别\t身份证号\t生日\t地址")?;
            for _ in 0..count {
                let record = generate_id(
                    region.as_deref(),
                    fixed_birth,
                    min_date,
                    max_date,
                    gender,
                )?;
                writeln!(
                    writer,
                    "{}\t{}\t{}\t{}\t{}",
                    record.name,
                    convert_gender_name(&record.gender),
                    record.id_number,
                    record.birthday,
                    record.address
                )?;
            }
        }
        OutputType::Csv => {
            writer.write_all(b"\xEF\xBB\xBF")?;
            writeln!(writer, "姓名,性别,身份证号,生日,地址")?;
            for _ in 0..count {
                let record = generate_id(
                    region.as_deref(),
                    fixed_birth,
                    min_date,
                    max_date,
                    gender,
                )?;
                writeln!(
                    writer,
                    "{},{},{},{},{}",
                    csv_cell(&record.name),
                    csv_cell(convert_gender_name(&record.gender)),
                    csv_cell(&record.id_number),
                    csv_cell(&record.birthday),
                    csv_cell(&record.address)
                )?;
            }
        }
        OutputType::Json => {
            writer.write_all(b"[\n")?;
            for index in 0..count {
                let record = generate_id(
                    region.as_deref(),
                    fixed_birth,
                    min_date,
                    max_date,
                    gender,
                )?;
                if index > 0 {
                    writer.write_all(b",\n")?;
                }
                serde_json::to_writer_pretty(&mut writer, &record)?;
            }
            writer.write_all(b"\n]\n")?;
        }
        OutputType::Excel => {
            let mut records = Vec::with_capacity(count as usize);
            for _ in 0..count {
                records.push(generate_id(
                    region.as_deref(),
                    fixed_birth,
                    min_date,
                    max_date,
                    gender,
                )?);
            }
            let path = std::env::temp_dir().join(format!("rtoolkit-idgen-{}.xlsx", rng().random::<u64>()));
            write_excel_file(&records, path.to_string_lossy().as_ref())?;
            let bytes = std::fs::read(&path)?;
            let _ = std::fs::remove_file(path);
            writer.write_all(&bytes)?;
        }
    }

    Ok(count)
}

pub fn validate_id_download_request(
    request: &IdGenerateRequest,
    output_type: OutputType,
) -> Result<(), IdError> {
    let count = request.count.unwrap_or(1).clamp(1, MAX_IDGEN_COUNT);
    if matches!(output_type, OutputType::Excel) && count > MAX_EXCEL_ROWS {
        return Err(IdError::ExcelRowLimit);
    }

    let min_birth = request
        .min_birth
        .as_deref()
        .unwrap_or("1970-01-01");
    let max_birth = request
        .max_birth
        .as_deref()
        .unwrap_or("2010-12-31");
    parse_date(min_birth)?;
    parse_date(max_birth)?;

    if let Some(birth) = request.birth.as_deref().filter(|value| !value.trim().is_empty()) {
        parse_date(birth)?;
    }

    if let Some(region) = request.region.as_deref().filter(|value| !value.trim().is_empty()) {
        validate_region(region)?;
        random_region_by_code(region).ok_or(IdError::InvalidRegion)?;
    }

    Ok(())
}

fn generate_id(
    region: Option<&str>,
    birth: Option<NaiveDate>,
    min: NaiveDate,
    max: NaiveDate,
    gender: Gender,
) -> Result<IdRecord, IdError> {
    let code6 = match region {
        Some(r) => {
            validate_region(r)?;
            random_region_by_code(r).ok_or(IdError::InvalidRegion)?
        }
        None => random_area(),
    };
    let address = get_full_area_info_str(code6.as_str()).unwrap_or_else(|| "地址未知".to_string());
    let birthday = birth.unwrap_or_else(|| random_date(min, max));
    let seq3 = random_seq(gender);
    let id17 = format!("{}{}{}", code6, birthday.format("%Y%m%d"), seq3);
    let check = checksum_char(&id17);
    let name: String = Name(ZH_CN).fake();

    Ok(IdRecord {
        name,
        id_number: format!("{}{}", id17, check),
        region: code6,
        birthday: birthday.format("%Y-%m-%d").to_string(),
        gender: if seq3
            .chars()
            .last()
            .and_then(|c| c.to_digit(10))
            .unwrap_or(0)
            % 2
            == 0
        {
            "female".to_string()
        } else {
            "male".to_string()
        },
        address,
    })
}

fn validate_region(code: &str) -> Result<(), IdError> {
    if matches!(code.len(), 2 | 4 | 6) && code.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(IdError::InvalidRegion)
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn parse_date(s: &str) -> Result<NaiveDate, IdError> {
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y%m%d") {
        return Ok(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d);
    }
    Err(IdError::InvalidDate(s.to_string()))
}

fn random_seq(gender: Gender) -> String {
    let mut rng = rng();
    let mut n = rng.random_range(0..=999);
    match gender {
        Gender::Male => {
            if n % 2 == 0 {
                n = (n + 1) % 1000;
            }
        }
        Gender::Female => {
            if n % 2 == 1 {
                n = (n + 1) % 1000;
            }
        }
        Gender::Any => {}
    }
    format!("{:03}", n)
}

fn random_date(min: NaiveDate, max: NaiveDate) -> NaiveDate {
    let mut rng = rng();
    let days = (max - min).num_days().max(0);
    let offset = rng.random_range(0..=days);
    min + chrono::Duration::days(offset)
}

fn checksum_char(id17: &str) -> char {
    let weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
    let mapping = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
    let sum: i32 = id17
        .chars()
        .zip(weights.iter())
        .map(|(c, w)| (c.to_digit(10).unwrap_or(0) as i32) * (*w as i32))
        .sum();
    mapping[(sum % 11) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate};

    #[test]
    fn test_checksum_known() {
        let id17 = "11010119900101001";
        let c = checksum_char(id17);
        assert!("10X98765432".contains(c));
    }

    #[test]
    fn test_random_date() {
        let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let max = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();
        let date = random_date(min, max);
        assert!(date >= min && date <= max);

        let day = NaiveDate::from_ymd_opt(2023, 5, 10).unwrap();
        let date = random_date(day, day);
        assert_eq!(date, day);

        let min = NaiveDate::from_ymd_opt(2023, 5, 10).unwrap();
        let max = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
        let date = random_date(min, max);
        assert_eq!(date, min);

        let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let max = NaiveDate::from_ymd_opt(2020, 1, 10).unwrap();
        let mut dates = std::collections::HashSet::new();
        for _ in 0..50 {
            dates.insert(random_date(min, max));
        }
        assert!(dates.len() > 1);
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("19900520").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1990, 5, 20).unwrap());

        let date = parse_date("1990-05-20").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1990, 5, 20).unwrap());

        let err = parse_date("20230230").unwrap_err();
        match err {
            IdError::InvalidDate(s) => assert_eq!(s, "20230230"),
            _ => panic!("expected InvalidDate error"),
        }

        let date = parse_date("20200229").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2020, 2, 29).unwrap());

        let d1 = parse_date("19000101").unwrap();
        let d2 = parse_date("20991231").unwrap();
        assert_eq!(d1.year(), 1900);
        assert_eq!(d2.year(), 2099);
    }
}
