use crate::models::{City, NetworkMetrics, TradeConnection, TradeRouteShift};

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

    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];

    for conn in connections {
        if let (Some(&from_idx), Some(&to_idx)) =
            (id_to_idx.get(&conn.city_from), id_to_idx.get(&conn.city_to))
        {
            adj[from_idx].insert(to_idx);
            adj[to_idx].insert(from_idx);
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

    let mut shifts = Vec::new();
    let mut prev_centroid: Option<(f64, f64)> = None;

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
            70.0
        };
        let centroid_lat = if total_weight > 0.0 {
            weighted_lat / total_weight
        } else {
            38.0
        };

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
            active_cities: active_city_ids.len() as i32,
            total_trade_volume: total_volume,
            shift_from_previous: shift_distance.map(|d| (d * 10.0).round() / 10.0),
        });

        prev_centroid = Some((centroid_lon, centroid_lat));
    }

    shifts
}
