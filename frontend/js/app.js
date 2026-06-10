const API_BASE = '/api';

let map, canvasOverlay, ctx;
let citiesData = [];
let climateData = [];
let tradeArrowsData = [];
let currentYear = 600;
let cityMarkers = {};
let isPlaying = false;
let playInterval = null;
let animationFrame = null;
let routeAnimOffset = 0;

document.addEventListener('DOMContentLoaded', () => {
    initMap();
    initCanvas();
    initTimeline();
    initModals();
    loadData();
});

function initMap() {
    map = L.map('map', {
        center: [38, 65],
        zoom: 4,
        minZoom: 3,
        maxZoom: 8,
        zoomControl: true,
    });

    L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', {
        attribution: '&copy; OpenStreetMap &copy; CARTO',
        subdomains: 'abcd',
        maxZoom: 19
    }).addTo(map);

    const legend = L.control({ position: 'bottomleft' });
    legend.onAdd = function() {
        const div = L.DomUtil.create('div', 'legend');
        div.innerHTML = `
            <div class="legend-item"><div class="legend-dot prosperity"></div>繁荣期</div>
            <div class="legend-item"><div class="legend-dot transition"></div>过渡期</div>
            <div class="legend-item"><div class="legend-dot decline"></div>衰落期</div>
            <div class="route-arrow-legend"><div class="legend-line land"></div>陆路</div>
            <div class="route-arrow-legend"><div class="legend-line sea"></div>海路</div>
        `;
        return div;
    };
    legend.addTo(map);

    map.on('moveend zoomend', () => {
        drawTradeArrows();
    });
}

function initCanvas() {
    canvasOverlay = document.getElementById('canvas-overlay');
    ctx = canvasOverlay.getContext('2d');
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);
}

function resizeCanvas() {
    const container = document.getElementById('map-container');
    canvasOverlay.width = container.clientWidth;
    canvasOverlay.height = container.clientHeight;
    drawTradeArrows();
}

function initTimeline() {
    const slider = document.getElementById('timeline-slider');
    const yearDisplay = document.getElementById('current-year');

    slider.addEventListener('input', (e) => {
        currentYear = parseInt(e.target.value);
        updateYearDisplay();
        updateCityStatuses();
        drawTradeArrows();
    });

    document.getElementById('btn-play').addEventListener('click', () => {
        if (!isPlaying) {
            isPlaying = true;
            playInterval = setInterval(() => {
                currentYear += 5;
                if (currentYear > 1500) currentYear = -200;
                slider.value = currentYear;
                updateYearDisplay();
                updateCityStatuses();
                drawTradeArrows();
            }, 100);
        }
    });

    document.getElementById('btn-pause').addEventListener('click', () => {
        isPlaying = false;
        if (playInterval) clearInterval(playInterval);
    });

    document.getElementById('btn-reset').addEventListener('click', () => {
        currentYear = -200;
        slider.value = currentYear;
        updateYearDisplay();
        updateCityStatuses();
        drawTradeArrows();
    });

    updateYearDisplay();
}

function updateYearDisplay() {
    const yearDisplay = document.getElementById('current-year');
    if (currentYear < 0) {
        yearDisplay.textContent = `前 ${Math.abs(currentYear)} 年`;
    } else {
        yearDisplay.textContent = `公元 ${currentYear} 年`;
    }
}

function initModals() {
    document.querySelectorAll('.modal-close').forEach(btn => {
        btn.addEventListener('click', () => {
            btn.closest('.modal').classList.add('hidden');
        });
    });

    document.querySelectorAll('.modal').forEach(modal => {
        modal.addEventListener('click', (e) => {
            if (e.target === modal) modal.classList.add('hidden');
        });
    });

    document.getElementById('panel-close').addEventListener('click', () => {
        document.getElementById('city-panel').classList.add('hidden');
    });

    document.getElementById('btn-analysis').addEventListener('click', showCoxAnalysis);
    document.getElementById('btn-route-shift').addEventListener('click', showRouteShift);
    document.getElementById('btn-climate').addEventListener('click', showClimateOverview);
}

