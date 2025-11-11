use std::{collections::HashMap,  sync::OnceLock};

use rand::{rng, Rng};
use serde::Deserialize;
use std::error::Error;

// 在编译时嵌入CSV文件内容
const PROVINCES_CSV: &str = include_str!("../../data/provinces.csv");
const CITIES_CSV: &str = include_str!("../../data/cities.csv");
const AREAS_CSV: &str = include_str!("../../data/areas.csv");

// 区
#[derive(Debug, Clone, Deserialize)]
pub struct Area {
    // code,name,cityCode,provinceCode
    pub code: String,
    pub name: String,
    #[serde(rename = "cityCode")]
    pub city_code: String,
    #[serde(rename = "provinceCode")]
    pub province_code: String,
}

// 市
#[derive(Debug, Clone, Deserialize)]
pub struct City {
    // code,name,provinceCode
    pub code: String,
    pub name: String,
    #[serde(rename = "provinceCode")]
    pub province_code: String,
}

// 省
#[derive(Debug, Clone, Deserialize)]
pub struct Province {
    // code,name,provinceCode
    pub code: String,
    pub name: String,
}

// 通用的区域类型枚举
#[derive(Debug, Clone)]
pub enum AreaType {
    Province(Province),
    City(City),
    Region(Area),
}

// 统一的数据缓存结构
#[derive(Debug)]
pub struct RegionCache {
    provinces: Vec<Province>,
    cities: Vec<City>,
    areas: Vec<Area>,
    // 快速查找索引
    province_map: HashMap<String, Province>,
    city_map: HashMap<String, City>,
    area_map: HashMap<String, Area>,
    // 关联索引
    cities_by_province: HashMap<String, Vec<City>>,
    areas_by_city: HashMap<String, Vec<Area>>,
    areas_by_province: HashMap<String, Vec<Area>>,
}

// 全局区域数据缓存
static REGION_CACHE: OnceLock<RegionCache> = OnceLock::new();

impl RegionCache {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let provinces = Self::load_provinces()?;
        let cities = Self::load_cities()?;
        let areas = Self::load_areas()?;

        // 构建索引
        let province_map: HashMap<_, _> = provinces
            .iter()
            .map(|p| (p.code.clone(), p.clone()))
            .collect();

        let city_map: HashMap<_, _> = cities.iter().map(|c| (c.code.clone(), c.clone())).collect();

        let area_map: HashMap<_, _> = areas.iter().map(|r| (r.code.clone(), r.clone())).collect();

        // 构建关联索引
        let mut cities_by_province: HashMap<String, Vec<City>> = HashMap::new();
        for city in &cities {
            cities_by_province
                .entry(city.province_code.clone())
                .or_insert_with(Vec::new)
                .push(city.clone());
        }

        let mut areas_by_city: HashMap<String, Vec<Area>> = HashMap::new();
        let mut areas_by_province: HashMap<String, Vec<Area>> = HashMap::new();
        for region in &areas {
            areas_by_city
                .entry(region.city_code.clone())
                .or_insert_with(Vec::new)
                .push(region.clone());

            areas_by_province
                .entry(region.province_code.clone())
                .or_insert_with(Vec::new)
                .push(region.clone());
        }

