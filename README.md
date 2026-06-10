# 丝绸之路古代城市兴衰与气候变迁关联分析系统

> 考古研究全栈应用：整合丝绸之路沿线120个古代城市兴衰记录与古气候重建数据，通过Cox比例风险模型和复杂网络分析，量化气候因子与贸易路线变迁对城市衰落风险的贡献。

---

## 目录

- [系统架构](#系统架构)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [快速开始](#快速开始)
  - [Docker 一键部署（推荐）](#docker-一键部署推荐)
  - [本地开发环境](#本地开发环境)
- [数据模拟器](#数据模拟器)
- [API 接口](#api-接口)
- [监控与指标](#监控与指标)
- [配置说明](#配置说明)
- [核心算法](#核心算法)
- [开发说明](#开发说明)

---

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                        前端 (浏览器)                         │
│  ┌──────────────┐  ┌───────────────┐  ┌────────────────┐  │
│  │ silkroad_map │  │ city_detail   │  │ 时间轴/控制面板│  │
│  │ 地图 + WebGL │  │ 详情 + 图表   │  │                │  │
│  └──────┬───────┘  └───────┬───────┘  └────────────────┘  │
│         │  HTTP/Gzip        │                                │
└─────────┼───────────────────┼────────────────────────────────┘
          │                   │
          ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                 Rust API 服务 (Axum)                         │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                        handlers                      │  │
│  │  12个API端点 · CORS · Gzip · Tracing · Metrics     │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                         │                                  │
│  ┌─────────┐  ┌───────────────┐  ┌──────────────────┐    │
│  │  data_  │  │ survival_     │  │  route_          │    │
│  │  loader │  │  model        │  │  analyzer        │    │
│  │ 数据预  │  │ Cox回归       │  │ 网络分析         │    │
│  │ 处理    │  │ 风险比计算    │  │ 路线变迁检测     │    │
│  └────┬────┘  └───────┬───────┘  └────────┬─────────┘    │
│       │                │                    │              │
│  ┌────▼────────────────▼────────────────────▼──────────┐  │
│  │                    config (参数外置)                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                         │                                  │
└─────────────────────────┼──────────────────────────────────┘
                          │ SQLx
                          ▼
┌─────────────────────────────────────────────────────────────┐
│            PostgreSQL 15 + PostGIS 3.4                      │
│  ┌──────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │  cities  │  │ climate_data │  │ trade_connections  │  │
│  │ 空间索引 │  │  区域+时期   │  │  空间索引+朝代    │  │
│  └──────────┘  └──────────────┘  └────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 技术栈

### 后端
- **Rust 1.75+**：高性能安全系统编程语言
- **Axum 0.7**：异步Web框架，基于 tokio + tower
- **SQLx 0.7**：编译时验证的异步数据库ORM
- **Tokio 1.0**：异步运行时
- **tracing**：结构化日志 + 分布式追踪
- **metrics + metrics-exporter-prometheus**：Prometheus 指标暴露
- **tower-http**：CORS、Gzip压缩、Trace中间件
- **nalgebra**：线性代数库（矩阵运算）

### 数据库
- **PostgreSQL 15**：关系型数据库
- **PostGIS 3.4**：空间数据扩展
- **GIST 空间索引**：地理位置快速查询

### 前端
- **Leaflet 1.9.4**：交互式地图库
- **WebGL**：高性能路线箭头渲染
- **Canvas API**：图表绘制
- **原生 JavaScript**：IIFE 模块化，无构建依赖

### 工程化
- **Docker + docker-compose**：容器化部署
- **多阶段构建**：最小化Rust镜像体积
- **健康检查**：服务依赖自动等待
- **幂等数据初始化**：已有数据自动跳过

---

## 项目结构

```
AI_solo_coder_task_A_079/
├── backend/                    # Rust 后端服务
│   ├── src/
│   │   ├── main.rs            # 服务入口 · tracing · metrics · gzip
│   │   ├── config.rs          # 配置参数（外置，支持环境变量）
│   │   ├── data_loader.rs     # 数据加载与预处理模块
│   │   ├── survival_model.rs  # Cox比例风险模型模块
│   │   ├── route_analyzer.rs  # 贸易网络分析模块
│   │   ├── handlers.rs        # API 路由处理
│   │   └── models.rs          # 数据模型定义
│   ├── Cargo.toml             # Rust 依赖配置
│   ├── Dockerfile             # Rust 镜像构建（多阶段）
│   ├── .dockerignore
│   └── .env.example           # 环境变量示例
├── frontend/                   # 前端应用
│   ├── index.html             # 主页面
│   ├── css/style.css          # 样式文件
│   └── js/
│       ├── silkroad_map.js    # 地图模块（Leaflet + WebGL）
│       ├── city_detail.js     # 详情面板 + 分析图表模块
│       └── app.js             # 入口文件，模块桥接
├── db/                         # 数据库
│   └── init.sql               # 初始化脚本（表 + 索引 + 触发器）
├── scripts/                    # 数据模拟器
│   ├── simulate_data.py       # 模拟数据生成脚本
│   └── Dockerfile             # 模拟器镜像
├── docker-compose.yml         # 三服务编排
├── .dockerignore              # 根目录构建忽略
└── README.md                  # 本文档
```

---

## 快速开始

### Docker 一键部署（推荐）

#### 前置要求
- Docker 20.10+
- docker-compose v2+

#### 启动服务

```bash
# 克隆或进入项目目录
cd AI_solo_coder_task_A_079

# 一键启动所有服务
docker-compose up -d

# 查看启动日志
docker-compose logs -f
```

#### 服务说明

| 服务 | 端口 | 说明 |
|------|------|------|
| **silkroad-api** | 3000 | Rust API服务 + 前端静态文件 |
| **silkroad-postgis** | 5432 | PostgreSQL + PostGIS 数据库 |
| **silkroad-simulator** | — | 数据模拟器（一次性任务） |

#### 访问

- 系统首页：http://localhost:3000
- API 文档（接口列表）：http://localhost:3000/api
- Prometheus 指标：http://localhost:3000/metrics

#### 常用命令

```bash
# 停止服务
docker-compose down

# 停止并清除数据（重新生成数据）
docker-compose down -v

# 重新构建镜像
docker-compose build

# 只重启API服务
docker-compose restart api

# 查看模拟器日志
docker-compose logs simulator
```

---

### 本地开发环境

#### 1. 数据库准备

```bash
# 使用Docker启动PostGIS
docker run -d \
  --name silkroad-postgis \
  -e POSTGRES_DB=silkroad \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgis/postgis:15-3.4

# 等待数据库就绪，初始化表结构
psql -U postgres -d silkroad -h localhost -f db/init.sql
```

#### 2. 生成模拟数据

```bash
cd scripts
pip install psycopg2-binary
python simulate_data.py
```

#### 3. 启动后端服务

```bash
cd backend

# 配置环境变量
cp .env.example .env

# 编译运行
cargo run
```

#### 4. 访问前端

直接打开浏览器访问 http://localhost:3000

---

## 数据模拟器

数据模拟器生成考古研究所需的完整数据集，可复现、可配置。

### 生成的数据

| 数据集 | 数量 | 说明 |
|--------|------|------|
| 古代城市 | 120 个 | 9大区域，含兴衰时间、人口、贸易量 |
| 气候记录 | 272 条 | 8个区域 × 34个时期（50年分辨率） |
| 贸易连接 | ~2000 条 | 10个朝代 × 区域间+区域内贸易网 |

### 朝代贸易模型

模拟器按真实历史朝代划分贸易强度：

| 朝代 | 时间跨度 | 贸易系数 | 海路系数 | 说明 |
|------|---------|---------|---------|------|
| 西汉 | -202 ~ 8 | 1.2 | 0.3 | 张骞通西域，丝路初通 |
| 东汉 | 25 ~ 220 | 1.3 | 0.4 | 班超经营西域 |
| 魏晋南北朝 | 220 ~ 589 | 0.7 | 0.5 | 战乱频繁，陆上丝路受阻 |
| 隋 | 581 ~ 618 | 1.0 | 0.5 | 统一王朝，贸易恢复 |
| 唐 | 618 ~ 907 | 1.8 | 0.8 | 盛世，丝路全盛期 |
| 五代十国 | 907 ~ 960 | 0.6 | 0.6 | 分裂割据 |
| 北宋 | 960 ~ 1127 | 1.5 | 1.2 | 经济繁荣，海上丝路兴起 |
| 南宋 | 1127 ~ 1279 | 1.3 | 1.5 | 偏安江南，海上贸易极盛 |
| 元 | 1271 ~ 1368 | 1.6 | 1.0 | 蒙古帝国，陆上丝路再通 |
| 明 | 1368 ~ 1644 | 1.1 | 0.9 | 海禁政策，贸易波动 |

### 古气候模型

基于历史气候重建研究，模拟5个主要气候阶段：

| 气候阶段 | 时间 | 温度距平 | 降水偏移 |
|---------|------|---------|---------|
| Roman Warm Period | -250 ~ 400 | +0.8°C | +0.2 |
| Late Antique Little Ice Age | 400 ~ 660 | -1.2°C | -0.3 |
| Early Medieval Cool Period | 660 ~ 900 | 0.0°C | +0.1 |
| Medieval Warm Period | 900 ~ 1300 | +0.7°C | +0.3 |
| Little Ice Age Onset | 1300 ~ 1500 | -0.6°C | -0.2 |

每个区域有独立的基准调整参数，体现地理差异。

### 模拟器环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `DB_HOST` | localhost | 数据库主机 |
| `DB_PORT` | 5432 | 数据库端口 |
| `DB_NAME` | silkroad | 数据库名 |
| `DB_USER` | postgres | 用户名 |
| `DB_PASSWORD` | postgres | 密码 |
| `SIM_RANDOM_SEED` | 42 | 随机种子（保证可复现） |

### 可复现性

设置相同的 `SIM_RANDOM_SEED` 可获得完全相同的数据集，便于研究对比。

```bash
# 生成种子42的数据集（默认）
SIM_RANDOM_SEED=42 python simulate_data.py

# 生成种子123的对比数据集
SIM_RANDOM_SEED=123 python simulate_data.py
```

---

## API 接口

### 城市相关

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/cities` | GET | 获取所有城市列表 |
| `/api/cities/{id}` | GET | 获取单个城市详情 |
| `/api/cities/{id}/timeline` | GET | 城市兴衰时间线 + 同期气候 |
| `/api/cities/status?year={year}` | GET | 指定年份城市状态 |

### 气候相关

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/climate` | GET | 所有气候数据 |
| `/api/climate/summary` | GET | 各区域气候汇总统计 |

### 贸易路线相关

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/trade` | GET | 所有贸易连接 |
| `/api/trade/arrows` | GET | 地图用箭头数据 |

### 分析模型

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/analysis/cox` | GET | Cox比例风险模型分析结果 |
| `/api/analysis/network?year={year}` | GET | 指定年份网络中心性 |
| `/api/analysis/route-shift` | GET | 各时期贸易路线重心转移 |

### 监控

| 接口 | 方法 | 说明 |
|------|------|------|
| `/metrics` | GET | Prometheus 格式指标 |

---

## 监控与指标

### Prometheus 指标

访问 `/metrics` 端点获取Prometheus格式指标。

**内置指标（tower-http + metrics）：**

| 指标名 | 类型 | 说明 |
|--------|------|------|
| `http_requests_total` | Counter | HTTP请求总数 |
| `http_requests_duration_seconds` | Histogram | 请求耗时分布 |
| `http_requests_active` | Gauge | 当前活跃请求数 |

**业务指标（待扩展）：**
- Cox模型计算次数
- 数据查询缓存命中率

### Tracing 日志

结构化日志输出，包含：
- 请求方法、路径、状态码
- 请求耗时（毫秒）
- 模块、级别

日志级别通过 `RUST_LOG` 环境变量控制：

```bash
# 开发调试
RUST_LOG=silkroad_analysis=debug,tower_http=debug

# 生产环境
RUST_LOG=silkroad_analysis=info,tower_http=warn,sqlx=warn
```

---

## 配置说明

### Cox 模型参数

| 参数 | 默认值 | 环境变量 | 说明 |
|------|--------|---------|------|
| interval_width | 200 | `COX_INTERVAL_WIDTH` | 时变区间宽度（年） |
| max_iterations | 300 | `COX_MAX_ITER` | Newton-Raphson最大迭代次数 |
| convergence_tolerance | 1e-8 | `COX_TOLERANCE` | 收敛容忍度 |
| lag_years | 200 | `COX_LAG_YEARS` | 气候滞后效应年数 |
| confidence_level | 0.95 | — | 置信区间置信水平 |
| min_sample_size | 10 | — | 最小样本量 |

### 网络分析参数

| 参数 | 默认值 | 环境变量 | 说明 |
|------|--------|---------|------|
| max_land_route_km | 800 | `NET_MAX_LAND_KM` | 陆路最大连接距离(km) |
| max_sea_route_km | 3000 | `NET_MAX_SEA_KM` | 海路最大连接距离(km) |
| min_neighbors | 5 | `NET_MIN_NEIGHBORS` | 每个城市最少邻居数 |

### 服务配置

| 参数 | 默认值 | 环境变量 | 说明 |
|------|--------|---------|------|
| database_url | — | `DATABASE_URL` | 数据库连接串 |
| listen_addr | 0.0.0.0:3000 | `LISTEN_ADDR` | 监听地址 |
| frontend_dir | ../frontend | `FRONTEND_DIR` | 前端静态文件目录 |

---

## 核心算法

### Cox 比例风险模型

时变协变量 Cox 模型（Counting Process 格式）：

```
h(t|X(t)) = h₀(t) × exp(β·X(t))
```

- 6个协变量：温度距平、降水指数、温度变化、降水变化、路线变迁、冰川前进
- Newton-Raphson 算法最大化偏似然
- 输出：回归系数β、风险比HR、95%置信区间、p值、Concordance指数

详见：[survival_model.rs](backend/src/survival_model.rs)

### 网络中心性分析

1. **度中心性**：直接连接数
2. **介数中心性**：Brandes算法，O(nm) 时间复杂度
3. **特征向量中心性**：幂迭代法

### 贸易路线变迁检测

- 加权地理重心计算
- 9个历史时期轨迹
- Savitzky-Golay 平滑
- 地理约束：山脉屏障检测

---

## 开发说明

### 代码模块化

后端采用单一职责原则拆分：
- **config.rs**：所有可配置参数
- **data_loader.rs**：数据加载 + 时变区间构建
- **survival_model.rs**：纯算法层，可独立测试
- **route_analyzer.rs**：网络分析纯算法
- **handlers.rs**：HTTP 层，编排业务逻辑

前端采用 IIFE 模块化：
- **silkroad_map.js**：地图渲染、时间轴、WebGL
- **city_detail.js**：详情面板、图表、分析模态框
- **app.js**：入口，事件桥接

### 后端编译优化

Release 配置启用 LTO 和 strip，最小化体积：

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### 前端 Gzip 压缩

后端通过 tower-http 的 CompressionLayer 对所有响应启用 Gzip 压缩，静态文件和 API 响应均受益。

---

## 数据说明

### 数据来源

本系统数据为**模拟生成**，基于以下研究结论的定性趋势：
- 丝绸之路沿线城市兴衰历史时间线
- 过去2000年全球温度重建（PAGES 2k Consortium）
- 历史贸易路线变迁研究

**注意**：模拟数据仅用于系统演示和算法验证，实际考古研究需使用经考证的真实数据。

### 字段说明

**城市衰落原因**：
- `war`：战争破坏
- `climate`：气候恶化
- `trade_route`：贸易路线变迁

**路线类型**：
- `land`：陆路贸易
- `sea`：海路贸易

**可靠性评分**：
- 贸易连接的数据可靠性，0.6-1.0 之间

---

## 参考文献

1. PAGES 2k Consortium (2019) - 过去2000年全球温度重建
2. Cox D.R. (1972) - Regression Models and Life-Tables
3. Brandes U. (2001) - A Faster Algorithm for Betweenness Centrality
4. 丝绸之路沿线环境变迁与文明兴衰研究
5. 中国历史气候变化研究

---

## 许可证

本项目仅供学术研究使用。