async function loadData() {
    try {
        const [citiesRes, climateRes, arrowsRes] = await Promise.all([
            fetch(`${API_BASE}/cities`),
            fetch(`${API_BASE}/climate`),
            fetch(`${API_BASE}/trade/arrows`)
        ]);

        citiesData = await citiesRes.json();
        climateData = await climateRes.json();
        tradeArrowsData = await arrowsRes.json();

        addCityMarkers();
        updateCityStatuses();
        startArrowAnimation();
    } catch (err) {
        console.error('Failed to load data:', err);
        citiesData = generateFallbackData();
        addCityMarkers();
        updateCityStatuses();
        startArrowAnimation();
    }
}

function generateFallbackData() {
    const names = [
        { name: "Chang'an", cn: '长安', lon: 108.94, lat: 34.26, f: -200, ps: -150, pe: 700, dy: 907, dr: 'war', reg: '关中' },
        { name: 'Dunhuang', cn: '敦煌', lon: 94.66, lat: 40.14, f: -111, ps: -100, pe: 800, dy: 1036, dr: 'trade_route', reg: '河西走廊' },
        { name: 'Loulan', cn: '楼兰', lon: 89.85, lat: 40.51, f: -176, ps: -150, pe: 300, dy: 500, dr: 'climate', reg: '罗布泊' },
        { name: 'Kashgar', cn: '喀什', lon: 75.99, lat: 39.47, f: -128, ps: -100, pe: 1500, dy: null, dr: null, reg: '塔里木盆地' },
        { name: 'Samarkand', cn: '撒马尔罕', lon: 66.96, lat: 39.65, f: -700, ps: -600, pe: 1200, dy: 1220, dr: 'war', reg: '河中地区' },
        { name: 'Baghdad', cn: '巴格达', lon: 44.37, lat: 33.31, f: 762, ps: 800, pe: 1100, dy: 1258, dr: 'war', reg: '美索不达米亚' },
        { name: 'Constantinople', cn: '君士坦丁堡', lon: 28.98, lat: 41.01, f: 330, ps: 400, pe: 1200, dy: 1453, dr: 'war', reg: '地中海' },
        { name: 'Merv', cn: '木鹿', lon: 62.18, lat: 37.66, f: -600, ps: -500, pe: 1100, dy: 1221, dr: 'war', reg: '河中地区' },
        { name: 'Bukhara', cn: '布哈拉', lon: 64.43, lat: 39.77, f: -500, ps: -400, pe: 1100, dy: 1220, dr: 'war', reg: '河中地区' },
        { name: 'Antioch', cn: '安条克', lon: 36.17, lat: 36.20, f: -300, ps: -250, pe: 600, dy: 637, dr: 'war', reg: '地中海' },
        { name: 'Khotan', cn: '于阗', lon: 79.93, lat: 37.11, f: -56, ps: -50, pe: 900, dy: 1006, dr: 'war', reg: '塔里木盆地' },
        { name: 'Niya', cn: '尼雅', lon: 82.70, lat: 37.09, f: -100, ps: -50, pe: 300, dy: 400, dr: 'climate', reg: '塔里木盆地' },
        { name: 'Kucha', cn: '龟兹', lon: 82.95, lat: 41.72, f: -176, ps: -150, pe: 700, dy: 800, dr: 'trade_route', reg: '塔里木盆地' },
        { name: 'Balkh', cn: '巴尔赫', lon: 66.80, lat: 36.75, f: -1500, ps: -1400, pe: 700, dy: 800, dr: 'war', reg: '河中地区' },
        { name: 'Alexandria', cn: '亚历山大', lon: 29.92, lat: 31.20, f: -331, ps: -300, pe: 600, dy: 642, dr: 'war', reg: '地中海' },
    ];
    return names.map((n, i) => ({
        id: i + 1, name: n.name, name_cn: n.cn, longitude: n.lon, latitude: n.lat,
        founded_year: n.f, prosperity_start: n.ps, prosperity_end: n.pe,
        decline_year: n.dy, decline_reason: n.dr, region: n.reg,
        description: '', population_peak: 50000, trade_volume: 700
    }));
}

function getCityStatus(city, year) {
    if (city.prosperity_start <= year && year <= city.prosperity_end) return 'prosperity';
    if (city.decline_year !== null && year >= city.decline_year) return 'decline';
    return 'transition';
}

function getStatusColor(status) {
    switch (status) {
        case 'prosperity': return '#27ae60';
        case 'transition': return '#f39c12';
        case 'decline': return '#e74c3c';
        default: return '#888';
    }
}

