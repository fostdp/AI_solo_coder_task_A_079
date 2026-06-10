use crate::config::{MountainBarrier, NetworkConfig};
use crate::models::{City, NetworkMetrics, TradeConnection, TradeRouteShift};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct RouteAnalyzer<'a> {
    config: &'a NetworkConfig,
    barriers: &'a [MountainBarrier],
}

impl<'a> RouteAnalyzer<'a> {
    pub fn new(config: &'a NetworkConfig, barriers: &'a [MountainBarrier]) -> Self {
        Self { config, barriers }
    }

    pub fn compute_network_metrics(
        &self,
        cities: &[City],
        connections: &[TradeConnection],
    ) -> Vec<NetworkMetrics> {
        let city_ids: Vec<i32> = cities.iter().map(|c| c.id).collect();
        let n = city_ids.len();
        if n == 0 { return vec![]; }

        let id_to_idx: HashMap<i32, usize> = city_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();
        let id_to_name: HashMap<i32, String> = cities
            .iter()
            .map(|c| (c.id, c.name.clone()))
            .collect();
        let id_to_pos: HashMap<i32, (f64, f64)> = cities
            .iter()
            .map(|c| (c.id, (c.longitude, c.latitude)))
            .collect();

        let adj = self.build_adjacency(cities, connections, &id_to_idx, &id_to_pos);

        let degree_centrality: Vec<f64> = adj
            .iter()
            .map(|neighbors| if n > 1 { neighbors.len() as f64 / (n - 1) as f64 } else { 0.0 })
            .collect();

        let betweenness_centrality = self.compute_betweenness(&adj, n);
        let eigenvector_centrality = self.compute_eigenvector(&adj, n);

        let mut metrics = Vec::new();
        for (i, &city_id) in city_ids.iter().enumerate() {
            metrics.push(NetworkMetrics {
                city_id,
                city_name: id_to_name.get(&city_id).cloned().unwrap_or_default(),
                degree_centrality: round4(degree_centrality[i]),
                betweenness_centrality: round4(betweenness_centrality[i]),
                eigenvector_centrality: round4(eigenvector_centrality[i]),
            });
        }

        metrics.sort_by(|a, b| b.betweenness_centrality.partial_cmp(&a.betweenness_centrality).unwrap());
        metrics
    }

    fn build_adjacency(
        &self,
        cities: &[City],
        connections: &[TradeConnection],
        id_to_idx: &HashMap<i32, usize>,
        id_to_pos: &HashMap<i32, (f64, f64)>,
    ) -> Vec<HashSet<usize>> {
        let n = cities.len();
        let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];

        for conn in connections {
            if let (Some(&from_idx), Some(&to_idx)) =
                (id_to_idx.get(&conn.city_from), id_to_idx.get(&conn.city_to))
            {
                if let (Some(from), Some(to)) =
                    (id_to_pos.get(&conn.city_from), id_to_pos.get(&conn.city_to))
                {
                    let dist = haversine_km(from.0, from.1, to.0, to.1);
                    let route_type = conn.route_type.as_deref().unwrap_or("land");
                    let max_dist = if route_type == "sea" {
                        self.config.max_sea_route_km
                    } else {
                        self.config.max_land_route_km
                    };
                    if dist > max_dist { continue; }
                    if route_type != "sea" && self.crosses_mountain_barrier(from, to) {
                        continue;
                    }
                }
                adj[from_idx].insert(to_idx);
                adj[to_idx].insert(from_idx);
            }
        }

        for city in cities {
            if let Some(&idx) = id_to_idx.get(&city.id) {
                for other in cities {
                    if city.id == other.id { continue; }
                    if adj[idx].len() >= self.config.min_neighbors { break; }
                    let dist = haversine_km(city.longitude, city.latitude, other.longitude, other.latitude);
                    if dist <= self.config.neighbor_distance_km
                        && !self.crosses_mountain_barrier(
                            &(city.longitude, city.latitude),
                            &(other.longitude, other.latitude),
                        )
                    {
                        if let Some(&other_idx) = id_to_idx.get(&other.id) {
                            adj[idx].insert(other_idx);
                            adj[other_idx].insert(idx);
                        }
                    }
                }
            }
        }

