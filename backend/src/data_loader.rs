use crate::config::CoxModelConfig;
use crate::models::{City, ClimateData, TimeVaryingInterval, TradeConnection};
use std::collections::HashMap;

pub struct DataLoader;

impl DataLoader {
    pub fn build_region_climate_map(climate: &[ClimateData]) -> HashMap<&str, Vec<&ClimateData>> {
        let mut map: HashMap<&str, Vec<&ClimateData>> = HashMap::new();
        for c in climate {
            map.entry(c.region.as_str()).or_default().push(c);
        }
        map
    }

    pub fn build_city_trade_map(connections: &[TradeConnection]) -> HashMap<i32, Vec<(i32, i32, i32)>> {
        let mut map: HashMap<i32, Vec<(i32, i32, i32)>> = HashMap::new();
        for conn in connections {
            let vol = conn.trade_volume.unwrap_or(500);
            map.entry(conn.city_from)
                .or_default()
                .push((conn.period_start, conn.period_end, vol));
            map.entry(conn.city_to)
                .or_default()
                .push((conn.period_start, conn.period_end, vol));
        }
        map
    }

    pub fn avg_climate_in_period(
        climate: &[&ClimateData],
        extract: fn(&ClimateData) -> Option<f64>,
        start: i32,
        end: i32,
    ) -> f64 {
        let vals: Vec<f64> = climate
            .iter()
            .filter(|c| c.period_end > start && c.period_start < end)
            .filter_map(extract)
            .collect();
        if vals.is_empty() {
            f64::NAN
        } else {
            vals.iter().sum::<f64>() / vals.len() as f64
        }
    }

    pub fn compute_period_trade(
        city_id: i32,
        map: &HashMap<i32, Vec<(i32, i32, i32)>>,
        start: i32,
        end: i32,
    ) -> i64 {
        map.get(&city_id)
            .map(|trades| {
                trades
                    .iter()
                    .filter(|(ps, pe, _)| *pe > start && *ps < end)
                    .map(|(_, _, vol)| *vol as i64)
                    .sum()
            })
            .unwrap_or(0)
    }
}

pub struct TimeVaryingBuilder<'a> {
    cities: &'a [City],
    climate: &'a [ClimateData],
    connections: &'a [TradeConnection],
    config: &'a CoxModelConfig,
}

impl<'a> TimeVaryingBuilder<'a> {
    pub fn new(
        cities: &'a [City],
        climate: &'a [ClimateData],
        connections: &'a [TradeConnection],
        config: &'a CoxModelConfig,
    ) -> Self {
        Self { cities, climate, connections, config }
    }

    pub fn build(&self) -> Vec<TimeVaryingInterval> {
        let mut intervals = Vec::new();
        let region_climate_map = DataLoader::build_region_climate_map(self.climate);
        let city_trade_map = DataLoader::build_city_trade_map(self.connections);

        for city in self.cities {
            let obs_start = city.founded_year;
            let obs_end = city.decline_year.unwrap_or(1500);
            if obs_end <= obs_start {
                continue;
            }

            let region = city.region.as_deref().unwrap_or("");
            let region_climate = region_climate_map.get(region).cloned().unwrap_or_default();

            self.slice_city(city, &region_climate, &city_trade_map, &mut intervals);
        }

        intervals
    }

    fn slice_city(
        &self,
        city: &City,
        region_climate: &[&ClimateData],
        city_trade_map: &HashMap<i32, Vec<(i32, i32, i32)>>,
        intervals: &mut Vec<TimeVaryingInterval>,
    ) {
        let obs_start = city.founded_year;
        let obs_end = city.decline_year.unwrap_or(1500);
        let width = self.config.interval_width;
        let lag = self.config.lag_years;

        let mut period_start = obs_start;
        while period_start < obs_end {
            let period_end = std::cmp::min(period_start + width, obs_end);
            let event = if period_end == obs_end && city.decline_year.is_some() {
                1
            } else {
                0
            };

            let temp_anomaly =
                DataLoader::avg_climate_in_period(region_climate, |c| c.temperature_anomaly, period_start, period_end);
            let precip_index =
                DataLoader::avg_climate_in_period(region_climate, |c| c.precipitation_index, period_start, period_end);

            let glacier_count = region_climate
                .iter()
                .filter(|c| {
                    c.glacier_advance == Some(true)
                        && c.period_end > period_start
                        && c.period_start < period_end
                })
                .count() as f64;

            let temp_lag = DataLoader::avg_climate_in_period(
                region_climate,
                |c| c.temperature_anomaly,
                period_start - lag,
                period_start,
            );
            let temp_change = if temp_lag.is_nan() { 0.0 } else { temp_anomaly - temp_lag };

            let precip_lag = DataLoader::avg_climate_in_period(
                region_climate,
                |c| c.precipitation_index,
                period_start - lag,
                period_start,
            );
            let precip_change = if precip_lag.is_nan() { 0.0 } else { precip_index - precip_lag };

            let prev_trade = DataLoader::compute_period_trade(city.id, city_trade_map, period_start - lag, period_start);
            let curr_trade = DataLoader::compute_period_trade(city.id, city_trade_map, period_start, period_end);
            let route_change = if prev_trade > 0 {
                (curr_trade as f64 - prev_trade as f64) / prev_trade as f64
            } else if curr_trade > 0 {
                1.0
            } else {
                0.0
            };

            intervals.push(TimeVaryingInterval {
                city_id: city.id,
                start: period_start,
                stop: period_end,
                event,
                temp_anomaly: if temp_anomaly.is_nan() { 0.0 } else { temp_anomaly },
                precip_index: if precip_index.is_nan() { 0.0 } else { precip_index },
                temp_change,
                precip_change,
                route_change,
                glacier_advance: glacier_count,
            });

            period_start = period_end;
        }
    }
}
