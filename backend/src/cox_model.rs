use crate::models::{AttributionInput, City, ClimateData, CoxResult, AnalysisResponse};

pub fn prepare_attribution_data(cities: &[City], climate: &[ClimateData]) -> Vec<AttributionInput> {
    let mut inputs = Vec::new();

    for city in cities {
        let survival_time = if let Some(dy) = city.decline_year {
            dy - city.founded_year
        } else {
            1500 - city.founded_year
        };

        if survival_time <= 0 {
            continue;
        }

        let event = if city.decline_year.is_some() { 1 } else { 0 };

        let region = city.region.clone().unwrap_or_default();

        let region_climate: Vec<&ClimateData> = climate
            .iter()
            .filter(|c| c.region == region)
            .collect();

        let prosperity_start_period = city.prosperity_start;
        let prosperity_end_period = city.prosperity_end;

        let before: Vec<&ClimateData> = region_climate
            .iter()
            .filter(|c| c.period_end <= prosperity_start_period)
            .take(3)
            .cloned()
            .collect();

        let during: Vec<&ClimateData> = region_climate
            .iter()
            .filter(|c| c.period_start >= prosperity_start_period && c.period_end <= prosperity_end_period)
            .take(5)
            .cloned()
            .collect();

        let temp_before = before.iter().filter_map(|c| c.temperature_anomaly).sum::<f64>()
            / before.iter().filter_map(|c| c.temperature_anomaly).count().max(1) as f64;
        let temp_during = during.iter().filter_map(|c| c.temperature_anomaly).sum::<f64>()
            / during.iter().filter_map(|c| c.temperature_anomaly).count().max(1) as f64;
        let temp_change = temp_during - temp_before;

        let precip_before = before.iter().filter_map(|c| c.precipitation_index).sum::<f64>()
            / before.iter().filter_map(|c| c.precipitation_index).count().max(1) as f64;
        let precip_during = during.iter().filter_map(|c| c.precipitation_index).sum::<f64>()
            / during.iter().filter_map(|c| c.precipitation_index).count().max(1) as f64;
        let precip_change = precip_during - precip_before;

        let route_change = match city.decline_reason.as_deref() {
            Some("trade_route") => 1.0,
            Some("war") => 0.3,
            Some("climate") => 0.1,
            _ => 0.0,
        };

        let glacier_advance = during
            .iter()
            .filter(|c| c.glacier_advance == Some(true))
            .count() as i32;

        inputs.push(AttributionInput {
            city_id: city.id,
            survival_time,
            event,
            temp_change,
            precip_change,
            route_change,
            glacier_advance,
        });
    }

    inputs
}

