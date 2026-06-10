use crate::models::{City, NetworkMetrics, TradeConnection, TradeRouteShift};

const MAX_LAND_ROUTE_KM: f64 = 800.0;
const MAX_SEA_ROUTE_KM: f64 = 3000.0;

const MOUNTAIN_BARRIERS: [(f64, f64, f64, f64); 4] = [
    (73.0, 37.0, 80.0, 42.0),
    (35.0, 36.0, 45.0, 42.0),
    (75.0, 27.0, 97.0, 36.0),
    (100.0, 28.0, 105.0, 35.0),
];

pub fn compute_network_metrics(
    cities: &[City],
    connections: &[TradeConnection],
) -> Vec<NetworkMetrics> {
    let city_ids: Vec<i32> = cities.iter().map(|c| c.id).collect();
    let n = city_ids.len();
    if n == 0 {
        return vec![];
    }

    let id_to_idx: std::collections::HashMap<i32, usize> = city_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    let id_to_name: std::collections::HashMap<i32, String> = cities
        .iter()
        .map(|c| (c.id, c.name.clone()))
        .collect();

    let id_to_pos: std::collections::HashMap<i32, (f64, f64)> = cities
        .iter()
        .map(|c| (c.id, (c.longitude, c.latitude)))
        .collect();

    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];

    for conn in connections {
        if let (Some(&from_idx), Some(&to_idx)) =
            (id_to_idx.get(&conn.city_from), id_to_idx.get(&conn.city_to))
        {
            let from_pos = id_to_pos.get(&conn.city_from);
            let to_pos = id_to_pos.get(&conn.city_to);
            if let (Some(from), Some(to)) = (from_pos, to_pos) {
                let dist = haversine_km(from.0, from.1, to.0, to.1);
                let route_type = conn.route_type.as_deref().unwrap_or("land");
                let max_dist = if route_type == "sea" { MAX_SEA_ROUTE_KM } else { MAX_LAND_ROUTE_KM };
                if dist > max_dist {
                    continue;
                }
                if route_type != "sea" && crosses_mountain_barrier(from, to) {
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
                if city.id == other.id {
                    continue;
                }
                if adj[idx].len() >= 5 {
                    break;
                }
                let dist = haversine_km(city.longitude, city.latitude, other.longitude, other.latitude);
                if dist <= 300.0 && !crosses_mountain_barrier(
                    &(city.longitude, city.latitude),
                    &(other.longitude, other.latitude),
                ) {
                    if let Some(&other_idx) = id_to_idx.get(&other.id) {
                        adj[idx].insert(other_idx);
                        adj[other_idx].insert(idx);
                    }
                }
            }
        }
    }

    let degree_centrality: Vec<f64> = adj
        .iter()
        .map(|neighbors| {
            if n > 1 {
                neighbors.len() as f64 / (n - 1) as f64
            } else {
                0.0
            }
        })
        .collect();

    let betweenness_centrality = compute_betweenness(&adj, n);

    let eigenvector_centrality = compute_eigenvector(&adj, n);

    let mut metrics = Vec::new();
    for (i, &city_id) in city_ids.iter().enumerate() {
        metrics.push(NetworkMetrics {
            city_id,
            city_name: id_to_name.get(&city_id).cloned().unwrap_or_default(),
            degree_centrality: (degree_centrality[i] * 10000.0).round() / 10000.0,
            betweenness_centrality: (betweenness_centrality[i] * 10000.0).round() / 10000.0,
            eigenvector_centrality: (eigenvector_centrality[i] * 10000.0).round() / 10000.0,
        });
    }

    metrics.sort_by(|a, b| b.betweenness_centrality.partial_cmp(&a.betweenness_centrality).unwrap());
    metrics
}