function addCityMarkers() {
    citiesData.forEach(city => {
        const status = getCityStatus(city, currentYear);
        const color = getStatusColor(status);

        const marker = L.circleMarker([city.latitude, city.longitude], {
            radius: 6,
            fillColor: color,
            color: '#fff',
            weight: 1,
            opacity: 0.8,
            fillOpacity: 0.85
        }).addTo(map);

        marker.bindTooltip(`${city.name_cn || city.name}`, {
            className: 'city-tooltip',
            direction: 'top',
            offset: [0, -8]
        });

        marker.on('click', () => showCityPanel(city.id));

        cityMarkers[city.id] = marker;
    });
}

function updateCityStatuses() {
    citiesData.forEach(city => {
        const marker = cityMarkers[city.id];
        if (!marker) return;

        if (city.founded_year > currentYear) {
            marker.setStyle({ fillOpacity: 0, opacity: 0 });
            return;
        }

        const status = getCityStatus(city, currentYear);
        const color = getStatusColor(status);
        const size = status === 'prosperity' ? 7 : status === 'decline' ? 5 : 6;

        marker.setStyle({
            fillColor: color,
            fillOpacity: 0.85,
            opacity: 0.8,
            radius: size
        });
    });
}

async function showCityPanel(cityId) {
    const panel = document.getElementById('city-panel');
    panel.classList.remove('hidden');

    let city = citiesData.find(c => c.id === cityId);
    let climateRecords = climateData.filter(c => c.region === city?.region);

    try {
        const res = await fetch(`${API_BASE}/cities/${cityId}/timeline`);
        if (res.ok) {
            const data = await res.json();
            city = data.city;
            climateRecords = data.climate_records;
        }
    } catch (e) {}

    if (!city) return;

    document.getElementById('panel-city-name').textContent = `${city.name_cn || ''} ${city.name}`;
    document.getElementById('panel-founded').textContent = formatYear(city.founded_year);
    document.getElementById('panel-prosperity').textContent = `${formatYear(city.prosperity_start)} — ${formatYear(city.prosperity_end)}`;
    document.getElementById('panel-decline').textContent = city.decline_year ? formatYear(city.decline_year) : '未衰落';
    document.getElementById('panel-reason').textContent = formatReason(city.decline_reason);
    document.getElementById('panel-region').textContent = city.region || '—';
    document.getElementById('panel-desc').textContent = city.description || '—';

    drawCityTimeline(city);
    drawCityClimateChart(city, climateRecords);
}

function formatYear(y) {
    if (y === null || y === undefined) return '—';
    return y < 0 ? `前${Math.abs(y)}年` : `${y}年`;
}

function formatReason(r) {
    if (!r) return '—';
    const map = { war: '战争', climate: '气候', trade_route: '贸易路线变迁' };
    return map[r] || r;
}

function drawCityTimeline(city) {
    const container = document.getElementById('panel-timeline-chart');
    container.innerHTML = '<div class="chart-title">兴衰时间线</div><canvas id="timeline-canvas" width="340" height="100"></canvas>';

    const canvas = document.getElementById('timeline-canvas');
    const c = canvas.getContext('2d');
    const w = canvas.width, h = canvas.height;
    const pad = 40, barH = 30;
    const minYear = -200, maxYear = 1500;
    const scale = (w - 2 * pad) / (maxYear - minYear);

    c.fillStyle = '#1a1a2e';
    c.fillRect(0, 0, w, h);

    c.strokeStyle = '#333';
    c.lineWidth = 0.5;
    for (let yr = -200; yr <= 1500; yr += 200) {
        const x = pad + (yr - minYear) * scale;
        c.beginPath();
        c.moveTo(x, 10);
        c.lineTo(x, h - 10);
        c.stroke();
        c.fillStyle = '#888';
        c.font = '9px sans-serif';
        c.textAlign = 'center';
        c.fillText(yr < 0 ? `${Math.abs(yr)}BC` : `${yr}AD`, x, h - 2);
    }

    const drawBar = (start, end, color, y) => {
        const x1 = pad + (start - minYear) * scale;
        const x2 = pad + (end - minYear) * scale;
        c.fillStyle = color;
        c.fillRect(x1, y, x2 - x1, barH);
    };

    drawBar(city.founded_year, city.prosperity_start, '#3498db', 15);
    drawBar(city.prosperity_start, city.prosperity_end, '#27ae60', 15);
    if (city.decline_year) {
        drawBar(city.prosperity_end, city.decline_year, '#f39c12', 15);
        drawBar(city.decline_year, Math.min(city.decline_year + 50, 1500), '#e74c3c', 15);
    } else {
        drawBar(city.prosperity_end, 1500, '#27ae60', 15);
    }

    const curX = pad + (currentYear - minYear) * scale;
    c.strokeStyle = '#fff';
    c.lineWidth = 2;
    c.setLineDash([4, 3]);
    c.beginPath();
    c.moveTo(curX, 10);
    c.lineTo(curX, h - 10);
    c.stroke();
    c.setLineDash([]);
}