        Ok(Self {
            provinces,
            cities,
            areas,
            province_map,
            city_map,
            area_map,
            cities_by_province,
            areas_by_city,
            areas_by_province,
        })
    }

    fn load_provinces() -> Result<Vec<Province>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_reader(PROVINCES_CSV.as_bytes());
        let mut provinces = Vec::new();

        for result in rdr.deserialize() {
            let province: Province = result?;
            provinces.push(province);
        }

        Ok(provinces)
    }

    fn load_cities() -> Result<Vec<City>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_reader(CITIES_CSV.as_bytes());
        let mut cities = Vec::new();

        for result in rdr.deserialize() {
            let city: City = result?;
            cities.push(city);
        }

        Ok(cities)
    }

    fn load_areas() -> Result<Vec<Area>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_reader(AREAS_CSV.as_bytes());
        let mut areas = Vec::new();

        for result in rdr.deserialize() {
            let area: Area = result?;
            areas.push(area);
        }

        Ok(areas)
    }

    // 获取所有省份
    pub fn get_provinces(&self) -> &[Province] {
        &self.provinces
    }

    // 获取所有城市
    pub fn get_cities(&self) -> &[City] {
        &self.cities
    }

    // 获取所有区域
    pub fn get_areas(&self) -> &[Area] {
        &self.areas
    }

    // 根据代码获取省份
    pub fn get_province(&self, code: &str) -> Option<&Province> {
        self.province_map.get(code)
    }

    // 根据代码获取城市
    pub fn get_city(&self, code: &str) -> Option<&City> {
        self.city_map.get(code)
    }

    // 根据代码获取区域
    pub fn get_region(&self, code: &str) -> Option<&Area> {
        self.area_map.get(code)
    }

    // 获取省份下的所有城市
    pub fn get_cities_by_province(&self, province_code: &str) -> &[City] {
        self.cities_by_province
            .get(province_code)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    // 获取城市下的所有区域
    pub fn get_regions_by_city(&self, city_code: &str) -> &[Area] {
        self.areas_by_city
            .get(city_code)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    // 获取省份下的所有区域
    pub fn get_regions_by_province(&self, province_code: &str) -> &[Area] {
        self.areas_by_province
            .get(province_code)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    // 随机获取省份
    pub fn random_province(&self) -> Option<&Province> {
        if self.provinces.is_empty() {
            return None;
        }
        let mut rng = rng();
        let index = rng.random_range(0..self.provinces.len());
        Some(&self.provinces[index])
    }

    // 随机获取城市
    pub fn random_city(&self) -> Option<&City> {
        if self.cities.is_empty() {
            return None;
        }
        let mut rng = rng();
        let index = rng.random_range(0..self.cities.len());
        Some(&self.cities[index])
    }

    // 随机获取区域
    pub fn random_region(&self) -> Option<&Area> {
        if self.areas.is_empty() {
            return None;
        }
        let mut rng = rng();
        let index = rng.random_range(0..self.areas.len());
        Some(&self.areas[index])
    }

    // 在指定省份内随机获取城市
    pub fn random_city_in_province(&self, province_code: &str) -> Option<&City> {
        let cities = self.get_cities_by_province(province_code);
        if cities.is_empty() {
            return None;
        }
        let mut rng = rng();
        let index = rng.random_range(0..cities.len());
        Some(&cities[index])
    }

    // 在指定城市内随机获取区域
    pub fn random_region_in_city(&self, city_code: &str) -> Option<&Area> {
        let regions = self.get_regions_by_city(city_code);
        if regions.is_empty() {
            return None;
        }
        let mut rng = rng();
        let index = rng.random_range(0..regions.len());
        Some(&regions[index])
    }

    // 在指定省份内随机获取区域
    pub fn random_region_in_province(&self, province_code: &str) -> Option<&Area> {
        let regions = self.get_regions_by_province(province_code);
        if regions.is_empty() {
            return None;
        }
        let mut rng = rng();
        let index = rng.random_range(0..regions.len());
        Some(&regions[index])
    }

    // 获取完整的区域链（省-市-区）
    pub fn get_full_area_chain(&self, region_code: &str) -> Option<(&Province, &City, &Area)> {
        let region = self.get_region(region_code)?;
        let city = self.get_city(&region.city_code)?;
        let province = self.get_province(&region.province_code)?;
        Some((province, city, region))
    }
}

// 初始化全局缓存
fn init_region_cache() -> Result<RegionCache, Box<dyn Error>> {
    RegionCache::new()
}
// 获取全局缓存
fn get_area_cache() -> &'static RegionCache {
    REGION_CACHE.get_or_init(|| {
        init_region_cache().unwrap_or_else(|e| {
            eprintln!("Failed to load area data: {}", e);
            // 创建空的缓存作为后备
            RegionCache {
                provinces: vec![],
                cities: vec![],
                areas: vec![],
                province_map: HashMap::new(),
                city_map: HashMap::new(),
                area_map: HashMap::new(),
                cities_by_province: HashMap::new(),
                areas_by_city: HashMap::new(),
                areas_by_province: HashMap::new(),
            }
        })
    })
}

// 公共API函数

// 随机获取区域代码
pub fn random_area() -> String {
    get_area_cache()
        .random_region()
        .map(|r| r.code.clone())
        .unwrap_or_else(|| "110101".to_string())
}

// 随机获取省份
pub fn random_province() -> Option<Province> {
    get_area_cache().random_province().cloned()
}

// 随机获取城市
pub fn random_city() -> Option<City> {
    get_area_cache().random_city().cloned()
}

// 随机获取区域完整信息
pub fn random_region_full() -> Area {
    get_area_cache()
        .random_region()
        .cloned()
        .unwrap_or_else(|| Area {
            code: "110101".to_string(),
            name: "东城区".to_string(),
            city_code: "1101".to_string(),
            province_code: "11".to_string(),
        })
}

// 根据代码查找区域名称
pub fn get_region_name(code: &str) -> Option<String> {
    get_area_cache().get_region(code).map(|r| r.name.clone())
}

// 获取所有区域数量
pub fn region_count() -> usize {
    get_area_cache().areas.len()
}

// 获取完整的区域链信息
pub fn get_full_area_info(region_code: &str) -> Option<(Province, City, Area)> {
    get_area_cache()
        .get_full_area_chain(region_code)
        .map(|(p, c, r)| (p.clone(), c.clone(), r.clone()))
}

// 获取完整的区域链信息
pub fn get_full_area_info_str(region_code: &str) -> Option<String> {
    get_area_cache()
        .get_full_area_chain(region_code)
        .map(|(p, c, r)| format!("{}{}{}", p.name, c.name, r.name))
}

// 根据省份代码获取所有城市
pub fn get_cities_by_province(province_code: &str) -> Vec<City> {
    get_area_cache()
        .get_cities_by_province(province_code)
        .to_vec()
}

// 根据城市代码获取所有区域
pub fn get_regions_by_city(city_code: &str) -> Vec<Area> {
    get_area_cache().get_regions_by_city(city_code).to_vec()
}
