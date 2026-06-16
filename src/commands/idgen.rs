use chrono::NaiveDate;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};

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
    #[arg(long = "region", help = "地区, 例如 110101", required = false)]
    region: Option<String>,
    #[arg(long = "birth", help = "出生日期")]
    birth: Option<String>,
    #[arg(long, default_value = "1970-01-01")]
    min_birth: String,
    #[arg(long, default_value = "2010-12-31")]
    max_birth: String,
    #[arg(value_enum, long = "gender", default_value_t = Gender::Any, help = "性别")]
    gender: Gender,
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

    for record in records {
        println!(
            "姓名: {}\t 身份证号: {}\t 地址:{}",
            record.name, record.id_number, record.address
        );
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

#[derive(Debug, Clone, Copy, clap::ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Any,
    Male,
    Female,
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
    let count = request.count.unwrap_or(1).clamp(1, 200);
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
    let region = request.region.as_deref().filter(|value| !value.trim().is_empty());
    let gender = request.gender.unwrap_or(Gender::Any);

    let mut records = Vec::with_capacity(count as usize);
    for _ in 0..count {
        records.push(generate_id(region, fixed_birth, min_date, max_date, gender)?);
    }

    Ok(records)
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
            r.to_string()
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
    if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(IdError::InvalidRegion)
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
