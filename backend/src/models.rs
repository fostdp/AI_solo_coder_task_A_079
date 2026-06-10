use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct City {
    pub id: i32,
    pub name: String,
    pub name_cn: Option<String>,
    pub longitude: f64,
    pub latitude: f64,
    pub founded_year: i32,
    pub prosperity_start: i32,
    pub prosperity_end: i32,
    pub decline_year: Option<i32>,
    pub decline_reason: Option<String>,
    pub region: Option<String>,
    pub description: Option<String>,
    pub population_peak: Option<i32>,
    pub trade_volume: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClimateData {
    pub id: i32,
    pub period_start: i32,
    pub period_end: i32,
    pub region: String,
    pub temperature_anomaly: Option<f64>,
    pub precipitation_index: Option<f64>,
    pub glacier_advance: Option<bool>,
    pub glacier_note: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TradeConnection {
    pub id: i32,
    pub city_from: i32,
    pub city_to: i32,
    pub period_start: i32,
    pub period_end: i32,
    pub trade_volume: Option<i32>,
    pub route_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityTimeline {
    pub city: City,
    pub climate_records: Vec<ClimateData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoxResult {
    pub variable: String,
    pub coefficient: f64,
    pub hazard_ratio: f64,
    pub std_error: f64,
    pub p_value: f64,
    pub confidence_interval_lower: f64,
    pub confidence_interval_upper: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRouteShift {
    pub period: String,
    pub centroid_longitude: f64,
    pub centroid_latitude: f64,
    pub active_cities: i32,
    pub total_trade_volume: i64,
    pub shift_from_previous: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub city_id: i32,
    pub city_name: String,
    pub degree_centrality: f64,
    pub betweenness_centrality: f64,
    pub eigenvector_centrality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteArrow {
    pub from_lon: f64,
    pub from_lat: f64,
    pub to_lon: f64,
    pub to_lat: f64,
    pub period_start: i32,
    pub period_end: i32,
    pub trade_volume: i32,
    pub route_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearQuery {
    pub year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityStatus {
    pub city: City,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionClimateSummary {
    pub region: String,
    pub avg_temperature_anomaly: f64,
    pub avg_precipitation_index: f64,
    pub glacier_advance_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeVaryingInterval {
    pub city_id: i32,
    pub start: i32,
    pub stop: i32,
    pub event: i32,
    pub temp_anomaly: f64,
    pub precip_index: f64,
    pub temp_change: f64,
    pub precip_change: f64,
    pub route_change: f64,
    pub glacier_advance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub cox_results: Vec<CoxResult>,
    pub sample_size: i32,
    pub log_likelihood: f64,
    pub concordance: f64,
}
