use crate::config::CoxModelConfig;
use crate::models::{AnalysisResponse, CoxResult, TimeVaryingInterval};

const VARIABLE_NAMES: &[&str] = &[
    "temp_anomaly",
    "precip_index",
    "temp_change",
    "precip_change",
    "route_change",
    "glacier_advance",
];

pub struct CoxModel<'a> {
    config: &'a CoxModelConfig,
    intervals: &'a [TimeVaryingInterval],
}

impl<'a> CoxModel<'a> {
    pub fn new(intervals: &'a [TimeVaryingInterval], config: &'a CoxModelConfig) -> Self {
        Self { config, intervals }
    }

    pub fn fit(&self) -> AnalysisResponse {
        let n = self.intervals.len() as i32;
        if n < self.config.min_sample_size {
            return AnalysisResponse {
                cox_results: vec![],
                sample_size: n,
                log_likelihood: 0.0,
                concordance: 0.0,
            };
        }

        let p = VARIABLE_NAMES.len();
        let mut beta = vec![0.0; p];

        for _ in 0..self.config.max_iterations {
            let (gradient, hessian) = self.compute_gradient_hessian(&beta);
            let step = solve_newton_step(&gradient, &hessian);
            if step.iter().all(|s| s.abs() < self.config.convergence_tolerance) {
                break;
            }
            let max_step = step.iter().map(|s| s.abs()).fold(0.0f64, f64::max);
            let step_scale = if max_step > self.config.step_scale_threshold {
                self.config.step_scale_threshold / max_step
            } else {
                1.0
            };
            for i in 0..beta.len() {
                beta[i] += step_scale * step[i];
            }
        }

        let (_, hessian) = self.compute_gradient_hessian(&beta);
        let log_likelihood = self.compute_log_likelihood(&beta);
        let concordance = self.compute_concordance(&beta);

        let z_value = (1.0 - (1.0 - self.config.confidence_level) / 2.0)
            .to_string()
            .parse::<f64>()
            .unwrap_or(1.96);
        let ci_multiplier = inverse_normal_cdf(1.0 - (1.0 - self.config.confidence_level) / 2.0);

        let mut results = Vec::new();
        for (i, var_name) in VARIABLE_NAMES.iter().enumerate() {
            let se = if i < hessian.len() && hessian[i][i] > 0.0 {
                (1.0 / hessian[i][i]).sqrt()
            } else {
                0.1
            };

            let hr = beta[i].exp();
            let z = beta[i] / se;
            let p_value = 2.0 * (1.0 - normal_cdf(z.abs()));
            let ci_lower = (beta[i] - ci_multiplier * se).exp();
            let ci_upper = (beta[i] + ci_multiplier * se).exp();

            results.push(CoxResult {
                variable: var_name.to_string(),
                coefficient: round4(beta[i]),
                hazard_ratio: round4(hr),
                std_error: round4(se),
                p_value: round4(p_value),
                confidence_interval_lower: round4(ci_lower),
                confidence_interval_upper: round4(ci_upper),
            });
        }

        AnalysisResponse {
            cox_results: results,
            sample_size: n,
            log_likelihood: (log_likelihood * 100.0).round() / 100.0,
            concordance: (concordance * 1000.0).round() / 1000.0,
        }
    }

    fn compute_log_likelihood(&self, beta: &[f64]) -> f64 {
        let mut log_lik = 0.0;
        let event_times = self.sorted_event_times();

        for &t in &event_times {
            let events_at_t: Vec<&TimeVaryingInterval> = self
                .intervals
                .iter()
                .filter(|iv| iv.event == 1 && iv.stop == t)
                .collect();

            let risk_set: Vec<&TimeVaryingInterval> = self
                .intervals
                .iter()
                .filter(|iv| iv.start < t && iv.stop >= t)
                .collect();

            if risk_set.is_empty() {
                continue;
            }

            let sum_risk: f64 = risk_set.iter().map(|iv| risk_score(iv, beta)).sum();
            if sum_risk <= 0.0 {
                continue;
            }

            for ev in &events_at_t {
                let risk_i = risk_score(ev, beta);
                log_lik += risk_i.ln() - sum_risk.ln();
            }
        }

        log_lik
    }

