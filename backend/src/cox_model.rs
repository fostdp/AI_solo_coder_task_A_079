use crate::models::{TimeVaryingInterval, City, ClimateData, CoxResult, AnalysisResponse};

const INTERVAL_WIDTH: i32 = 200;

pub fn prepare_time_varying_data(
    cities: &[City],
    climate: &[ClimateData],
    connections: &[crate::models::TradeConnection],
) -> Vec<TimeVaryingInterval> {
    let mut intervals = Vec::new();

    let region_climate_map: std::collections::HashMap<&str, Vec<&ClimateData>> = {
        let mut m = std::collections::HashMap::new();
        for c in climate {
            m.entry(c.region.as_str()).or_default().push(c);
        }
        m
    };

    let city_trade_map = build_city_trade_map(connections);

    for city in cities {
        let obs_start = city.founded_year;
        let obs_end = city.decline_year.unwrap_or(1500);
        if obs_end <= obs_start {
            continue;
        }

        let region = city.region.as_deref().unwrap_or("");
        let region_climate = region_climate_map.get(region).cloned().unwrap_or_default();

        let mut period_start = obs_start;
        while period_start < obs_end {
            let period_end = std::cmp::min(period_start + INTERVAL_WIDTH, obs_end);
            let event = if period_end == obs_end && city.decline_year.is_some() {
                1
            } else {
                0
            };

            let temp_anomaly = avg_climate_in_period(
                &region_climate,
                |c| c.temperature_anomaly,
                period_start,
                period_end,
            );

            let precip_index = avg_climate_in_period(
                &region_climate,
                |c| c.precipitation_index,
                period_start,
                period_end,
            );

            let glacier_count = region_climate
                .iter()
                .filter(|c| {
                    c.glacier_advance == Some(true)
                        && c.period_end > period_start
                        && c.period_start < period_end
                })
                .count() as f64;

            let temp_lag = avg_climate_in_period(
                &region_climate,
                |c| c.temperature_anomaly,
                period_start - 200,
                period_start,
            );

            let temp_change = if temp_lag.is_nan() {
                0.0
            } else {
                temp_anomaly - temp_lag
            };

            let precip_lag = avg_climate_in_period(
                &region_climate,
                |c| c.precipitation_index,
                period_start - 200,
                period_start,
            );

            let precip_change = if precip_lag.is_nan() {
                0.0
            } else {
                precip_index - precip_lag
            };

            let prev_trade = compute_period_trade(city.id, &city_trade_map, period_start - 200, period_start);
            let curr_trade = compute_period_trade(city.id, &city_trade_map, period_start, period_end);
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

    intervals
}

fn avg_climate_in_period(
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

type CityTradeMap = std::collections::HashMap<i32, Vec<(i32, i32, i32)>>;

fn build_city_trade_map(connections: &[crate::models::TradeConnection]) -> CityTradeMap {
    let mut map: CityTradeMap = std::collections::HashMap::new();
    for conn in connections {
        let vol = conn.trade_volume.unwrap_or(500);
        map.entry(conn.city_from).or_default().push((conn.period_start, conn.period_end, vol));
        map.entry(conn.city_to).or_default().push((conn.period_start, conn.period_end, vol));
    }
    map
}

fn compute_period_trade(city_id: i32, map: &CityTradeMap, start: i32, end: i32) -> i64 {
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

pub fn run_time_varying_cox(intervals: &[TimeVaryingInterval]) -> AnalysisResponse {
    let n = intervals.len() as i32;
    if n < 10 {
        return AnalysisResponse {
            cox_results: vec![],
            sample_size: n,
            log_likelihood: 0.0,
            concordance: 0.0,
        };
    }

    let variables = [
        "temp_anomaly",
        "precip_index",
        "temp_change",
        "precip_change",
        "route_change",
        "glacier_advance",
    ];

    let mut beta = vec![0.0; variables.len()];

    for _ in 0..300 {
        let (gradient, hessian) = compute_tv_gradient_hessian(intervals, &beta);
        let step = solve_newton_step(&gradient, &hessian);
        if step.iter().all(|s| s.abs() < 1e-8) {
            break;
        }
        let max_step = step.iter().map(|s| s.abs()).fold(0.0f64, f64::max);
        let step_scale = if max_step > 2.0 { 2.0 / max_step } else { 1.0 };
        for i in 0..beta.len() {
            beta[i] += step_scale * step[i];
        }
    }

    let (_, hessian) = compute_tv_gradient_hessian(intervals, &beta);
    let log_likelihood = compute_tv_log_likelihood(intervals, &beta);
    let concordance = compute_tv_concordance(intervals, &beta);

    let mut results = Vec::new();
    for (i, var_name) in variables.iter().enumerate() {
        let se = if i < hessian.len() && hessian[i][i] > 0.0 {
            (1.0 / hessian[i][i]).sqrt()
        } else {
            0.1
        };

        let hr = beta[i].exp();
        let z = beta[i] / se;
        let p_value = 2.0 * (1.0 - normal_cdf(z.abs()));
        let ci_lower = (beta[i] - 1.96 * se).exp();
        let ci_upper = (beta[i] + 1.96 * se).exp();

        results.push(CoxResult {
            variable: var_name.to_string(),
            coefficient: (beta[i] * 10000.0).round() / 10000.0,
            hazard_ratio: (hr * 10000.0).round() / 10000.0,
            std_error: (se * 10000.0).round() / 10000.0,
            p_value: (p_value * 10000.0).round() / 10000.0,
            confidence_interval_lower: (ci_lower * 10000.0).round() / 10000.0,
            confidence_interval_upper: (ci_upper * 10000.0).round() / 10000.0,
        });
    }

    AnalysisResponse {
        cox_results: results,
        sample_size: n,
        log_likelihood: (log_likelihood * 100.0).round() / 100.0,
        concordance: (concordance * 1000.0).round() / 1000.0,
    }
}

fn compute_tv_log_likelihood(intervals: &[TimeVaryingInterval], beta: &[f64]) -> f64 {
    let mut log_lik = 0.0;

    let mut event_times: Vec<i32> = intervals
        .iter()
        .filter(|iv| iv.event == 1)
        .map(|iv| iv.stop)
        .collect();
    event_times.sort();
    event_times.dedup();

    for &t in &event_times {
        let events_at_t: Vec<&TimeVaryingInterval> = intervals
            .iter()
            .filter(|iv| iv.event == 1 && iv.stop == t)
            .collect();

        let risk_set: Vec<&TimeVaryingInterval> = intervals
            .iter()
            .filter(|iv| iv.start < t && iv.stop >= t)
            .collect();

        if risk_set.is_empty() {
            continue;
        }

        let sum_risk: f64 = risk_set.iter().map(|iv| compute_risk_score_tv(iv, beta)).sum();

        if sum_risk <= 0.0 {
            continue;
        }

        for ev in &events_at_t {
            let risk_i = compute_risk_score_tv(ev, beta);
            log_lik += risk_i.ln() - sum_risk.ln();
        }
    }

    log_lik
}

fn compute_tv_gradient_hessian(
    intervals: &[TimeVaryingInterval],
    beta: &[f64],
) -> (Vec<f64>, Vec<Vec<f64>>) {
    let p = beta.len();
    let mut gradient = vec![0.0; p];
    let mut hessian = vec![vec![0.0; p]; p];

    let mut event_times: Vec<i32> = intervals
        .iter()
        .filter(|iv| iv.event == 1)
        .map(|iv| iv.stop)
        .collect();
    event_times.sort();
    event_times.dedup();

    for &t in &event_times {
        let events_at_t: Vec<&TimeVaryingInterval> = intervals
            .iter()
            .filter(|iv| iv.event == 1 && iv.stop == t)
            .collect();

        let risk_set: Vec<&TimeVaryingInterval> = intervals
            .iter()
            .filter(|iv| iv.start < t && iv.stop >= t)
            .collect();

        if risk_set.is_empty() {
            continue;
        }

        let mut sum_risk = 0.0f64;
        let mut sum_weighted = vec![0.0; p];
        let mut sum_weighted_outer = vec![vec![0.0; p]; p];

        for iv in &risk_set {
            let risk_j = compute_risk_score_tv(iv, beta);
            let x_j = get_covariates_tv(iv);
            sum_risk += risk_j;

            for k in 0..p {
                sum_weighted[k] += risk_j * x_j[k];
                for l in 0..p {
                    sum_weighted_outer[k][l] += risk_j * x_j[k] * x_j[l];
                }
            }
        }

        if sum_risk <= 0.0 {
            continue;
        }

        for ev in &events_at_t {
            let x_i = get_covariates_tv(ev);

            for k in 0..p {
                gradient[k] += x_i[k] - sum_weighted[k] / sum_risk;
                for l in 0..p {
                    hessian[k][l] -= (sum_weighted_outer[k][l] * sum_risk
                        - sum_weighted[k] * sum_weighted[l])
                        / (sum_risk * sum_risk);
                }
            }
        }
    }

    for k in 0..p {
        for l in 0..p {
            hessian[k][l] = -hessian[k][l];
        }
    }

    (gradient, hessian)
}

fn compute_risk_score_tv(iv: &TimeVaryingInterval, beta: &[f64]) -> f64 {
    let x = get_covariates_tv(iv);
    let mut s = 0.0;
    for i in 0..beta.len() {
        s += beta[i] * x[i];
    }
    s.exp()
}

fn get_covariates_tv(iv: &TimeVaryingInterval) -> Vec<f64> {
    vec![
        iv.temp_anomaly,
        iv.precip_index,
        iv.temp_change,
        iv.precip_change,
        iv.route_change,
        iv.glacier_advance,
    ]
}

fn compute_tv_concordance(intervals: &[TimeVaryingInterval], beta: &[f64]) -> f64 {
    let mut concordant = 0i64;
    let mut discordant = 0i64;

    let event_intervals: Vec<&TimeVaryingInterval> =
        intervals.iter().filter(|iv| iv.event == 1).collect();

    let all_intervals: Vec<&TimeVaryingInterval> = intervals.iter().collect();

    for ev in &event_intervals {
        for other in &all_intervals {
            if other.event == 1 && other.stop <= ev.stop {
                continue;
            }
            if other.start >= ev.stop {
                continue;
            }
            if other.stop < ev.stop {
                let risk_ev = linear_pred_tv(ev, beta);
                let risk_other = linear_pred_tv(other, beta);
                if risk_ev > risk_other {
                    concordant += 1;
                } else if risk_ev < risk_other {
                    discordant += 1;
                }
            }
        }
    }

    let total = concordant + discordant;
    if total == 0 {
        0.5
    } else {
        concordant as f64 / total as f64
    }
}

fn linear_pred_tv(iv: &TimeVaryingInterval, beta: &[f64]) -> f64 {
    let x = get_covariates_tv(iv);
    let mut s = 0.0;
    for i in 0..beta.len() {
        s += beta[i] * x[i];
    }
    s
}

fn solve_newton_step(gradient: &[f64], hessian: &[Vec<f64>]) -> Vec<f64> {
    let n = gradient.len();
    let mut aug = vec![vec![0.0; n + 1]; n];

    for i in 0..n {
        for j in 0..n {
            aug[i][j] = hessian[i][j];
        }
        aug[i][n] = gradient[i];
    }

    for col in 0..n {
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        let tmp = aug[col].clone();
        aug[col] = aug[max_row].clone();
        aug[max_row] = tmp;

        if aug[col][col].abs() < 1e-12 {
            continue;
        }

        for row in (col + 1)..n {
            let factor = aug[row][col] / aug[col][col];
            for j in col..=n {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        if aug[i][i].abs() < 1e-12 {
            x[i] = 0.0;
            continue;
        }
        x[i] = aug[i][n];
        for j in (i + 1)..n {
            x[i] -= aug[i][j] * x[j];
        }
        x[i] /= aug[i][i];
    }

    x
}

fn normal_cdf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = x.abs() / std::f64::consts::SQRT2;

    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();

    0.5 * (1.0 + sign * y)
}