        adj
    }

    pub fn compute_route_shifts(
        &self,
        cities: &[City],
        connections: &[TradeConnection],
        periods: &[(i32, i32)],
    ) -> Vec<TradeRouteShift> {
        let city_map: HashMap<i32, (f64, f64)> = cities
            .iter()
            .map(|c| (c.id, (c.longitude, c.latitude)))
            .collect();

        let mut raw_centroids: Vec<(i32, i32, f64, f64, i32, i64)> = Vec::new();

        for &(start, end) in periods {
            let active: Vec<&TradeConnection> = connections
                .iter()
                .filter(|c| c.period_start <= end && c.period_end >= start)
                .collect();

            let mut active_ids: HashSet<i32> = HashSet::new();
            let mut total_volume: i64 = 0;
            let mut weighted_lon = 0.0f64;
            let mut weighted_lat = 0.0f64;
            let mut total_weight = 0.0f64;

            for conn in &active {
                active_ids.insert(conn.city_from);
                active_ids.insert(conn.city_to);
                let vol = conn.trade_volume.unwrap_or(500) as f64;
                total_volume += conn.trade_volume.unwrap_or(500) as i64;

                if let Some(&(lon, lat)) = city_map.get(&conn.city_from) {
                    weighted_lon += lon * vol;
                    weighted_lat += lat * vol;
                    total_weight += vol;
                }
                if let Some(&(lon, lat)) = city_map.get(&conn.city_to) {
                    weighted_lon += lon * vol;
                    weighted_lat += lat * vol;
                    total_weight += vol;
                }
            }

            let centroid_lon = if total_weight > 0.0 { weighted_lon / total_weight } else { f64::NAN };
            let centroid_lat = if total_weight > 0.0 { weighted_lat / total_weight } else { f64::NAN };

            raw_centroids.push((start, end, centroid_lon, centroid_lat, active_ids.len() as i32, total_volume));
        }

        let filled = self.interpolate_centroids(&raw_centroids);
        let smoothed = self.smooth_trajectory(&filled);

        let mut shifts = Vec::new();
        let mut prev: Option<(f64, f64)> = None;

        for (i, (start, end, lon, lat, active, vol)) in smoothed.iter().enumerate() {
            let shift = if let Some((prev_lon, prev_lat)) = prev {
                let dlat = lat - prev_lat;
                let lat_rad = lat.to_radians();
                let dlon = (lon - prev_lon) * lat_rad.cos();
                Some((dlat * dlat + dlon * dlon).sqrt() * 111.0)
            } else {
                None
            };

            shifts.push(TradeRouteShift {
                period: format!("{}~{}", start, end),
                centroid_longitude: round2(*lon),
                centroid_latitude: round2(*lat),
                active_cities: *active,
                total_trade_volume: *vol,
                shift_from_previous: shift.map(|d| (d * 10.0).round() / 10.0),
            });

            if !lon.is_nan() && !lat.is_nan() {
                prev = Some((filled[i].2, filled[i].3));
            }
        }

        shifts
    }

    fn interpolate_centroids(
        &self,
        raw: &[(i32, i32, f64, f64, i32, i64)],
    ) -> Vec<(i32, i32, f64, f64, i32, i64)> {
        let mut filled = Vec::new();
        let weights = &self.config.interpolation_weights;

        for (i, &(start, end, lon, lat, cities_count, vol)) in raw.iter().enumerate() {
            if !lon.is_nan() && !lat.is_nan() {
                filled.push((start, end, lon, lat, cities_count, vol));
            } else {
                let mut interp_lon = 0.0f64;
                let mut interp_lat = 0.0f64;
                let mut interp_weight = 0.0f64;
                let mut interp_cities = 0i32;
                let mut interp_vol = 0i64;

                for (delta, weight) in weights {
                    let ni = i as i32 + delta;
                    if ni >= 0 && (ni as usize) < raw.len() {
                        let nc = &raw[ni as usize];
                        if !nc.2.is_nan() && !nc.3.is_nan() {
                            interp_lon += nc.2 * weight;
                            interp_lat += nc.3 * weight;
                            interp_cities += nc.4;
                            interp_vol += nc.5;
                            interp_weight += weight;
                        }
                    }
                }

                if interp_weight > 0.0 {
                    filled.push((
                        start, end,
                        interp_lon / interp_weight,
                        interp_lat / interp_weight,
                        interp_cities / (interp_weight as i32).max(1),
                        interp_vol / (interp_weight as i64).max(1),
                    ));
                } else {
                    filled.push((start, end, 70.0, 38.0, cities_count, vol));
                }
            }
        }

        filled
    }

    fn smooth_trajectory(
        &self,
        centroids: &[(i32, i32, f64, f64, i32, i64)],
    ) -> Vec<(i32, i32, f64, f64, i32, i64)> {
        if centroids.len() < 5 { return centroids.to_vec(); }

        let n = centroids.len();
        let mut smoothed = centroids.to_vec();
        let kernel = self.config.smoothing_kernel;

        for i in 2..(n - 2) {
            let mut s_lon = 0.0f64;
            let mut s_lat = 0.0f64;
            for (k, &w) in kernel.iter().enumerate() {
                let idx = (i + k).saturating_sub(2).min(n - 1);
                s_lon += centroids[idx].2 * w;
                s_lat += centroids[idx].3 * w;
            }
            smoothed[i].2 = s_lon;
            smoothed[i].3 = s_lat;
        }

        smoothed
    }

    fn crosses_mountain_barrier(&self, from: &(f64, f64), to: &(f64, f64)) -> bool {
        const STEPS: i32 = 20;
        for i in 0..=STEPS {
            let t = i as f64 / STEPS as f64;
            let lon = from.0 + t * (to.0 - from.0);
            let lat = from.1 + t * (to.1 - from.1);
            for b in self.barriers {
                if lon >= b.lon_min && lon <= b.lon_max && lat >= b.lat_min && lat <= b.lat_max {
                    let center_lat = (b.lat_min + b.lat_max) / 2.0;
                    let rel_lat = (lat - center_lat).abs() / ((b.lat_max - b.lat_min) / 2.0);
                    let lon_range = b.lon_max - b.lon_min;
                    if lon_range > 5.0 {
                        let mid_lon = (b.lon_min + b.lon_max) / 2.0;
                        let rel_lon = (lon - mid_lon).abs() / (lon_range / 2.0);
                        if rel_lat > 0.3 && rel_lon < 0.7 { return true; }
                    } else if rel_lat > 0.3 {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn compute_betweenness(&self, adj: &[HashSet<usize>], n: usize) -> Vec<f64> {
        let mut betweenness = vec![0.0; n];
        for s in 0..n {
            let mut stack: Vec<usize> = Vec::new();
            let mut predecessors: Vec<Vec<usize>> = vec![vec![]; n];
            let mut sigma = vec![0i64; n];
            sigma[s] = 1;
            let mut dist = vec![-1i32; n];
            dist[s] = 0;
            let mut queue = VecDeque::new();
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                for &w in &adj[v] {
                    if dist[w] < 0 {
                        dist[w] = dist[v] + 1;
                        queue.push_back(w);
                    }
                    if dist[w] == dist[v] + 1 {
                        sigma[w] += sigma[v];
                        predecessors[w].push(v);
                    }
                }
            }

            let mut delta = vec![0.0; n];
            while let Some(w) = stack.pop() {
                for &v in &predecessors[w] {
                    delta[v] += (sigma[v] as f64 / sigma[w] as f64) * (1.0 + delta[w]);
                }
                if w != s { betweenness[w] += delta[w]; }
            }
        }

        if n > 2 {
            for b in betweenness.iter_mut() {
                *b /= ((n - 1) * (n - 2)) as f64;
            }
        }
        betweenness
    }

    fn compute_eigenvector(&self, adj: &[HashSet<usize>], n: usize) -> Vec<f64> {
        let mut x = vec![1.0 / n as f64; n];
        for _ in 0..self.config.eigenvector_max_iter {
            let mut new_x = vec![0.0; n];
            for i in 0..n {
                for &j in &adj[i] { new_x[i] += x[j]; }
            }
            let norm: f64 = new_x.iter().map(|v| v * v).sum::<f64>().sqrt();
            if norm < 1e-12 { break; }
            for i in 0..n { new_x[i] /= norm; }
            let diff: f64 = x.iter().zip(new_x.iter()).map(|(a, b)| (a - b).abs()).sum();
            x = new_x;
            if diff < self.config.eigenvector_tolerance { break; }
        }
        let max_val = x.iter().cloned().fold(0.0f64, f64::max);
        if max_val > 0.0 { for v in x.iter_mut() { *v /= max_val; } }
        x
    }
}

fn haversine_km(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    6371.0 * c
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

pub const DEFAULT_PERIODS: [(i32, i32); 9] = [
    (-200, 0),
    (0, 200),
    (200, 400),
    (400, 600),
    (600, 800),
    (800, 1000),
    (1000, 1200),
    (1200, 1400),
    (1400, 1500),
];
