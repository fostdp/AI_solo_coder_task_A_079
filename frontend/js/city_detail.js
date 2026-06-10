(function(global) {
    'use strict';

    const CityDetail = {
        API_BASE: '/api',
        currentCityId: null,

        init: function(apiBase) {
            this.API_BASE = apiBase;
            this.initModals();
        },

        initModals: function() {
            const self = this;

            document.querySelectorAll('.modal-close').forEach(function(btn) {
                btn.addEventListener('click', function() {
                    btn.closest('.modal').classList.add('hidden');
                });
            });

            document.querySelectorAll('.modal').forEach(function(modal) {
                modal.addEventListener('click', function(e) {
                    if (e.target === modal) modal.classList.add('hidden');
                });
            });

            document.getElementById('panel-close').addEventListener('click', function() {
                document.getElementById('city-panel').classList.add('hidden');
            });

            document.getElementById('btn-analysis').addEventListener('click', function() {
                self.showCoxAnalysis();
            });
            document.getElementById('btn-route-shift').addEventListener('click', function() {
                self.showRouteShift();
            });
            document.getElementById('btn-climate').addEventListener('click', function() {
                self.showClimateOverview();
            });
        },

        showCityPanel: function(cityId) {
            this.currentCityId = cityId;
            const panel = document.getElementById('city-panel');
            panel.classList.remove('hidden');

            const self = this;
            fetch(this.API_BASE + '/cities/' + cityId + '/timeline')
                .then(function(res) {
                    if (!res.ok) throw new Error('Failed to load');
                    return res.json();
                })
                .then(function(data) {
                    self.renderCityInfo(data.city);
                    self.drawCityTimeline(data.city);
                    self.drawCityClimateChart(data.city, data.climate_records);
                })
                .catch(function() {
                    const city = global.SilkroadMap ? global.SilkroadMap.getCityById(cityId) : null;
                    if (city) {
                        self.renderCityInfo(city);
                        self.drawCityTimeline(city);
                        self.drawCityClimateChart(city, []);
                    }
                });
        },

        renderCityInfo: function(city) {
            if (!city) return;
            document.getElementById('panel-city-name').textContent =
                (city.name_cn || '') + ' ' + (city.name || '');
            document.getElementById('panel-founded').textContent = this.formatYear(city.founded_year);
            document.getElementById('panel-prosperity').textContent =
                this.formatYear(city.prosperity_start) + ' — ' + this.formatYear(city.prosperity_end);
            document.getElementById('panel-decline').textContent = city.decline_year ? this.formatYear(city.decline_year) : '未衰落';
            document.getElementById('panel-reason').textContent = this.formatReason(city.decline_reason);
            document.getElementById('panel-region').textContent = city.region || '—';
            document.getElementById('panel-desc').textContent = city.description || '—';
        },

        formatYear: function(y) {
            if (y === null || y === undefined) return '—';
            return y < 0 ? '前' + Math.abs(y) + '年' : y + '年';
        },

        formatReason: function(r) {
            if (!r) return '—';
            const map = { war: '战争', climate: '气候', trade_route: '贸易路线变迁' };
            return map[r] || r;
        },

        drawCityTimeline: function(city) {
            const container = document.getElementById('panel-timeline-chart');
            container.innerHTML =
                '<div class="chart-title">兴衰时间线</div>' +
                '<canvas id="timeline-canvas" width="340" height="100"></canvas>';

            const canvas = document.getElementById('timeline-canvas');
            const c = canvas.getContext('2d');
            const w = canvas.width, h = canvas.height;
            const pad = 40, barH = 30;
            const minYear = -200, maxYear = 1500;
            const scale = (w - 2 * pad) / (maxYear - minYear);
            const currentYear = global.SilkroadMap ? global.SilkroadMap.currentYear : 600;

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
                c.fillText(yr < 0 ? Math.abs(yr) + 'BC' : yr + 'AD', x, h - 2);
            }

            const drawBar = function(start, end, color, y) {
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
        },

        drawCityClimateChart: function(city, records) {
            const container = document.getElementById('panel-climate-chart');
            container.innerHTML =
                '<div class="chart-title">同期气候数据</div>' +
                '<canvas id="climate-canvas" width="340" height="120"></canvas>';

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

            const temps = records.filter(function(r) { return r.temperature_anomaly != null; })
                .map(function(r) {
                    return { year: (r.period_start + r.period_end) / 2, val: r.temperature_anomaly };
                });
            const precips = records.filter(function(r) { return r.precipitation_index != null; })
                .map(function(r) {
                    return { year: (r.period_start + r.period_end) / 2, val: r.precipitation_index };
                });

            this.drawLineChart(c, temps, pad, 10, plotW, plotH, '#e74c3c', '温度距平', -1.5, 1.5);
            this.drawLineChart(c, precips, pad, 10 + plotH + 10, plotW, plotH, '#3498db', '降水指数', 0, 1.6);
        },

        drawLineChart: function(c, data, x0, y0, w, h, color, label, minV, maxV) {
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
            data.forEach(function(d, i) {
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
        },

        showCoxAnalysis: function() {
            const modal = document.getElementById('analysis-modal');
            modal.classList.remove('hidden');

            const resultsDiv = document.getElementById('cox-results');
            resultsDiv.innerHTML = '<p style="color:#888;text-align:center;">加载中...</p>';

            const self = this;
            fetch(this.API_BASE + '/analysis/cox')
                .then(function(res) { return res.json(); })
                .then(function(data) {
                    self.renderCoxResults(data);
                    self.drawCoxChart(data.cox_results);
                })
                .catch(function() {
                    resultsDiv.innerHTML = '<p style="color:#e74c3c;text-align:center;">分析服务暂不可用</p>';
                });
        },

        renderCoxResults: function(data) {
            const resultsDiv = document.getElementById('cox-results');
            const varNames = {
                temp_anomaly: '温度距平',
                precip_index: '降水指数',
                temp_change: '温度变化',
                precip_change: '降水变化',
                route_change: '路线变迁',
                glacier_advance: '冰川前进'
            };

            let html = '<table><tr><th>变量</th><th>系数(β)</th><th>风险比(HR)</th><th>标准误</th><th>p值</th><th>95% CI</th></tr>';

            data.cox_results.forEach(function(r) {
                const sig = r.p_value < 0.05 ? 'style="color:#e94560;font-weight:bold"' : '';
                html += '<tr>' +
                    '<td>' + (varNames[r.variable] || r.variable) + '</td>' +
                    '<td ' + sig + '>' + r.coefficient.toFixed(4) + '</td>' +
                    '<td ' + sig + '>' + r.hazard_ratio.toFixed(4) + '</td>' +
                    '<td>' + r.std_error.toFixed(4) + '</td>' +
                    '<td ' + sig + '>' + r.p_value.toFixed(4) + '</td>' +
                    '<td>[' + r.confidence_interval_lower.toFixed(4) + ', ' + r.confidence_interval_upper.toFixed(4) + ']</td>' +
                    '</tr>';
            });
            html += '</table>';
            resultsDiv.innerHTML = html;

            document.getElementById('cox-summary').innerHTML =
                '<p>样本量: ' + data.sample_size + ' 个观测区间 (时变Cox模型) | 对数偏似然: ' +
                data.log_likelihood.toFixed(2) + ' | Concordance: ' + data.concordance.toFixed(3) + '</p>' +
                '<p style="margin-top:6px;">时变协变量Cox模型：每个城市的观测被拆分为200年时间区间，每区间独立提取气候协变量。' +
                '风险比(HR) &gt; 1 表示该因子增加城市衰落风险；HR &lt; 1 表示降低风险。p &lt; 0.05 为统计显著。</p>';
        },

        drawCoxChart: function(results) {
            const container = document.getElementById('cox-chart');
            container.innerHTML = '<canvas id="cox-canvas" width="560" height="200"></canvas>';

            const canvas = document.getElementById('cox-canvas');
            const c = canvas.getContext('2d');
            const w = canvas.width, h = canvas.height;
            const pad = 60;

            c.fillStyle = '#1a1a2e';
            c.fillRect(0, 0, w, h);

            const varNames = {
                temp_anomaly: '温度距平', precip_index: '降水指数',
                temp_change: '温度变化', precip_change: '降水变化',
                route_change: '路线变迁', glacier_advance: '冰川前进'
            };

            const barW = results.length <= 4 ? 80 : 60;
            const gap = (w - 2 * pad - results.length * barW) / (results.length + 1);
            const maxHR = Math.max.apply(null, results.map(function(r) { return r.hazard_ratio; }).concat([2]));

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

            results.forEach(function(r, i) {
                const x = pad + gap + i * (barW + gap);
                const hrH = (r.hazard_ratio / maxHR) * (h - 2 * pad);
                const y = h - pad - hrH;
                const color = r.p_value < 0.05 ? '#e94560' : '#3498db';

                c.fillStyle = color;
                c.fillRect(x, y, barW, hrH);

                c.fillStyle = '#f5f5f5';
                c.font = results.length <= 4 ? '11px sans-serif' : '9px sans-serif';
                c.textAlign = 'center';
                c.fillText(varNames[r.variable] || r.variable, x + barW / 2, h - pad + 16);

                c.fillStyle = color;
                c.font = 'bold 11px sans-serif';
                c.fillText(r.hazard_ratio.toFixed(2), x + barW / 2, y - 5);
            });
        },

        showRouteShift: function() {
            const modal = document.getElementById('route-modal');
            modal.classList.remove('hidden');

            const shiftDiv = document.getElementById('route-shift-chart');
            shiftDiv.innerHTML = '<canvas id="shift-canvas" width="860" height="250"></canvas>';

            const self = this;
            fetch(this.API_BASE + '/analysis/route-shift')
                .then(function(res) { return res.json(); })
                .then(function(data) {
                    self.drawRouteShiftChart(data);
                    self.drawRouteNetworkInfo(data);
                })
                .catch(function() {
                    shiftDiv.innerHTML = '<p style="color:#e74c3c;text-align:center;">数据加载失败</p>';
                });
        },

        drawRouteShiftChart: function(shifts) {
            const canvas = document.getElementById('shift-canvas');
            const c = canvas.getContext('2d');
            const w = canvas.width, h = canvas.height;
            const pad = 60;

            c.fillStyle = '#1a1a2e';
            c.fillRect(0, 0, w, h);

            if (shifts.length === 0) return;

            const plotW = w - 2 * pad;
            const plotH = h - 2 * pad - 20;

            const lons = shifts.map(function(s) { return s.centroid_longitude; });
            const lats = shifts.map(function(s) { return s.centroid_latitude; });
            const minLon = Math.min.apply(null, lons) - 5;
            const maxLon = Math.max.apply(null, lons) + 5;
            const minLat = Math.min.apply(null, lats) - 3;
            const maxLat = Math.max.apply(null, lats) + 3;

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
            shifts.forEach(function(s, i) {
                const x = pad + (i / (shifts.length - 1 || 1)) * plotW;
                c.fillText(s.period, x, h - pad + 14);
            });

            const drawConnectedPoints = function(values, minV, maxV, color) {
                c.strokeStyle = color;
                c.lineWidth = 1.5;
                c.beginPath();
                shifts.forEach(function(s, i) {
                    const x = pad + (i / (shifts.length - 1 || 1)) * plotW;
                    const y = pad + (1 - (values[i] - minV) / (maxV - minV)) * plotH;
                    if (i === 0) c.moveTo(x, y);
                    else c.lineTo(x, y);
                });
                c.stroke();

                shifts.forEach(function(s, i) {
                    const x = pad + (i / (shifts.length - 1 || 1)) * plotW;
                    const y = pad + (1 - (values[i] - minV) / (maxV - minV)) * plotH;
                    const size = 3 + (s.active_cities / 120) * 8;
                    c.fillStyle = color;
                    c.beginPath();
                    c.arc(x, y, size, 0, Math.PI * 2);
                    c.fill();
                });
            };

            drawConnectedPoints(lons, minLon, maxLon, '#3498db');
            drawConnectedPoints(lats, minLat, maxLat, '#e94560');

            c.fillStyle = '#3498db';
            c.font = '11px sans-serif';
            c.textAlign = 'left';
            c.fillText('● 经度重心', w - pad - 160, pad + 10);
            c.fillStyle = '#e94560';
            c.fillText('● 纬度重心', w - pad - 70, pad + 10);
        },

        drawRouteNetworkInfo: function(shifts) {
            const div = document.getElementById('route-network-info');
            let html = '<table style="width:100%;border-collapse:collapse;font-size:12px;margin-top:12px;">' +
                '<tr><th>时期</th><th>重心经度</th><th>重心纬度</th><th>活跃城市数</th><th>贸易总量</th><th>重心偏移(km)</th></tr>';

            shifts.forEach(function(s) {
                html += '<tr>' +
                    '<td>' + s.period + '</td>' +
                    '<td>' + s.centroid_longitude.toFixed(2) + '</td>' +
                    '<td>' + s.centroid_latitude.toFixed(2) + '</td>' +
                    '<td>' + s.active_cities + '</td>' +
                    '<td>' + s.total_trade_volume.toLocaleString() + '</td>' +
                    '<td>' + (s.shift_from_previous !== null ? s.shift_from_previous.toFixed(1) : '—') + '</td>' +
                    '</tr>';
            });
            html += '</table>';
            div.innerHTML = html;
        },

        showClimateOverview: function() {
            const modal = document.getElementById('climate-modal');
            modal.classList.remove('hidden');

            const self = this;
            Promise.all([
                fetch(this.API_BASE + '/climate').then(function(r) { return r.json(); }),
                fetch(this.API_BASE + '/climate/summary').then(function(r) { return r.json(); })
            ]).then(function(results) {
                self.drawClimateChart(results[0]);
                self.drawClimateSummaryTable(results[1]);
            }).catch(function() {
                document.getElementById('climate-chart').innerHTML =
                    '<p style="color:#e74c3c;text-align:center;">数据加载失败</p>';
            });
        },

        drawClimateChart: function(allClimate) {
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
            const currentYear = global.SilkroadMap ? global.SilkroadMap.currentYear : 600;

            c.strokeStyle = '#444';
            c.lineWidth = 0.5;
            [0, -0.5, 0.5, -1.0, 1.0].forEach(function(v) {
                const y = pad + (1 - (v - minT) / (maxT - minT)) * plotH;
                c.beginPath();
                c.moveTo(pad, y);
                c.lineTo(w - pad, y);
                c.stroke();
                c.fillStyle = '#666';
                c.font = '9px sans-serif';
                c.textAlign = 'right';
                c.fillText((v > 0 ? '+' : '') + v + '°C', pad - 5, y + 3);
            });

            const xScale = function(yr) { return pad + ((yr - minYear) / (maxYear - minYear)) * plotW; };
            const yScale = function(v) { return pad + (1 - (v - minT) / (maxT - minT)) * plotH; };

            regions.forEach(function(region, ri) {
                const records = allClimate.filter(function(r) {
                    return r.region === region && r.temperature_anomaly != null;
                });
                if (records.length === 0) return;

                c.strokeStyle = colors[ri];
                c.lineWidth = 1.5;
                c.beginPath();
                records.forEach(function(r, i) {
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

            regions.forEach(function(region, ri) {
                c.fillStyle = colors[ri];
                c.font = '10px sans-serif';
                c.textAlign = 'left';
                c.fillRect(w - pad - 150, pad + ri * 16, 10, 10);
                c.fillText(region, w - pad - 135, pad + ri * 16 + 9);
            });
        },

        drawClimateSummaryTable: function(summary) {
            const div = document.getElementById('climate-summary-table');
            let html = '<table>';
            html += '<tr><th>区域</th><th>平均温度距平</th><th>平均降水指数</th><th>冰川前进占比</th></tr>';
            summary.forEach(function(s) {
                html += '<tr>' +
                    '<td>' + s.region + '</td>' +
                    '<td>' + (s.avg_temperature_anomaly > 0 ? '+' : '') + s.avg_temperature_anomaly.toFixed(2) + '°C</td>' +
                    '<td>' + s.avg_precipitation_index.toFixed(2) + '</td>' +
                    '<td>' + s.glacier_advance_percent.toFixed(1) + '%</td>' +
                    '</tr>';
            });
            html += '</table>';
            div.innerHTML = html;
        }
    };

    global.CityDetail = CityDetail;

})(window);