function drawCityClimateChart(city, records) {
    const container = document.getElementById('panel-climate-chart');
    container.innerHTML = '<div class="chart-title">同期气候数据</div><canvas id="climate-canvas" width="340" height="120"></canvas>';

    const canvas = document.getElementById('climate-canvas');
    const c = canvas.getContext('2d');
    const w = canvas.width, h = canvas.height;
    const pad = 40;

    c.fillStyle = '#1a1a2e';
    c.fillRect(0, 0, w, h);

    if (records.length === 0) {
        c.fillStyle = '#888';
        c.font = '12px sans-serif';
        c.textAlign = 'center';
        c.fillText('无气候数据', w / 2, h / 2);
        return;
    }

    const plotW = w - 2 * pad;
    const plotH = (h - 40) / 2;

    const temps = records.filter(r => r.temperature_anomaly != null).map(r => ({
        year: (r.period_start + r.period_end) / 2,
        val: r.temperature_anomaly
    }));
    const precips = records.filter(r => r.precipitation_index != null).map(r => ({
        year: (r.period_start + r.period_end) / 2,
        val: r.precipitation_index
    }));

    drawLineChart(c, temps, pad, 10, plotW, plotH, '#e74c3c', '温度距平', -1.5, 1.5);
    drawLineChart(c, precips, pad, 10 + plotH + 10, plotW, plotH, '#3498db', '降水指数', 0, 1.6);
}

function drawLineChart(c, data, x0, y0, w, h, color, label, minV, maxV) {
    if (data.length === 0) return;

    c.strokeStyle = '#444';
    c.lineWidth = 0.5;
    c.beginPath();
    c.moveTo(x0, y0 + h / 2);
    c.lineTo(x0 + w, y0 + h / 2);
    c.stroke();

    c.strokeStyle = color;
    c.lineWidth = 2;
    c.beginPath();
    data.forEach((d, i) => {
        const x = x0 + (i / (data.length - 1 || 1)) * w;
        const y = y0 + h - ((d.val - minV) / (maxV - minV)) * h;
        if (i === 0) c.moveTo(x, y);
        else c.lineTo(x, y);
    });
    c.stroke();

    c.fillStyle = color;
    c.font = '9px sans-serif';
    c.textAlign = 'left';
    c.fillText(label, x0 + 2, y0 + 10);
}

function startArrowAnimation() {
    function animate() {
        routeAnimOffset += 0.5;
        if (routeAnimOffset > 20) routeAnimOffset = 0;
        drawTradeArrows();
        animationFrame = requestAnimationFrame(animate);
    }
    animate();
}

function drawTradeArrows() {
    if (!ctx) return;
    ctx.clearRect(0, 0, canvasOverlay.width, canvasOverlay.height);

    const activeArrows = tradeArrowsData.filter(a =>
        a.period_start <= currentYear && a.period_end >= currentYear
    );

    activeArrows.forEach(arrow => {
        const from = map.latLngToContainerPoint([arrow.from_lat, arrow.from_lon]);
        const to = map.latLngToContainerPoint([arrow.to_lat, arrow.to_lon]);

        const color = arrow.route_type === 'sea' ? 'rgba(155, 89, 182, 0.6)' : 'rgba(52, 152, 219, 0.5)';
        const vol = Math.min(arrow.trade_volume, 1000);
        const width = 1 + (vol / 1000) * 3;

        ctx.strokeStyle = color;
        ctx.lineWidth = width;
        ctx.setLineDash([8, 6]);
        ctx.lineDashOffset = -routeAnimOffset;
        ctx.beginPath();
        ctx.moveTo(from.x, from.y);
        ctx.lineTo(to.x, to.y);
        ctx.stroke();
        ctx.setLineDash([]);

        const angle = Math.atan2(to.y - from.y, to.x - from.x);
        const dist = Math.sqrt((to.x - from.x) ** 2 + (to.y - from.y) ** 2);
        if (dist < 30) return;

        const headLen = 8;
        const midX = (from.x + to.x) / 2;
        const midY = (from.y + to.y) / 2;

        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.moveTo(midX, midY);
        ctx.lineTo(midX - headLen * Math.cos(angle - 0.4), midY - headLen * Math.sin(angle - 0.4));
        ctx.lineTo(midX - headLen * Math.cos(angle + 0.4), midY - headLen * Math.sin(angle + 0.4));
        ctx.closePath();
        ctx.fill();
    });
}

