(function(global) {
    'use strict';

    const SilkroadMap = {
        map: null,
        glCanvas: null,
        gl: null,
        shaderProgram: null,
        cityCanvasRenderer: null,
        cityLayer: null,
        citiesData: [],
        tradeArrowsData: [],
        currentYear: 600,
        isPlaying: false,
        playInterval: null,
        animationFrame: null,
        routeAnimOffset: 0,
        lastRenderTime: 0,
        FRAME_INTERVAL: 33,
        glBuffers: { arrows: null, arrowCount: 0, heads: null, headCount: 0 },
        onCityClick: null,
        onYearChange: null,

        init: function(apiBase) {
            this.API_BASE = apiBase;
            this.initMap();
            this.initWebGL();
            this.initTimeline();
            this.startArrowAnimation();
        },

        initMap: function() {
            this.cityCanvasRenderer = L.canvas({ padding: 0.5 });

            this.map = L.map('map', {
                center: [38, 65],
                zoom: 4,
                minZoom: 3,
                maxZoom: 8,
                zoomControl: true,
                renderer: this.cityCanvasRenderer,
            });

            L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', {
                attribution: '&copy; OpenStreetMap &copy; CARTO',
                subdomains: 'abcd',
                maxZoom: 19
            }).addTo(this.map);

            const legend = L.control({ position: 'bottomleft' });
            const self = this;
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
            legend.addTo(this.map);

            this.map.on('moveend zoomend', function() {
                self.updateCityStatuses();
                self.updateWebGLArrows();
            });
        },

        initWebGL: function() {
            this.glCanvas = document.getElementById('gl-overlay');
            this.gl = this.glCanvas.getContext('webgl', { alpha: true, premultipliedAlpha: false, antialias: true });
            if (!this.gl) {
                this.gl = this.glCanvas.getContext('experimental-webgl', { alpha: true });
            }

            this.resizeGLCanvas();
            const self = this;
            window.addEventListener('resize', function() {
                self.resizeGLCanvas();
                if (self.gl) self.updateWebGLArrows();
            });

            if (this.gl) {
                this.initShaders();
            }
        },

        resizeGLCanvas: function() {
            const container = document.getElementById('map-container');
            const w = container.clientWidth;
            const h = container.clientHeight;
            const dpr = window.devicePixelRatio || 1;
            this.glCanvas.width = w * dpr;
            this.glCanvas.height = h * dpr;
            this.glCanvas.style.width = w + 'px';
            this.glCanvas.style.height = h + 'px';
            if (this.gl) {
                this.gl.viewport(0, 0, this.glCanvas.width, this.glCanvas.height);
            }
        },

        initShaders: function() {
            const gl = this.gl;
            const vsSource = `
                attribute vec2 aPosition;
                attribute vec4 aColor;
                attribute float aDashPhase;
                uniform vec2 uResolution;
                uniform float uDashOffset;
                varying vec4 vColor;
                varying float vDashPhase;
                void main() {
                    vec2 clipSpace = (aPosition / uResolution) * 2.0 - 1.0;
                    clipSpace.y *= -1.0;
                    gl_Position = vec4(clipSpace, 0.0, 1.0);
                    vColor = aColor;
                    vDashPhase = aDashPhase + uDashOffset;
                }
            `;

            const fsSource = `
                precision mediump float;
                varying vec4 vColor;
                varying float vDashPhase;
                void main() {
                    float dash = mod(vDashPhase, 14.0);
                    if (dash > 8.0) discard;
                    gl_FragColor = vColor;
                }
            `;

            const vs = this.compileShader(gl.VERTEX_SHADER, vsSource);
            const fs = this.compileShader(gl.FRAGMENT_SHADER, fsSource);

            this.shaderProgram = gl.createProgram();
            gl.attachShader(this.shaderProgram, vs);
            gl.attachShader(this.shaderProgram, fs);
            gl.linkProgram(this.shaderProgram);

            if (!gl.getProgramParameter(this.shaderProgram, gl.LINK_STATUS)) {
                console.error('Shader link failed:', gl.getProgramInfoLog(this.shaderProgram));
                this.shaderProgram = null;
            }
        },

        compileShader: function(type, source) {
            const gl = this.gl;
            const shader = gl.createShader(type);
            gl.shaderSource(shader, source);
            gl.compileShader(shader);
            if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
                console.error('Shader compile failed:', gl.getShaderInfoLog(shader));
            }
            return shader;
        },

        initTimeline: function() {
            const slider = document.getElementById('timeline-slider');
            const self = this;

            slider.addEventListener('input', function(e) {
                self.setYear(parseInt(e.target.value));
            });

            document.getElementById('btn-play').addEventListener('click', function() {
                self.play();
            });

            document.getElementById('btn-pause').addEventListener('click', function() {
                self.pause();
            });

            document.getElementById('btn-reset').addEventListener('click', function() {
                self.reset();
            });

            this.updateYearDisplay();
        },

        setYear: function(year) {
            this.currentYear = year;
            document.getElementById('timeline-slider').value = year;
            this.updateYearDisplay();
            this.updateCityStatuses();
            this.updateWebGLArrows();
            if (this.onYearChange) {
                this.onYearChange(year);
            }
        },

        play: function() {
            if (!this.isPlaying) {
                this.isPlaying = true;
                const self = this;
                this.playInterval = setInterval(function() {
                    let y = self.currentYear + 5;
                    if (y > 1500) y = -200;
                    self.setYear(y);
                    document.getElementById('timeline-slider').value = y;
                }, 100);
            }
        },

        pause: function() {
            this.isPlaying = false;
            if (this.playInterval) {
                clearInterval(this.playInterval);
                this.playInterval = null;
            }
        },

        reset: function() {
            this.setYear(-200);
            document.getElementById('timeline-slider').value = -200;
        },

        updateYearDisplay: function() {
            const yearDisplay = document.getElementById('current-year');
            if (this.currentYear < 0) {
                yearDisplay.textContent = '前 ' + Math.abs(this.currentYear) + ' 年';
            } else {
                yearDisplay.textContent = '公元 ' + this.currentYear + ' 年';
            }
        },

        loadCities: function() {
            const self = this;
            return fetch(this.API_BASE + '/cities')
                .then(function(r) { return r.json(); })
                .then(function(data) {
                    self.citiesData = data;
                    self.addCityMarkers();
                    self.updateCityStatuses();
                    return data;
                });
        },

        loadArrows: function() {
            const self = this;
            return fetch(this.API_BASE + '/trade/arrows')
                .then(function(r) { return r.json(); })
                .then(function(data) {
                    self.tradeArrowsData = data;
                    self.updateWebGLArrows();
                    return data;
                });
        },

        getCityStatus: function(city, year) {
            if (city.prosperity_start <= year && year <= city.prosperity_end) return 'prosperity';
            if (city.decline_year !== null && year >= city.decline_year) return 'decline';
            return 'transition';
        },

        getStatusColor: function(status) {
            switch (status) {
                case 'prosperity': return '#27ae60';
                case 'transition': return '#f39c12';
                case 'decline': return '#e74c3c';
                default: return '#888';
            }
        },

        addCityMarkers: function() {
            if (this.cityLayer) {
                this.map.removeLayer(this.cityLayer);
            }

            const geojson = {
                type: 'FeatureCollection',
                features: this.citiesData.map(function(city) {
                    return {
                        type: 'Feature',
                        properties: { id: city.id, status: 'prosperity' },
                        geometry: { type: 'Point', coordinates: [city.longitude, city.latitude] }
                    };
                })
            };

            const self = this;
            this.cityLayer = L.geoJSON(geojson, {
                renderer: this.cityCanvasRenderer,
                pointToLayer: function(feature, latlng) {
                    const color = '#27ae60';
                    return L.circleMarker(latlng, {
                        radius: 6,
                        fillColor: color,
                        color: '#fff',
                        weight: 1,
                        opacity: 0.8,
                        fillOpacity: 0.85
                    });
                },
                onEachFeature: function(feature, layer) {
                    const cityId = feature.properties.id;
                    const city = self.citiesData.find(function(c) { return c.id === cityId; });
                    if (city) {
                        layer.bindTooltip(city.name_cn || city.name, {
                            className: 'city-tooltip',
                            direction: 'top',
                            offset: [0, -8]
                        });
                        layer.on('click', function() {
                            if (self.onCityClick) {
                                self.onCityClick(cityId);
                            }
                        });
                    }
                }
            }).addTo(this.map);
        },

        updateCityStatuses: function() {
            if (!this.cityLayer) return;
            const bounds = this.map.getBounds();
            const zoom = this.map.getZoom();
            const self = this;

            this.cityLayer.eachLayer(function(layer) {
                const feature = layer.feature;
                const cityId = feature.properties.id;
                const city = self.citiesData.find(function(c) { return c.id === cityId; });
                if (!city) return;

                if (city.founded_year > self.currentYear) {
                    layer.setStyle({ fillOpacity: 0, opacity: 0 });
                    return;
                }

                const latlng = L.latLng(city.latitude, city.longitude);
                if (!bounds.contains(latlng)) {
                    layer.setStyle({ fillOpacity: 0, opacity: 0 });
                    return;
                }

                const status = self.getCityStatus(city, self.currentYear);
                const color = self.getStatusColor(status);
                const size = status === 'prosperity' ? 7 : status === 'decline' ? 5 : 6;
                const scaleFactor = Math.max(0.6, Math.min(1.5, zoom / 4));

                layer.setStyle({
                    fillColor: color,
                    fillOpacity: 0.85,
                    opacity: 0.8,
                    radius: size * scaleFactor
                });
            });
        },

        startArrowAnimation: function() {
            const self = this;
            function animate(timestamp) {
                if (timestamp - self.lastRenderTime < self.FRAME_INTERVAL) {
                    self.animationFrame = requestAnimationFrame(animate);
                    return;
                }
                self.lastRenderTime = timestamp;
                self.routeAnimOffset += 0.5;
                if (self.routeAnimOffset > 20) self.routeAnimOffset = 0;
                self.renderWebGLArrows();
                self.animationFrame = requestAnimationFrame(animate);
            }
            this.animationFrame = requestAnimationFrame(animate);
        },

        updateWebGLArrows: function() {
            if (!this.gl || !this.shaderProgram) return;

            const bounds = this.map.getBounds();
            const dpr = window.devicePixelRatio || 1;
            const self = this;

            const activeArrows = this.tradeArrowsData.filter(function(a) {
                if (a.period_start > self.currentYear || a.period_end < self.currentYear) return false;
                const fromLatlng = L.latLng(a.from_lat, a.from_lon);
                const toLatlng = L.latLng(a.to_lat, a.to_lon);
                return bounds.contains(fromLatlng) || bounds.contains(toLatlng) ||
                    bounds.intersects(L.latLngBounds([fromLatlng, toLatlng]));
            });

            const arrowVertices = [];
            const headVertices = [];

            for (const arrow of activeArrows) {
                const from = this.map.latLngToContainerPoint([arrow.from_lat, arrow.from_lon]);
                const to = this.map.latLngToContainerPoint([arrow.to_lat, arrow.to_lon]);

                const dist = Math.sqrt((to.x - from.x) ** 2 + (to.y - from.y) ** 2);
                if (dist < 20) continue;

                const fx = from.x * dpr;
                const fy = from.y * dpr;
                const tx = to.x * dpr;
                const ty = to.y * dpr;

                const isSea = arrow.route_type === 'sea';
                const r = isSea ? 155 : 52;
                const g = isSea ? 89 : 152;
                const b = isSea ? 182 : 219;
                const alpha = isSea ? 0.6 : 0.5;
                const vol = Math.min(arrow.trade_volume, 1000);
                const lineHalfWidth = (0.5 + (vol / 1000) * 1.5) * dpr;

                const dx = tx - fx;
                const dy = ty - fy;
                const len = Math.sqrt(dx * dx + dy * dy);
                const nx = -dy / len * lineHalfWidth;
                const ny = dx / len * lineHalfWidth;

                const totalLen = len / dpr;

                const v = [
                    fx + nx, fy + ny, r / 255, g / 255, b / 255, alpha, 0,
                    fx - nx, fy - ny, r / 255, g / 255, b / 255, alpha, 0,
                    tx + nx, ty + ny, r / 255, g / 255, b / 255, alpha, totalLen,
                    fx - nx, fy - ny, r / 255, g / 255, b / 255, alpha, 0,
                    tx + nx, ty + ny, r / 255, g / 255, b / 255, alpha, totalLen,
                    tx - nx, ty + ny, r / 255, g / 255, b / 255, alpha, totalLen
                ];
                for (const n of v) arrowVertices.push(n);

                if (dist > 50) {
                    const angle = Math.atan2(ty - fy, tx - fx);
                    const midX = (fx + tx) / 2;
                    const midY = (fy + ty) / 2;
                    const headLen = 8 * dpr;

                    const p1x = midX;
                    const p1y = midY;
                    const p2x = midX - headLen * Math.cos(angle - 0.4);
                    const p2y = midY - headLen * Math.sin(angle - 0.4);
                    const p3x = midX - headLen * Math.cos(angle + 0.4);
                    const p3y = midY - headLen * Math.sin(angle + 0.4);

                    headVertices.push(
                        p1x, p1y, r / 255, g / 255, b / 255, alpha + 0.2, 0,
                        p2x, p2y, r / 255, g / 255, b / 255, alpha + 0.2, 0,
                        p3x, p3y, r / 255, g / 255, b / 255, alpha + 0.2, 0
                    );
                }
            }

            const gl = this.gl;
            if (arrowVertices.length > 0) {
                if (!this.glBuffers.arrows) this.glBuffers.arrows = gl.createBuffer();
                gl.bindBuffer(gl.ARRAY_BUFFER, this.glBuffers.arrows);
                gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(arrowVertices), gl.DYNAMIC_DRAW);
                this.glBuffers.arrowCount = arrowVertices.length / 7;
            } else {
                this.glBuffers.arrowCount = 0;
            }

            if (headVertices.length > 0) {
                if (!this.glBuffers.heads) this.glBuffers.heads = gl.createBuffer();
                gl.bindBuffer(gl.ARRAY_BUFFER, this.glBuffers.heads);
                gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(headVertices), gl.DYNAMIC_DRAW);
                this.glBuffers.headCount = headVertices.length / 7;
            } else {
                this.glBuffers.headCount = 0;
            }
        },

        renderWebGLArrows: function() {
            if (!this.gl || !this.shaderProgram) return;
            const gl = this.gl;

            gl.clearColor(0, 0, 0, 0);
            gl.clear(gl.COLOR_BUFFER_BIT);
            gl.enable(gl.BLEND);
            gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

            gl.useProgram(this.shaderProgram);

            const uResolution = gl.getUniformLocation(this.shaderProgram, 'uResolution');
            gl.uniform2f(uResolution, this.glCanvas.width, this.glCanvas.height);

            const uDashOffset = gl.getUniformLocation(this.shaderProgram, 'uDashOffset');
            gl.uniform1f(uDashOffset, this.routeAnimOffset);

            const aPosition = gl.getAttribLocation(this.shaderProgram, 'aPosition');
            const aColor = gl.getAttribLocation(this.shaderProgram, 'aColor');
            const aDashPhase = gl.getAttribLocation(this.shaderProgram, 'aDashPhase');

            const STRIDE = 7 * 4;

            if (this.glBuffers.arrowCount > 0) {
                gl.bindBuffer(gl.ARRAY_BUFFER, this.glBuffers.arrows);
                gl.enableVertexAttribArray(aPosition);
                gl.vertexAttribPointer(aPosition, 2, gl.FLOAT, false, STRIDE, 0);
                gl.enableVertexAttribArray(aColor);
                gl.vertexAttribPointer(aColor, 4, gl.FLOAT, false, STRIDE, 8);
                gl.enableVertexAttribArray(aDashPhase);
                gl.vertexAttribPointer(aDashPhase, 1, gl.FLOAT, false, STRIDE, 24);
                gl.drawArrays(gl.TRIANGLES, 0, this.glBuffers.arrowCount);
            }

            if (this.glBuffers.headCount > 0) {
                gl.bindBuffer(gl.ARRAY_BUFFER, this.glBuffers.heads);
                gl.enableVertexAttribArray(aPosition);
                gl.vertexAttribPointer(aPosition, 2, gl.FLOAT, false, STRIDE, 0);
                gl.enableVertexAttribArray(aColor);
                gl.vertexAttribPointer(aColor, 4, gl.FLOAT, false, STRIDE, 8);
                gl.enableVertexAttribArray(aDashPhase);
                gl.vertexAttribPointer(aDashPhase, 1, gl.FLOAT, false, STRIDE, 24);
                gl.drawArrays(gl.TRIANGLES, 0, this.glBuffers.headCount);
            }
        },

        getCityById: function(id) {
            return this.citiesData.find(function(c) { return c.id === id; });
        }
    };

    global.SilkroadMap = SilkroadMap;

})(window);
