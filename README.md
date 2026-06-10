# 丝绸之路沿线古代城市兴衰与气候变迁关联分析系统

## 项目概述

本系统是一个用于考古研究的全栈应用，整合了丝绸之路沿线从公元前200年到公元1500年共120个古代城市的兴衰记录与古气候重建数据，通过Cox比例风险模型和网络分析方法，量化气候因子与贸易路线变迁对城市衰落风险的贡献度。

## 技术栈

### 后端
- **Rust (Axum 0.7)**: 高性能Web服务框架
- **SQLx 0.7**: 异步数据库ORM，支持PostgreSQL
- **Tokio 1.0**: 异步运行时
- **PostgreSQL + PostGIS**: 空间数据存储与查询

### 前端
- **Leaflet 1.9.4**: 交互式地图库
- **HTML5 Canvas**: 动态路线箭头绘制
- **原生JavaScript**: 无需额外框架

### 数据分析
- **Cox比例风险模型**: 生存分析，量化各风险因子
- **网络分析**: 度中心性、介数中心性、特征向量中心性
- **贸易路线重心转移检测**: 基于加权地理重心计算

## 项目结构

```
AI_solo_coder_task_A_079/
├── backend/                    # Rust后端
│   ├── src/
│   │   ├── main.rs            # 服务入口
│   │   ├── handlers.rs        # API路由处理
│   │   ├── models.rs          # 数据模型定义
│   │   ├── cox_model.rs       # Cox比例风险模型实现
│   │   └── network_analysis.rs # 网络分析与路线变迁检测
│   ├── Cargo.toml             # Rust依赖配置
│   └── .env.example           # 环境变量示例
├── frontend/                   # 前端应用
│   ├── index.html             # 主页面
│   ├── css/
│   │   └── style.css          # 样式文件
│   └── js/
│       └── app.js             # 前端逻辑
├── db/                         # 数据库
│   └── init.sql               # 数据库初始化脚本
└── scripts/                    # 数据模拟脚本
    └── simulate_data.py       # Python数据生成脚本
```

## 数据库设计

### 核心表结构

1. **cities** - 城市基本信息
   - 地理位置（经纬度 + PostGIS几何类型）
   - 建立年代、繁荣期、衰落年代
   - 衰落原因（战争/气候/贸易路线改变）
   - 人口峰值、贸易量

2. **climate_data** - 古气候数据（50年分辨率）
   - 温度距平（相对于基准期）
   - 降水指数
   - 冰川进退标记

3. **trade_connections** - 贸易连接网络
   - 城市间连接关系
   - 时期、贸易量、路线类型（陆路/海路）

## API接口文档

### 城市相关
- `GET /api/cities` - 获取所有城市列表
- `GET /api/cities/{id}` - 获取单个城市详情
- `GET /api/cities/{id}/timeline` - 获取城市兴衰时间线及同期气候数据
- `GET /api/cities/status?year={year}` - 获取指定年份各城市的兴衰状态

### 气候相关
- `GET /api/climate` - 获取所有气候数据
- `GET /api/climate/summary` - 获取各区域气候汇总统计

### 贸易路线相关
- `GET /api/trade` - 获取所有贸易连接
- `GET /api/trade/arrows` - 获取用于地图绘制的路线箭头数据

### 分析模型
- `GET /api/analysis/cox` - 运行Cox比例风险模型，输出各因子风险比
- `GET /api/analysis/network?year={year}` - 计算指定年份的网络中心性指标
- `GET /api/analysis/route-shift` - 计算各时期贸易路线重心转移

## 快速开始

### 1. 环境准备

**数据库要求：**
- PostgreSQL 14+
- PostGIS 3.0+

**创建数据库：**
```sql
CREATE DATABASE silkroad;
\c silkroad
CREATE EXTENSION postgis;
```

### 2. 数据库初始化

```bash
cd db
psql -U postgres -d silkroad -f init.sql
```

### 3. 数据模拟

```bash
cd scripts
pip install psycopg2-binary
python simulate_data.py
```

### 4. 后端编译运行

**使用GNU工具链（Windows）：**
```bash
cd backend
cargo build --target x86_64-pc-windows-gnu --release
```

**配置环境变量（backend/.env）：**
```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/silkroad
FRONTEND_DIR=../frontend
LISTEN_ADDR=0.0.0.0:3000
RUST_LOG=silkroad_analysis=info,tower_http=debug
```

**启动服务：**
```bash
cargo run --target x86_64-pc-windows-gnu
```

### 5. 访问系统

打开浏览器访问：`http://localhost:3000`

## 核心功能说明