fn crosses_mountain_barrier(from: &(f64, f64), to: &(f64, f64)) -> bool {
    let steps = 20;
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let lon = from.0 + t * (to.0 - from.0);
        let lat = from.1 + t * (to.1 - from.1);
        for (blon, blat, tron, tlat) in &MOUNTAIN_BARRIERS {
            if lon >= *blon && lon <= *tron && lat >= *blat && lat <= *tlat {
                let center_lat = (blat + tlat) / 2.0;
                let rel_lat = (lat - center_lat).abs() / ((tlat - blat) / 2.0);
                let lon_range = tron - blon;
                if lon_range > 5.0 {
                    let mid_lon = (blon + tron) / 2.0;
                    let rel_lon = (lon - mid_lon).abs() / (lon_range / 2.0);
                    if rel_lat > 0.3 && rel_lon < 0.7 {
                        return true;
                    }
                } else if rel_lat > 0.3 {
                    return true;
                }
            }
        }
    }
    false
}

fn haversine_km(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    6371.0 * c
}

pub fn compute_route_shifts(
    cities: &[City],
    connections: &[TradeConnection],
) -> Vec<TradeRouteShift> {
    let periods: Vec<(i32, i32)> = vec![
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

    let city_map: std::collections::HashMap<i32, (f64, f64)> = cities
        .iter()
        .map(|c| (c.id, (c.longitude, c.latitude)))
        .collect();

    let mut raw_centroids: Vec<(i32, i32, f64, f64, i32, i64)> = Vec::new();

    for &(start, end) in &periods {
        let active_connections: Vec<&TradeConnection> = connections
            .iter()
            .filter(|c| c.period_start <= end && c.period_end >= start)
            .collect();

        let mut active_city_ids: std::collections::HashSet<i32> = std::collections::HashSet::new();
        let mut total_volume: i64 = 0;
        let mut weighted_lon = 0.0f64;
        let mut weighted_lat = 0.0f64;
        let mut total_weight = 0.0f64;

        for conn in &active_connections {
            active_city_ids.insert(conn.city_from);
            active_city_ids.insert(conn.city_to);

            let vol = conn.trade_volume.unwrap_or(500) as f64;
            total_volume += conn.trade_volume.unwrap_or(500) as i64;

            if let Some(&(from_lon, from_lat)) = city_map.get(&conn.city_from) {
                weighted_lon += from_lon * vol;
                weighted_lat += from_lat * vol;
                total_weight += vol;
            }
            if let Some(&(to_lon, to_lat)) = city_map.get(&conn.city_to) {
                weighted_lon += to_lon * vol;
                weighted_lat += to_lat * vol;
                total_weight += vol;
            }
        }

        let centroid_lon = if total_weight > 0.0 {
            weighted_lon / total_weight
        } else {
            f64::NAN
        };
        let centroid_lat = if total_weight > 0.0 {
            weighted_lat / total_weight
        } else {
            f64::NAN
        };

        raw_centroids.push((start, end, centroid_lon, centroid_lat, active_city_ids.len() as i32, total_volume));
    }

    let mut filled_centroids: Vec<(i32, i32, f64, f64, i32, i64)> = Vec::new();

    for (i, &(start, end, lon, lat, cities_count, vol)) in raw_centroids.iter().enumerate() {
        if !lon.is_nan() && !lat.is_nan() {
            filled_centroids.push((*start, *end, lon, lat, *cities_count, *vol));
        } else {
            let mut interp_lon = 0.0f64;
            let mut interp_lat = 0.0f64;
            let mut interp_weight = 0.0f64;
            let mut interp_cities = 0i32;
            let mut interp_vol = 0i64;

            for (delta, weight) in [(-1i32, 2.0f64), (1, 2.0), (-2, 1.0), (2, 1.0)] {
                let ni = i as i32 + delta;
                if ni >= 0 && (ni as usize) < raw_centroids.len() {
                    let nc = &raw_centroids[ni as usize];
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
                filled_centroids.push((
                    *start,
                    *end,
                    interp_lon / interp_weight,
                    interp_lat / interp_weight,
                    interp_cities / (interp_weight as i32).max(1),
                    interp_vol / (interp_weight as i64).max(1),
                ));
            } else {
                filled_centroids.push((*start, *end, 70.0, 38.0, *cities_count, *vol));
            }
        }
    }

    let smoothed = smooth_centroid_trajectory(&filled_centroids);

    let mut shifts = Vec::new();
    let mut prev_centroid: Option<(f64, f64)> = None;

    for (i, (start, end, centroid_lon, centroid_lat, active_cities, total_volume)) in smoothed.iter().enumerate() {
        let shift_distance = if let Some((prev_lon, prev_lat)) = prev_centroid {
            let dlat = centroid_lat - prev_lat;
            let lat_rad = centroid_lat * std::f64::consts::PI / 180.0;
            let dlon = (centroid_lon - prev_lon) * lat_rad.cos();
            Some((dlat * dlat + dlon * dlon).sqrt() * 111.0)
        } else {
            None
        };

        shifts.push(TradeRouteShift {
            period: format!("{}~{}", start, end),
            centroid_longitude: (centroid_lon * 100.0).round() / 100.0,
            centroid_latitude: (centroid_lat * 100.0).round() / 100.0,
            active_cities: *active_cities,
            total_trade_volume: *total_volume,
            shift_from_previous: shift_distance.map(|d| (d * 10.0).round() / 10.0),
        });

        if !centroid_lon.is_nan() && !centroid_lat.is_nan() {
            let raw = &filled_centroids[i];
            prev_centroid = Some((raw.2, raw.3));
        }
    }

    shifts
}

fn smooth_centroid_trajectory(centroids: &[(i32, i32, f64, f64, i32, i64)]) -> Vec<(i32, i32, f64, f64, i32, i64)> {
    if centroids.len() < 5 {
        return centroids.to_vec();
    }

    let n = centroids.len();
    let mut smoothed = centroids.to_vec();

    let kernel = [0.1, 0.25, 0.3, 0.25, 0.1];

    for i in 2..(n - 2) {
        let mut s_lon = 0.0f64;
        let mut s_lat = 0.0f64;
        for (k, &w) in kernel.iter().enumerate() {
            let idx = if i + k < 2 { 2 } else if i + k > n + 2 { n + 2 } else { i + k - 2 };
            if idx < n {
                s_lon += centroids[idx].2 * w;
                s_lat += centroids[idx].3 * w;
            }
        }
        smoothed[i].2 = s_lon;
        smoothed[i].3 = s_lat;
    }

    smoothed
}

fn compute_betweenness(adj: &[std::collections::HashSet<usize>], n: usize) -> Vec<f64> {
    let mut betweenness = vec![0.0; n];

    for s in 0..n {
        let mut stack: Vec<usize> = Vec::new();
        let mut predecessors: Vec<Vec<usize>> = vec![vec![]; n];
        let mut sigma = vec![0i64; n];
        sigma[s] = 1;
        let mut dist = vec![-1i32; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
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
            if w != s {
                betweenness[w] += delta[w];
            }
        }
    }

    if n > 2 {
        for b in betweenness.iter_mut() {
            *b /= ((n - 1) * (n - 2)) as f64;
        }
    }

    betweenness
}

fn compute_eigenvector(adj: &[std::collections::HashSet<usize>], n: usize) -> Vec<f64> {
    let mut x = vec![1.0 / n as f64; n];

    for _ in 0..100 {
        let mut new_x = vec![0.0; n];
        for i in 0..n {
            for &j in &adj[i] {
                new_x[i] += x[j];
            }
        }

        let norm: f64 = new_x.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm < 1e-12 {
            break;
        }
        for i in 0..n {
            new_x[i] /= norm;
        }

        let diff: f64 = x.iter().zip(new_x.iter()).map(|(a, b)| (a - b).abs()).sum();
        x = new_x;
        if diff < 1e-10 {
            break;
        }
    }

    let max_val = x.iter().cloned().fold(0.0f64, f64::max);
    if max_val > 0.0 {
        for v in x.iter_mut() {
            *v /= max_val;
        }
    }

    x
}