async function showCoxAnalysis() {
    const modal = document.getElementById('analysis-modal');
    modal.classList.remove('hidden');

    const resultsDiv = document.getElementById('cox-results');
    resultsDiv.innerHTML = '<p style="color:#888;text-align:center;">加载中...</p>';

    try {
        const res = await fetch(`${API_BASE}/analysis/cox`);
        const data = await res.json();

        let html = `<table>
            <tr><th>变量</th><th>系数(β)</th><th>风险比(HR)</th><th>标准误</th><th>p值</th><th>95% CI</th></tr>`;

        const varNames = { temp_change: '温度变化', precip_change: '降水变化', route_change: '路线变迁', glacier_advance: '冰川前进' };

        data.cox_results.forEach(r => {
            const sig = r.p_value < 0.05 ? 'style="color:#e94560;font-weight:bold"' : '';
            html += `<tr>
                <td>${varNames[r.variable] || r.variable}</td>
                <td ${sig}>${r.coefficient.toFixed(4)}</td>
                <td ${sig}>${r.hazard_ratio.toFixed(4)}</td>
                <td>${r.std_error.toFixed(4)}</td>
                <td ${sig}>${r.p_value.toFixed(4)}</td>
                <td>[${r.confidence_interval_lower.toFixed(4)}, ${r.confidence_interval_upper.toFixed(4)}]</td>
            </tr>`;
        });
        html += '</table>';
        resultsDiv.innerHTML = html;

        drawCoxChart(data.cox_results);

        document.getElementById('cox-summary').innerHTML = `
            <p>样本量: ${data.sample_size} 个城市 | 对数偏似然: ${data.log_likelihood.toFixed(2)} | Concordance: ${data.concordance.toFixed(3)}</p>
            <p style="margin-top:6px;">风险比(HR) > 1 表示该因子增加城市衰落风险；HR < 1 表示降低风险。p < 0.05 为统计显著。</p>
        `;
    } catch (e) {
        resultsDiv.innerHTML = '<p style="color:#e74c3c;text-align:center;">分析服务暂不可用</p>';
    }
}

function drawCoxChart(results) {
    const container = document.getElementById('cox-chart');
    container.innerHTML = '<canvas id="cox-canvas" width="560" height="200"></canvas>';

    const canvas = document.getElementById('cox-canvas');
    const c = canvas.getContext('2d');
    const w = canvas.width, h = canvas.height;
    const pad = 60;

    c.fillStyle = '#1a1a2e';
    c.fillRect(0, 0, w, h);

    const varNames = { temp_change: '温度变化', precip_change: '降水变化', route_change: '路线变迁', glacier_advance: '冰川前进' };

    const barW = 80;
    const gap = (w - 2 * pad - results.length * barW) / (results.length + 1);
    const maxHR = Math.max(...results.map(r => r.hazard_ratio), 2);

    c.strokeStyle = '#666';
    c.lineWidth = 1;
    const refY = pad + (1 - 1 / maxHR) * (h - 2 * pad);
    c.setLineDash([4, 4]);
    c.beginPath();
    c.moveTo(pad, refY);
    c.lineTo(w - pad, refY);
    c.stroke();
    c.setLineDash([]);
    c.fillStyle = '#888';
    c.font = '10px sans-serif';
    c.textAlign = 'right';
    c.fillText('HR=1', pad - 5, refY + 4);

    results.forEach((r, i) => {
        const x = pad + gap + i * (barW + gap);
        const hrH = (r.hazard_ratio / maxHR) * (h - 2 * pad);
        const y = h - pad - hrH;
        const color = r.p_value < 0.05 ? '#e94560' : '#3498db';

        c.fillStyle = color;
        c.fillRect(x, y, barW, hrH);

        c.fillStyle = '#f5f5f5';
        c.font = '11px sans-serif';
        c.textAlign = 'center';
        c.fillText(varNames[r.variable] || r.variable, x + barW / 2, h - pad + 16);

        c.fillStyle = color;
        c.font = 'bold 11px sans-serif';
        c.fillText(r.hazard_ratio.toFixed(2), x + barW / 2, y - 5);
    });
}

