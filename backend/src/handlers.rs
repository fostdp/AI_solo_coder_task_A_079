use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use sqlx::PgPool;

use crate::cox_model;
use crate::models::*;
use crate::network_analysis;

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/cities", get(list_cities))
        .route("/cities/{id}", get(get_city))
        .route("/cities/{id}/timeline", get(get_city_timeline))
        .route("/cities/status", get(get_cities_by_year))
        .route("/climate", get(list_climate))
        .route("/climate/summary", get(climate_summary))
        .route("/trade", get(list_trade))
        .route("/trade/arrows", get(trade_arrows))
        .route("/analysis/cox", get(run_cox_analysis))
        .route("/analysis/network", get(network_analysis_handler))
        .route("/analysis/route-shift", get(route_shift_handler))
        .with_state(pool)
}

async fn list_cities(State(pool): State<PgPool>) -> Json<Vec<City>> {
    let cities = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities ORDER BY id"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(cities)
}

async fn get_city(State(pool): State<PgPool>, Path(id): Path<i32>) -> Json<Option<City>> {
    let city = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    Json(city)
}

async fn get_city_timeline(State(pool): State<PgPool>, Path(id): Path<i32>) -> Json<CityTimeline> {
    let city = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let city = city.unwrap_or_else(|| City {
        id: 0,
        name: String::new(),
        name_cn: None,
        longitude: 0.0,
        latitude: 0.0,
        founded_year: 0,
        prosperity_start: 0,
        prosperity_end: 0,
        decline_year: None,
        decline_reason: None,
        region: None,
        description: None,
        population_peak: None,
        trade_volume: None,
    });

    let region = city.region.clone().unwrap_or_default();

    let climate_records = sqlx::query_as::<_, ClimateData>(
        "SELECT id, period_start, period_end, region, temperature_anomaly, precipitation_index, glacier_advance, glacier_note, notes FROM climate_data WHERE region = $1 ORDER BY period_start"
    )
    .bind(&region)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(CityTimeline {
        city,
        climate_records,
    })
}

async fn get_cities_by_year(
    State(pool): State<PgPool>,
    Query(params): Query<YearQuery>,
) -> Json<Vec<CityStatus>> {
    let year = params.year;
    let cities = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities WHERE founded_year <= $1 ORDER BY id"
    )
    .bind(year)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let statuses: Vec<CityStatus> = cities
        .into_iter()
        .map(|city| {
            let status = if city.prosperity_start <= year && year <= city.prosperity_end {
                "prosperity".to_string()
            } else if city.decline_year.map_or(false, |d| year >= d) {
                "decline".to_string()
            } else {
                "transition".to_string()
            };
            CityStatus { city, status }
        })
        .collect();

    Json(statuses)
}