### 1. 交互式历史地图
- 丝绸之路沿线120个城市的空间分布
- 城市圆点颜色根据兴衰状态动态变化：
  - 🟢 绿色：繁荣期
  - 🟡 黄色：过渡期
  - 🔴 红色：衰落期
- 可拖动时间轴（公元前200年 - 公元1500年）
- 播放/暂停/重置动画控制

### 2. 城市详情面板
点击城市圆点弹出详情面板，包含：
- 城市基本信息（名称、建立年代、区域等）
- 兴衰时间线可视化（Canvas绘制）
- 同期温度距平曲线
- 同期降水指数曲线

### 3. 贸易路线动态展示
- Canvas绘制动态流动箭头
- 箭头粗细表示贸易量大小
- 蓝色：陆路贸易
- 紫色：海路贸易
- 随时间轴变化显示不同时期的贸易网络

### 4. Cox比例风险模型分析
量化以下因子对城市衰落风险的贡献：
- **温度变化**：繁荣期与前期的温度距平差异
- **降水变化**：繁荣期与前期的降水指数差异
- **路线变迁**：贸易路线改变的影响
- **冰川前进**：寒冷期冰川活动的影响

输出指标：
- 回归系数(β)
- 风险比(HR)：HR>1表示增加风险，HR<1表示降低风险
- 95%置信区间
- p值（统计显著性）
- Concordance指数（模型拟合优度）

### 5. 贸易路线变迁检测
- 计算各时期贸易网络的地理重心
- 追踪经度和纬度重心的迁移轨迹
- 量化相邻时期的重心偏移距离（公里）
- 网络中心性分析：
  - 度中心性：城市连接数
  - 介数中心性：城市在最短路径中的出现频率
  - 特征向量中心性：连接到重要城市的程度

## 数据说明

### 城市数据
包含120个丝绸之路沿线城市，涵盖以下区域：
- 关中地区（长安、咸阳等）
- 河西走廊（兰州、敦煌、武威等）
- 塔里木盆地（楼兰、于阗、龟兹、疏勒等）
- 河中地区（撒马尔罕、布哈拉、塔什干等）
- 波斯（木鹿、尼沙普尔、赫拉特等）
- 美索不达米亚（巴格达、泰西封、巴比伦等）
- 地中海东岸（安条克、大马士革、巴尔米拉等）
- 安纳托利亚（君士坦丁堡、以弗所、特拉布宗等）
- 地中海西端（罗马、亚历山大、威尼斯等）

### 古气候数据
每50年分辨率，包含：
- 罗马暖期（公元前250年 - 公元400年）
- 晚古典小冰期（公元530年 - 660年）
- 中世纪暖期（公元900年 - 1300年）
- 小冰期前期（公元1300年 - 1500年）

## 算法原理

### Cox比例风险模型

Cox模型的风险函数形式为：
```
h(t|X) = h₀(t) * exp(β₁X₁ + β₂X₂ + ... + βₚXₚ)
```

其中：
- h₀(t) 是基准风险函数
- βᵢ 是协变量Xᵢ的回归系数
- 风险比HRᵢ = exp(βᵢ) 表示协变量每增加一个单位时，风险的倍数变化

本系统使用Newton-Raphson算法最大化偏似然函数估计参数。

### 网络中心性计算

1. **度中心性**：节点的直接连接数占最大可能连接数的比例
2. **介数中心性**：使用Brandes算法计算所有节点对之间最短路径经过该节点的比例
3. **特征向量中心性**：使用幂迭代法计算邻接矩阵的主特征向量

### 贸易路线重心转移

加权地理重心计算公式：
```
Lon_center = Σ(Lonᵢ * Volumeᵢ) / Σ(Volumeᵢ)
Lat_center = Σ(Latᵢ * Volumeᵢ) / Σ(Volumeᵢ)
```

两点间球面距离近似：
```
Distance(km) = 111 * √(ΔLat² + (ΔLon * cos(Lat_center))²)
```

## 开发说明

### 后端编译注意事项

Windows环境下如果缺少MSVC链接器，可使用GNU工具链：
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu
```

### 前端开发

前端使用原生JavaScript，无需构建工具。直接在浏览器中打开`index.html`即可（需配合后端API服务）。

## 参考文献

1. 丝绸之路沿线环境变迁与文明兴衰研究
2. 过去2000年中国气候变化的重建与机制研究
3. Cox D.R. (1972) Regression Models and Life-Tables
4. Brandes U. (2001) A Faster Algorithm for Betweenness Centrality

## 许可证

本项目仅供学术研究使用。