pub fn run_cox_model(inputs: &[AttributionInput]) -> AnalysisResponse {
    let n = inputs.len() as i32;
    if n < 5 {
        return AnalysisResponse {
            cox_results: vec![],
            sample_size: n,
            log_likelihood: 0.0,
            concordance: 0.0,
        };
    }

    let variables = ["temp_change", "precip_change", "route_change", "glacier_advance"];

    let mut results = Vec::new();

    let mut beta = vec![0.0; variables.len()];

    for _ in 0..200 {
        let (gradient, hessian) = compute_cox_gradient_hessian(inputs, &beta);

        let step = solve_newton_step(&gradient, &hessian);
        if step.iter().all(|s| s.abs() < 1e-8) {
            break;
        }
        for i in 0..beta.len() {
            beta[i] += step[i];
        }
    }

    let (_, hessian) = compute_cox_gradient_hessian(inputs, &beta);

    let log_likelihood = compute_partial_log_likelihood(inputs, &beta);
    let concordance = compute_concordance(inputs, &beta);

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

fn compute_partial_log_likelihood(inputs: &[AttributionInput], beta: &[f64]) -> f64 {
    let mut log_lik = 0.0;

    let mut sorted: Vec<&AttributionInput> = inputs.iter().collect();
    sorted.sort_by(|a, b| a.survival_time.cmp(&b.survival_time));

    for (i, input) in sorted.iter().enumerate() {
        if input.event == 0 {
            continue;
        }

        let risk_i = compute_risk_score(input, beta);

        let mut sum_risk = 0.0f64;
        for j in i..sorted.len() {
            sum_risk += compute_risk_score(sorted[j], beta);
        }

        if sum_risk > 0.0 {
            log_lik += risk_i.ln() - sum_risk.ln();
        }
    }

    log_lik
}

fn compute_cox_gradient_hessian(
    inputs: &[AttributionInput],
    beta: &[f64],
) -> (Vec<f64>, Vec<Vec<f64>>) {
    let p = beta.len();
    let mut gradient = vec![0.0; p];
    let mut hessian = vec![vec![0.0; p]; p];

    let mut sorted: Vec<&AttributionInput> = inputs.iter().collect();
    sorted.sort_by(|a, b| a.survival_time.cmp(&b.survival_time));

    for (i, input) in sorted.iter().enumerate() {
        if input.event == 0 {
            continue;
        }

        let x_i = get_covariates(input);
        let risk_i = compute_risk_score(input, beta);

        let mut sum_risk = 0.0f64;
        let mut sum_weighted = vec![0.0; p];
        let mut sum_weighted_outer = vec![vec![0.0; p]; p];

        for j in i..sorted.len() {
            let x_j = get_covariates(sorted[j]);
            let risk_j = compute_risk_score(sorted[j], beta);
            sum_risk += risk_j;

            for k in 0..p {
                sum_weighted[k] += risk_j * x_j[k];
                for l in 0..p {
                    sum_weighted_outer[k][l] += risk_j * x_j[k] * x_j[l];
                }
            }
        }

        if sum_risk > 0.0 {
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

fn compute_risk_score(input: &AttributionInput, beta: &[f64]) -> f64 {
    let x = get_covariates(input);
    let mut score = 0.0;
    for i in 0..beta.len() {
        score += beta[i] * x[i];
    }
    score.exp()
}

fn get_covariates(input: &AttributionInput) -> Vec<f64> {
    vec![
        input.temp_change,
        input.precip_change,
        input.route_change,
        input.glacier_advance as f64,
    ]
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

fn compute_concordance(inputs: &[AttributionInput], beta: &[f64]) -> f64 {
    let mut concordant = 0i64;
    let mut discordant = 0i64;

    let events: Vec<&AttributionInput> = inputs.iter().filter(|i| i.event == 1).collect();
    let censored: Vec<&AttributionInput> = inputs.iter().filter(|i| i.event == 0).collect();

    for e in &events {
        for c in &censored {
            if e.survival_time < c.survival_time {
                let risk_e = linear_predictor(e, beta);
                let risk_c = linear_predictor(c, beta);
                if risk_e > risk_c {
                    concordant += 1;
                } else if risk_e < risk_c {
                    discordant += 1;
                }
            }
        }
    }

    for i in 0..events.len() {
        for j in (i + 1)..events.len() {
            if events[i].survival_time != events[j].survival_time {
                let (shorter, longer) = if events[i].survival_time < events[j].survival_time {
                    (events[i], events[j])
                } else {
                    (events[j], events[i])
                };
                let risk_s = linear_predictor(shorter, beta);
                let risk_l = linear_predictor(longer, beta);
                if risk_s > risk_l {
                    concordant += 1;
                } else if risk_s < risk_l {
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

fn linear_predictor(input: &AttributionInput, beta: &[f64]) -> f64 {
    let x = get_covariates(input);
    let mut s = 0.0;
    for i in 0..beta.len() {
        s += beta[i] * x[i];
    }
    s
}

fn normal_cdf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs() / std::f64::consts::SQRT2;

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    0.5 * (1.0 + sign * y)
}
