-- =========================================
-- 丝绸之路分析系统 - 数据库初始化脚本
-- PostgreSQL 15+ / PostGIS 3.4+
-- =========================================

-- 启用PostGIS扩展
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

-- =========================================
-- 性能参数配置（会话级）
-- =========================================
SET work_mem = '8MB';
SET maintenance_work_mem = '64MB';
SET random_page_cost = 2.0;
SET effective_cache_size = '768MB';
SET shared_buffers = '256MB';

-- =========================================
-- 清理旧表
-- =========================================
DROP TABLE IF EXISTS trade_connections;
DROP TABLE IF EXISTS climate_data;
DROP TABLE IF EXISTS cities;

-- =========================================
-- 城市表
-- =========================================
CREATE TABLE cities (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    name_cn VARCHAR(100),
    longitude DOUBLE PRECISION NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    geom GEOMETRY(Point, 4326),
    founded_year INTEGER NOT NULL,
    prosperity_start INTEGER NOT NULL,
    prosperity_end INTEGER NOT NULL,
    decline_year INTEGER,
    decline_reason VARCHAR(50),
    region VARCHAR(100),
    dynasty VARCHAR(100),
    description TEXT,
    population_peak INTEGER,
    trade_volume INTEGER,
    importance_rank INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 空间索引：地理位置GIST索引
CREATE INDEX idx_cities_geom ON cities USING GIST(geom);

-- B树索引：时间维度查询
CREATE INDEX idx_cities_founded ON cities(founded_year);
CREATE INDEX idx_cities_decline ON cities(decline_year);
CREATE INDEX idx_cities_prosperity ON cities(prosperity_start, prosperity_end);

-- 复合索引：区域 + 衰落状态
CREATE INDEX idx_cities_region_decline ON cities(region, decline_year)
    WHERE decline_year IS NOT NULL;

-- 部分索引：重要城市
CREATE INDEX idx_cities_important ON cities(importance_rank)
    WHERE importance_rank <= 20;

-- =========================================
-- 气候数据表
-- =========================================
CREATE TABLE climate_data (
    id SERIAL PRIMARY KEY,
    period_start INTEGER NOT NULL,
    period_end INTEGER NOT NULL,
    region VARCHAR(100) NOT NULL,
    temperature_anomaly DOUBLE PRECISION,
    precipitation_index DOUBLE PRECISION,
    glacier_advance BOOLEAN,
    glacier_note TEXT,
    climate_phase VARCHAR(50),
    notes TEXT
);

-- 复合索引：区域 + 时期（时空查询模式）
CREATE INDEX idx_climate_region_period ON climate_data(region, period_start, period_end);

-- 单独的时期索引
CREATE INDEX idx_climate_period ON climate_data(period_start, period_end);

-- 部分索引：有冰川标记的记录
CREATE INDEX idx_climate_glacier ON climate_data(period_start, region)
    WHERE glacier_advance = true;

-- =========================================
-- 贸易连接表
-- =========================================
CREATE TABLE trade_connections (
    id SERIAL PRIMARY KEY,
    city_from INTEGER REFERENCES cities(id) ON DELETE CASCADE,
    city_to INTEGER REFERENCES cities(id) ON DELETE CASCADE,
    period_start INTEGER NOT NULL,
    period_end INTEGER NOT NULL,
    trade_volume INTEGER,
    route_type VARCHAR(50),
    commodity VARCHAR(100),
    dynasty VARCHAR(100),
    reliability_score DOUBLE PRECISION DEFAULT 1.0,
    geom GEOMETRY(LineString, 4326)
);

-- 复合索引：城市对 + 时期
CREATE INDEX idx_trade_cities_period ON trade_connections(city_from, city_to, period_start);

-- 反向索引：按目的城市查询
CREATE INDEX idx_trade_to_city ON trade_connections(city_to, period_start);

-- 时期索引
CREATE INDEX idx_trade_period ON trade_connections(period_start, period_end);

-- 路线类型索引
CREATE INDEX idx_trade_route_type ON trade_connections(route_type, period_start)
    WHERE route_type IS NOT NULL;

-- 空间索引：贸易路线GIST索引
CREATE INDEX idx_trade_geom ON trade_connections USING GIST(geom);

-- 朝代索引
CREATE INDEX idx_trade_dynasty ON trade_connections(dynasty, period_start)
    WHERE dynasty IS NOT NULL;

-- =========================================
-- 触发器：自动更新 updated_at
-- =========================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_cities_updated_at BEFORE UPDATE ON cities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =========================================
-- 物化视图：城市贸易汇总（可选优化）
-- =========================================
-- CREATE MATERIALIZED VIEW city_trade_summary AS
-- SELECT
--     c.id as city_id,
--     c.name,
--     tc.period_start,
--     tc.period_end,
--     COUNT(DISTINCT CASE WHEN tc.city_from = c.id THEN tc.city_to ELSE tc.city_from END) as connection_count,
--     SUM(tc.trade_volume) as total_volume
-- FROM cities c
-- JOIN trade_connections tc ON c.id = tc.city_from OR c.id = tc.city_to
-- GROUP BY c.id, c.name, tc.period_start, tc.period_end;

-- CREATE INDEX idx_city_trade_summary ON city_trade_summary(city_id, period_start);

-- =========================================
-- 注释与元数据
-- =========================================
COMMENT ON TABLE cities IS '丝绸之路沿线古代城市基本信息与兴衰记录';
COMMENT ON COLUMN cities.geom IS '城市地理位置（WGS84坐标系）';
COMMENT ON COLUMN cities.decline_reason IS '衰落原因：war/climate/trade_route/other';

COMMENT ON TABLE climate_data IS '古气候重建数据（多区域、多时期）';
COMMENT ON COLUMN climate_data.temperature_anomaly IS '温度距平（相对于1961-1990基准期，单位：°C）';
COMMENT ON COLUMN climate_data.precipitation_index IS '降水指数（0-2，1为正常）';
COMMENT ON COLUMN climate_data.glacier_advance IS '冰川前进标记（寒冷期）';

COMMENT ON TABLE trade_connections IS '城市间贸易连接网络（分时期）';
COMMENT ON COLUMN trade_connections.trade_volume IS '贸易量指数（相对值，非绝对值）';
COMMENT ON COLUMN trade_connections.route_type IS '路线类型：land/sea/river';
COMMENT ON COLUMN trade_connections.reliability_score IS '数据可靠性评分（0-1）';
