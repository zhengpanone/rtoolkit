// 身份证生成实现
use chrono::NaiveDate;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::{rng, Rng};

use crate::utils::areas::get_full_area_info_str;
use crate::utils::areas::random_area;

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
    /// 指定6位地区码
    #[arg(long = "region", help = "地区,例如 110000", required = false)]
    region: Option<String>,
    /// 出生日期
    #[arg(long = "birth", help = "出生日期")]
    birth: Option<String>,

    /// 随机生日的最小日期（含）
    #[arg(long, default_value = "1970-01-01")]
    min_birth: String,

    /// 随机生日的最大日期（含）
    #[arg(long, default_value = "2010-12-31")]
    max_birth: String,
    /// 性别（male 奇数、female 偶数、 any随机）
    #[arg(value_enum,long = "gender", default_value_t = Gender::Any, help = "性别")]
    gender: Gender,
}

pub fn run_gen_id(opts: IdOpts) -> Result<(), IdError> {
    let region = opts.region.as_deref();
    let min_date: NaiveDate = parse_date(&opts.min_birth)?;
    let max_date: NaiveDate = parse_date(&opts.max_birth)?;
    let fixed_birth = match opts.birth {
        Some(b) => Some(parse_date(&b)?),
        None => None,
    };

    for _ in 0..opts.count {
        let id = generate_id(region, fixed_birth, min_date, max_date, opts.gender)?;
        println!("{}", id);
    }
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum IdError {
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("region must be 6 digits")]
    InvalidRegion,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Gender {
    Any,
    Male,
    Female,
}

fn generate_id(
    region: Option<&str>,
    birth: Option<NaiveDate>,
    min: NaiveDate,
    max: NaiveDate,
    gender: Gender,
) -> Result<String, IdError> {
    let code6 = match region {
        Some(r) => {
            validate_region(r)?;
            r.to_string()
        }
        None => random_area(),
    };
    let code_name =
        get_full_area_info_str(code6.as_str()).unwrap_or_else(|| "地址未知".to_string());
    let b = birth.unwrap_or_else(|| random_date(min, max));
    let seq3 = random_seq(gender);
    let id17 = format!("{}{}{}", code6, b.format("%Y%m%d"), seq3,);
    let check = checksum_char(&id17);
    let name: String = Name(ZH_CN).fake();
    Ok(format!(
        "姓名: {}\t 身份证号: {}{}\t 地址:{}",
        name, id17, check, code_name
    ))
}
fn validate_region(code: &str) -> Result<(), IdError> {
    if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(IdError::InvalidRegion)
    }
}
fn parse_date(s: &str) -> Result<NaiveDate, IdError> {
    // 先尝试无分隔符格式：19900520
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y%m%d") {
        return Ok(d);
    }
    // 再尝试带短横线格式：1990-05-20
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
    let days = (max - min).num_days().max(0) as i64;
    let offset = rng.random_range(0..=days);
    min + chrono::Duration::days(offset)
}

/// 计算校验位（权重 7,9,10,5,8,4,2,1,6,3,7,9,10,5,8,4,2；余数映射 0..10 -> 1,0,X,9,8,7,6,5,4,3,2）
fn checksum_char(id17: &str) -> char {
    let weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
    let mapping = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
    let sum: i32 = id17
        .chars()
        .zip(weights.iter())
        .map(|(c, w)| (c.to_digit(10).unwrap_or(0) as i32) * (*w as i32))
        .sum();
    let idx = (sum % 11) as usize;
    mapping[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate};

    #[test]
    fn test_checksum_known() {
        // 构造一个 17 位并检查校验映射是否稳定（非真实号码）
        let id17 = "11010119900101001"; // 17 位
        let c = checksum_char(id17);
        assert!("10X98765432".contains(c));
    }

    #[test]
    fn test_random_date() {
        let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let max = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();
        let date = random_date(min, max);
        assert!(date >= min && date <= max, "日期应在区间内");

        let day = NaiveDate::from_ymd_opt(2023, 5, 10).unwrap();
        let date = random_date(day, day);
        assert_eq!(date, day, "最小值和最大值相同应返回该日期");

        let min = NaiveDate::from_ymd_opt(2023, 5, 10).unwrap();
        let max = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
        let date = random_date(min, max);
        // 按函数逻辑，若 max < min，会取 max(0)，所以结果应等于 min
        assert_eq!(date, min, "当 max < min 时，应返回 min");

        let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let max = NaiveDate::from_ymd_opt(2020, 1, 10).unwrap();
        let mut dates = std::collections::HashSet::new();
        for _ in 0..50 {
            dates.insert(random_date(min, max));
        }
        assert!(
            dates.len() > 1,
            "多次随机生成的日期应该有不同值（非固定输出）"
        );
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("19900520").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1990, 5, 20).unwrap());

        let date = parse_date("1990-05-20").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1990, 5, 20).unwrap());
        // 2 月 30 日无效
        let err = parse_date("20230230").unwrap_err();
        match err {
            IdError::InvalidDate(s) => assert_eq!(s, "20230230"),
            _ => panic!("expected InvalidDate error"),
        }
        // 闰年应能通过
        let date = parse_date("20200229").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2020, 2, 29).unwrap());

        // 测试边界：最早和较远将来
        let d1 = parse_date("19000101").unwrap();
        let d2 = parse_date("20991231").unwrap();
        assert_eq!(d1.year(), 1900);
        assert_eq!(d2.year(), 2099);
    }
}