    fn compute_gradient_hessian(&self, beta: &[f64]) -> (Vec<f64>, Vec<Vec<f64>>) {
        let p = beta.len();
        let mut gradient = vec![0.0; p];
        let mut hessian = vec![vec![0.0; p]; p];

        let event_times = self.sorted_event_times();

        for &t in &event_times {
            let events_at_t: Vec<&TimeVaryingInterval> = self
                .intervals
                .iter()
                .filter(|iv| iv.event == 1 && iv.stop == t)
                .collect();

            let risk_set: Vec<&TimeVaryingInterval> = self
                .intervals
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
                let risk_j = risk_score(iv, beta);
                let x_j = covariates(iv);
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
                let x_i = covariates(ev);
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

    fn compute_concordance(&self, beta: &[f64]) -> f64 {
        let mut concordant = 0i64;
        let mut discordant = 0i64;

        let event_intervals: Vec<&TimeVaryingInterval> =
            self.intervals.iter().filter(|iv| iv.event == 1).collect();

        for ev in &event_intervals {
            for other in self.intervals {
                if other.event == 1 && other.stop <= ev.stop {
                    continue;
                }
                if other.start >= ev.stop {
                    continue;
                }
                if other.stop < ev.stop {
                    let risk_ev = linear_pred(ev, beta);
                    let risk_other = linear_pred(other, beta);
                    if risk_ev > risk_other {
                        concordant += 1;
                    } else if risk_ev < risk_other {
                        discordant += 1;
                    }
                }
            }
        }

        let total = concordant + discordant;
        if total == 0 { 0.5 } else { concordant as f64 / total as f64 }
    }

    fn sorted_event_times(&self) -> Vec<i32> {
        let mut times: Vec<i32> = self
            .intervals
            .iter()
            .filter(|iv| iv.event == 1)
            .map(|iv| iv.stop)
            .collect();
        times.sort();
        times.dedup();
        times
    }
}

fn risk_score(iv: &TimeVaryingInterval, beta: &[f64]) -> f64 {
    linear_pred(iv, beta).exp()
}

fn linear_pred(iv: &TimeVaryingInterval, beta: &[f64]) -> f64 {
    let x = covariates(iv);
    let mut s = 0.0;
    for i in 0..beta.len() {
        s += beta[i] * x[i];
    }
    s
}

fn covariates(iv: &TimeVaryingInterval) -> Vec<f64> {
    vec![
        iv.temp_anomaly,
        iv.precip_index,
        iv.temp_change,
        iv.precip_change,
        iv.route_change,
        iv.glacier_advance,
    ]
}

fn solve_newton_step(gradient: &[f64], hessian: &[Vec<f64>]) -> Vec<f64> {
    let n = gradient.len();
    let mut aug = vec![vec![0.0; n + 1]; n];
    for i in 0..n {
        for j in 0..n { aug[i][j] = hessian[i][j]; }
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
        aug.swap(col, max_row);

        if aug[col][col].abs() < 1e-12 { continue; }

        for row in (col + 1)..n {
            let factor = aug[row][col] / aug[col][col];
            for j in col..=n { aug[row][j] -= factor * aug[col][j]; }
        }
    }

    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        if aug[i][i].abs() < 1e-12 { x[i] = 0.0; continue; }
        x[i] = aug[i][n];
        for j in (i + 1)..n { x[i] -= aug[i][j] * x[j]; }
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

fn inverse_normal_cdf(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 { return f64::NAN; }
    let mut lo = -10.0;
    let mut hi = 10.0;
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if normal_cdf(mid) < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.001);
        assert!((normal_cdf(1.96) - 0.975).abs() < 0.001);
    }

    #[test]
    fn test_inverse_normal_cdf() {
        let z = inverse_normal_cdf(0.975);
        assert!((z - 1.96).abs() < 0.01);
    }
}