async function showRouteShift() {
    const modal = document.getElementById('route-modal');
    modal.classList.remove('hidden');

    const shiftDiv = document.getElementById('route-shift-chart');
    shiftDiv.innerHTML = '<canvas id="shift-canvas" width="860" height="250"></canvas>';

    try {
        const res = await fetch(`${API_BASE}/analysis/route-shift`);
        const data = await res.json();
        drawRouteShiftChart(data);
        drawRouteNetworkInfo(data);
    } catch (e) {
        shiftDiv.innerHTML = '<p style="color:#e74c3c;text-align:center;">数据加载失败</p>';
    }
}

function drawRouteShiftChart(shifts) {
    const canvas = document.getElementById('shift-canvas');
    const c = canvas.getContext('2d');
    const w = canvas.width, h = canvas.height;
    const pad = 60;

    c.fillStyle = '#1a1a2e';
    c.fillRect(0, 0, w, h);

    if (shifts.length === 0) return;

    const plotW = w - 2 * pad;
    const plotH = h - 2 * pad - 20;

    const lons = shifts.map(s => s.centroid_longitude);
    const lats = shifts.map(s => s.centroid_latitude);
    const minLon = Math.min(...lons) - 5;
    const maxLon = Math.max(...lons) + 5;
    const minLat = Math.min(...lats) - 3;
    const maxLat = Math.max(...lats) + 3;

    c.strokeStyle = '#444';
    c.lineWidth = 0.5;
    c.beginPath();
    c.moveTo(pad, h - pad);
    c.lineTo(w - pad, h - pad);
    c.stroke();
    c.beginPath();
    c.moveTo(pad, pad);
    c.lineTo(pad, h - pad);
    c.stroke();

    c.fillStyle = '#888';
    c.font = '9px sans-serif';
    c.textAlign = 'center';
    shifts.forEach((s, i) => {
        const x = pad + (i / (shifts.length - 1 || 1)) * plotW;
        c.fillText(s.period, x, h - pad + 14);
    });

    shifts.forEach((s, i) => {
        const x = pad + (i / (shifts.length - 1 || 1)) * plotW;
        const y1 = pad + (1 - (s.centroid_longitude - minLon) / (maxLon - minLon)) * plotH;
        const y2 = pad + (1 - (s.centroid_latitude - minLat) / (maxLat - minLat)) * plotH;
        const size = 3 + (s.active_cities / 120) * 8;

        c.fillStyle = '#3498db';
        c.beginPath();
        c.arc(x, y1, size, 0, Math.PI * 2);
        c.fill();

        c.fillStyle = '#e94560';
        c.beginPath();
        c.arc(x, y2, size, 0, Math.PI * 2);
        c.fill();
    });

    c.fillStyle = '#3498db';
    c.font = '11px sans-serif';
    c.textAlign = 'left';
    c.fillText('● 经度重心', w - pad - 160, pad + 10);
    c.fillStyle = '#e94560';
    c.fillText('● 纬度重心', w - pad - 70, pad + 10);
}

function drawRouteNetworkInfo(shifts) {
    const div = document.getElementById('route-network-info');
    let html = '<table style="width:100%;border-collapse:collapse;font-size:12px;margin-top:12px;">';
    html += '<tr><th>时期</th><th>重心经度</th><th>重心纬度</th><th>活跃城市数</th><th>贸易总量</th><th>重心偏移(km)</th></tr>';

    shifts.forEach(s => {
        html += `<tr>
            <td>${s.period}</td>
            <td>${s.centroid_longitude.toFixed(2)}</td>
            <td>${s.centroid_latitude.toFixed(2)}</td>
            <td>${s.active_cities}</td>
            <td>${s.total_trade_volume.toLocaleString()}</td>
            <td>${s.shift_from_previous !== null ? s.shift_from_previous.toFixed(1) : '—'}</td>
        </tr>`;
    });
    html += '</table>';
    div.innerHTML = html;
}

