CREATE EXTENSION IF NOT EXISTS postgis;

DROP TABLE IF EXISTS trade_connections;
DROP TABLE IF EXISTS climate_data;
DROP TABLE IF EXISTS cities;

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
    description TEXT,
    population_peak INTEGER,
    trade_volume INTEGER
);

CREATE TABLE climate_data (
    id SERIAL PRIMARY KEY,
    period_start INTEGER NOT NULL,
    period_end INTEGER NOT NULL,
    region VARCHAR(100) NOT NULL,
    temperature_anomaly DOUBLE PRECISION,
    precipitation_index DOUBLE PRECISION,
    glacier_advance BOOLEAN,
    glacier_note TEXT,
    notes TEXT
);

CREATE TABLE trade_connections (
    id SERIAL PRIMARY KEY,
    city_from INTEGER REFERENCES cities(id),
    city_to INTEGER REFERENCES cities(id),
    period_start INTEGER NOT NULL,
    period_end INTEGER NOT NULL,
    trade_volume INTEGER,
    route_type VARCHAR(50)
);

CREATE INDEX idx_cities_geom ON cities USING GIST(geom);
CREATE INDEX idx_cities_founded ON cities(founded_year);
CREATE INDEX idx_cities_decline ON cities(decline_year);
CREATE INDEX idx_climate_period ON climate_data(period_start, period_end);
CREATE INDEX idx_climate_region ON climate_data(region);
CREATE INDEX idx_trade_cities ON trade_connections(city_from, city_to);
CREATE INDEX idx_trade_period ON trade_connections(period_start, period_end);