async fn list_climate(State(pool): State<PgPool>) -> Json<Vec<ClimateData>> {
    let data = sqlx::query_as::<_, ClimateData>(
        "SELECT id, period_start, period_end, region, temperature_anomaly, precipitation_index, glacier_advance, glacier_note, notes FROM climate_data ORDER BY period_start, region"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(data)
}

async fn climate_summary(State(pool): State<PgPool>) -> Json<Vec<RegionClimateSummary>> {
    let data = sqlx::query_as::<_, ClimateData>(
        "SELECT id, period_start, period_end, region, temperature_anomaly, precipitation_index, glacier_advance, glacier_note, notes FROM climate_data ORDER BY region, period_start"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut summaries: Vec<RegionClimateSummary> = Vec::new();
    let regions = ["关中", "河西走廊", "塔里木盆地", "河中地区", "美索不达米亚"];

    for region in &regions {
        let records: Vec<&ClimateData> = data.iter().filter(|d| d.region == *region).collect();
        if records.is_empty() {
            continue;
        }
        let avg_temp = records
            .iter()
            .filter_map(|r| r.temperature_anomaly)
            .sum::<f64>()
            / records.iter().filter_map(|r| r.temperature_anomaly).count().max(1) as f64;
        let avg_precip = records
            .iter()
            .filter_map(|r| r.precipitation_index)
            .sum::<f64>()
            / records.iter().filter_map(|r| r.precipitation_index).count().max(1) as f64;
        let glacier_pct = records
            .iter()
            .filter(|r| r.glacier_advance == Some(true))
            .count() as f64
            / records.len().max(1) as f64
            * 100.0;

        summaries.push(RegionClimateSummary {
            region: region.to_string(),
            avg_temperature_anomaly: (avg_temp * 100.0).round() / 100.0,
            avg_precipitation_index: (avg_precip * 100.0).round() / 100.0,
            glacier_advance_percent: (glacier_pct * 10.0).round() / 10.0,
        });
    }

    Json(summaries)
}

async fn list_trade(State(pool): State<PgPool>) -> Json<Vec<TradeConnection>> {
    let data = sqlx::query_as::<_, TradeConnection>(
        "SELECT id, city_from, city_to, period_start, period_end, trade_volume, route_type FROM trade_connections ORDER BY period_start, id"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(data)
}

async fn trade_arrows(State(pool): State<PgPool>) -> Json<Vec<RouteArrow>> {
    let connections = sqlx::query_as::<_, TradeConnection>(
        "SELECT id, city_from, city_to, period_start, period_end, trade_volume, route_type FROM trade_connections ORDER BY period_start"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let cities = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let city_map: std::collections::HashMap<i32, (f64, f64)> = cities
        .iter()
        .map(|c| (c.id, (c.longitude, c.latitude)))
        .collect();

    let arrows: Vec<RouteArrow> = connections
        .into_iter()
        .filter_map(|conn| {
            let from = city_map.get(&conn.city_from)?;
            let to = city_map.get(&conn.city_to)?;
            Some(RouteArrow {
                from_lon: from.0,
                from_lat: from.1,
                to_lon: to.0,
                to_lat: to.1,
                period_start: conn.period_start,
                period_end: conn.period_end,
                trade_volume: conn.trade_volume.unwrap_or(500),
                route_type: conn.route_type.unwrap_or_else(|| "land".to_string()),
            })
        })
        .collect();

    Json(arrows)
}

async fn run_cox_analysis(State(pool): State<PgPool>) -> Json<AnalysisResponse> {
    let cities = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let climate = sqlx::query_as::<_, ClimateData>(
        "SELECT id, period_start, period_end, region, temperature_anomaly, precipitation_index, glacier_advance, glacier_note, notes FROM climate_data ORDER BY period_start"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let inputs = cox_model::prepare_attribution_data(&cities, &climate);
    let result = cox_model::run_cox_model(&inputs);

    Json(result)
}

async fn network_analysis_handler(
    State(pool): State<PgPool>,
    Query(params): Query<YearQuery>,
) -> Json<Vec<NetworkMetrics>> {
    let year = params.year;

    let connections = sqlx::query_as::<_, TradeConnection>(
        "SELECT id, city_from, city_to, period_start, period_end, trade_volume, route_type FROM trade_connections WHERE period_start <= $1 AND period_end >= $1"
    )
    .bind(year)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let cities = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let metrics = network_analysis::compute_network_metrics(&cities, &connections);
    Json(metrics)
}

async fn route_shift_handler(State(pool): State<PgPool>) -> Json<Vec<TradeRouteShift>> {
    let connections = sqlx::query_as::<_, TradeConnection>(
        "SELECT id, city_from, city_to, period_start, period_end, trade_volume, route_type FROM trade_connections ORDER BY period_start"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let cities = sqlx::query_as::<_, City>(
        "SELECT id, name, name_cn, longitude, latitude, founded_year, prosperity_start, prosperity_end, decline_year, decline_reason, region, description, population_peak, trade_volume FROM cities"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let shifts = network_analysis::compute_route_shifts(&cities, &connections);
    Json(shifts)
}
