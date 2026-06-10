use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoxModelConfig {
    pub interval_width: i32,
    pub max_iterations: usize,
    pub convergence_tolerance: f64,
    pub step_scale_threshold: f64,
    pub lag_years: i32,
    pub confidence_level: f64,
    pub min_sample_size: i32,
}

impl Default for CoxModelConfig {
    fn default() -> Self {
        Self {
            interval_width: 200,
            max_iterations: 300,
            convergence_tolerance: 1e-8,
            step_scale_threshold: 2.0,
            lag_years: 200,
            confidence_level: 0.95,
            min_sample_size: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_land_route_km: f64,
    pub max_sea_route_km: f64,
    pub min_neighbors: usize,
    pub neighbor_distance_km: f64,
    pub eigenvector_max_iter: usize,
    pub eigenvector_tolerance: f64,
    pub smoothing_kernel: [f64; 5],
    pub interpolation_weights: [(i32, f64); 4],
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_land_route_km: 800.0,
            max_sea_route_km: 3000.0,
            min_neighbors: 5,
            neighbor_distance_km: 300.0,
            eigenvector_max_iter: 100,
            eigenvector_tolerance: 1e-10,
            smoothing_kernel: [0.1, 0.25, 0.3, 0.25, 0.1],
            interpolation_weights: [(-1, 2.0), (1, 2.0), (-2, 1.0), (2, 1.0)],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainBarrier {
    pub lon_min: f64,
    pub lat_min: f64,
    pub lon_max: f64,
    pub lat_max: f64,
}

impl MountainBarrier {
    pub const TIEN_SHAN: Self = Self { lon_min: 73.0, lat_min: 37.0, lon_max: 80.0, lat_max: 42.0 };
    pub const IRANIAN_PLATEAU: Self = Self { lon_min: 35.0, lat_min: 36.0, lon_max: 45.0, lat_max: 42.0 };
    pub const HIMALAYA: Self = Self { lon_min: 75.0, lat_min: 27.0, lon_max: 97.0, lat_max: 36.0 };
    pub const HENGDUAN: Self = Self { lon_min: 100.0, lat_min: 28.0, lon_max: 105.0, lat_max: 35.0 };

    pub fn all() -> [Self; 4] {
        [Self::TIEN_SHAN, Self::IRANIAN_PLATEAU, Self::HIMALAYA, Self::HENGDUAN]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub cox: CoxModelConfig,
    pub network: NetworkConfig,
    pub barriers: Vec<MountainBarrier>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cox: CoxModelConfig::default(),
            network: NetworkConfig::default(),
            barriers: MountainBarrier::all().to_vec(),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(v) = std::env::var("COX_INTERVAL_WIDTH") {
            if let Ok(n) = v.parse::<i32>() { config.cox.interval_width = n; }
        }
        if let Ok(v) = std::env::var("COX_MAX_ITER") {
            if let Ok(n) = v.parse::<usize>() { config.cox.max_iterations = n; }
        }
        if let Ok(v) = std::env::var("COX_TOLERANCE") {
            if let Ok(n) = v.parse::<f64>() { config.cox.convergence_tolerance = n; }
        }
        if let Ok(v) = std::env::var("COX_LAG_YEARS") {
            if let Ok(n) = v.parse::<i32>() { config.cox.lag_years = n; }
        }
        if let Ok(v) = std::env::var("NET_MAX_LAND_KM") {
            if let Ok(n) = v.parse::<f64>() { config.network.max_land_route_km = n; }
        }
        if let Ok(v) = std::env::var("NET_MAX_SEA_KM") {
            if let Ok(n) = v.parse::<f64>() { config.network.max_sea_route_km = n; }
        }
        if let Ok(v) = std::env::var("NET_MIN_NEIGHBORS") {
            if let Ok(n) = v.parse::<usize>() { config.network.min_neighbors = n; }
        }

        config
    }
}