async function showClimateOverview() {
    const modal = document.getElementById('climate-modal');
    modal.classList.remove('hidden');

    try {
        const [climateRes, summaryRes] = await Promise.all([
            fetch(`${API_BASE}/climate`),
            fetch(`${API_BASE}/climate/summary`)
        ]);
        const allClimate = await climateRes.json();
        const summary = await summaryRes.json();

        drawClimateChart(allClimate);
        drawClimateSummaryTable(summary);
    } catch (e) {
        document.getElementById('climate-chart').innerHTML = '<p style="color:#e74c3c;text-align:center;">数据加载失败</p>';
    }
}

function drawClimateChart(allClimate) {
    const container = document.getElementById('climate-chart');
    container.innerHTML = '<canvas id="all-climate-canvas" width="860" height="250"></canvas>';

    const canvas = document.getElementById('all-climate-canvas');
    const c = canvas.getContext('2d');
    const w = canvas.width, h = canvas.height;
    const pad = 60;

    c.fillStyle = '#1a1a2e';
    c.fillRect(0, 0, w, h);

    const regions = ['关中', '河西走廊', '塔里木盆地', '河中地区', '美索不达米亚'];
    const colors = ['#e74c3c', '#f39c12', '#27ae60', '#3498db', '#9b59b6'];
    const plotW = w - 2 * pad;
    const plotH = h - 2 * pad;
    const minYear = -200, maxYear = 1500;
    const minT = -1.5, maxT = 1.5;

    c.strokeStyle = '#444';
    c.lineWidth = 0.5;
    [0, -0.5, 0.5, -1.0, 1.0].forEach(v => {
        const y = pad + (1 - (v - minT) / (maxT - minT)) * plotH;
        c.beginPath();
        c.moveTo(pad, y);
        c.lineTo(w - pad, y);
        c.stroke();
        c.fillStyle = '#666';
        c.font = '9px sans-serif';
        c.textAlign = 'right';
        c.fillText(`${v > 0 ? '+' : ''}${v}°C`, pad - 5, y + 3);
    });

    const xScale = (yr) => pad + ((yr - minYear) / (maxYear - minYear)) * plotW;
    const yScale = (v) => pad + (1 - (v - minT) / (maxT - minT)) * plotH;

    regions.forEach((region, ri) => {
        const records = allClimate.filter(r => r.region === region && r.temperature_anomaly != null);
        if (records.length === 0) return;

        c.strokeStyle = colors[ri];
        c.lineWidth = 1.5;
        c.beginPath();
        records.forEach((r, i) => {
            const x = xScale((r.period_start + r.period_end) / 2);
            const y = yScale(r.temperature_anomaly);
            if (i === 0) c.moveTo(x, y);
            else c.lineTo(x, y);
        });
        c.stroke();
    });

    const curX = xScale(currentYear);
    c.strokeStyle = 'rgba(255,255,255,0.6)';
    c.lineWidth = 2;
    c.setLineDash([4, 3]);
    c.beginPath();
    c.moveTo(curX, pad);
    c.lineTo(curX, h - pad);
    c.stroke();
    c.setLineDash([]);

    regions.forEach((region, ri) => {
        c.fillStyle = colors[ri];
        c.font = '10px sans-serif';
        c.textAlign = 'left';
        c.fillRect(w - pad - 150, pad + ri * 16, 10, 10);
        c.fillText(region, w - pad - 135, pad + ri * 16 + 9);
    });
}

function drawClimateSummaryTable(summary) {
    const div = document.getElementById('climate-summary-table');
    let html = '<table>';
    html += '<tr><th>区域</th><th>平均温度距平</th><th>平均降水指数</th><th>冰川前进占比</th></tr>';
    summary.forEach(s => {
        html += `<tr>
            <td>${s.region}</td>
            <td>${s.avg_temperature_anomaly > 0 ? '+' : ''}${s.avg_temperature_anomaly.toFixed(2)}°C</td>
            <td>${s.avg_precipitation_index.toFixed(2)}</td>
            <td>${s.glacier_advance_percent.toFixed(1)}%</td>
        </tr>`;
    });
    html += '</table>';
    div.innerHTML = html;
}
