# -*- coding: utf-8 -*-
"""
丝绸之路古代城市数据模拟器
====================

生成120个古代城市、按朝代划分的贸易网络、多区域古气候重建数据。

环境变量配置：
    DB_HOST       数据库主机（默认 localhost）
    DB_PORT       数据库端口（默认 5432）
    DB_NAME       数据库名（默认 silkroad）
    DB_USER       用户名（默认 postgres）
    DB_PASSWORD   密码（默认 postgres）
    SIM_NUM_CITIES 城市数量（默认 120）
    SIM_RANDOM_SEED 随机种子（默认 42）
    SIM_CLI_PERIODS 气候时期数量（默认 34，50年分辨率）
"""

import psycopg2
import random
import math
import os
import sys

# =========================================
# 配置
# =========================================
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_PORT = os.environ.get("DB_PORT", "5432")
DB_NAME = os.environ.get("DB_NAME", "silkroad")
DB_USER = os.environ.get("DB_USER", "postgres")
DB_PASSWORD = os.environ.get("DB_PASSWORD", "postgres")
RANDOM_SEED = int(os.environ.get("SIM_RANDOM_SEED", "42"))

DATABASE_URL = f"postgresql://{DB_USER}:{DB_PASSWORD}@{DB_HOST}:{DB_PORT}/{DB_NAME}"

# =========================================
# 朝代定义
# =========================================
DYNASTIES = [
    {"name": "西汉", "en_name": "Western Han", "start": -202, "end": 8, "trade_factor": 1.2, "sea_factor": 0.3},
    {"name": "东汉", "en_name": "Eastern Han", "start": 25, "end": 220, "trade_factor": 1.3, "sea_factor": 0.4},
    {"name": "魏晋南北朝", "en_name": "Wei-Jin-North-South", "start": 220, "end": 589, "trade_factor": 0.7, "sea_factor": 0.5},
    {"name": "隋", "en_name": "Sui", "start": 581, "end": 618, "trade_factor": 1.0, "sea_factor": 0.5},
    {"name": "唐", "en_name": "Tang", "start": 618, "end": 907, "trade_factor": 1.8, "sea_factor": 0.8},
    {"name": "五代十国", "en_name": "Five Dynasties", "start": 907, "end": 960, "trade_factor": 0.6, "sea_factor": 0.6},
    {"name": "北宋", "en_name": "Northern Song", "start": 960, "end": 1127, "trade_factor": 1.5, "sea_factor": 1.2},
    {"name": "南宋", "en_name": "Southern Song", "start": 1127, "end": 1279, "trade_factor": 1.3, "sea_factor": 1.5},
    {"name": "元", "en_name": "Yuan", "start": 1271, "end": 1368, "trade_factor": 1.6, "sea_factor": 1.0},
    {"name": "明", "en_name": "Ming", "start": 1368, "end": 1644, "trade_factor": 1.1, "sea_factor": 0.9},
]

# 主要商品类型
COMMODITIES = [
    "silk", "porcelain", "tea", "spices", "jade",
    "gold", "silver", "glass", "paper", "wool",
    "horses", "grain", "incense", "gems", "textiles"
]

# =========================================
# 城市数据
# =========================================
CITIES_RAW = [
    ("Chang'an", "长安", 108.94, 34.26, -1046, -202, 907, "关中",
     "Capital of Han and Tang dynasties, eastern terminus of the Silk Road", 1000000, 9500, 1),
    ("Xianyang", "咸阳", 108.71, 34.33, -350, -221, -206, "关中",
     "Capital of Qin dynasty, predecessor to Chang'an", 500000, 6800, 3),
    ("Yongcheng", "雍城", 107.39, 34.52, -771, -677, -383, "关中",
     "Capital of Qin state during Spring and Autumn period", 120000, 3200, 8),
    ("Huayin", "华阴", 110.09, 34.57, -200, -100, 600, "关中",
     "Gateway city at the foot of Mount Hua", 80000, 2800, 12),
    ("Baoji", "宝鸡", 107.14, 34.37, -600, -400, 400, "关中",
     "Strategic western gateway of Guanzhong", 150000, 4100, 6),
    ("Fengxiang", "凤翔", 107.40, 34.52, -677, -600, 200, "关中",
     "Ancient Qin capital, later important market town", 90000, 2500, 10),
    ("Tongchuan", "铜川", 108.95, 34.90, -150, 50, 700, "关中",
     "Ceramics production center on Silk Road corridor", 60000, 1800, 15),
    ("Lintong", "临潼", 109.21, 34.37, -300, -200, 800, "关中",
     "Site of Terracotta Army, important garrison town", 110000, 3000, 9),
    ("Pucheng", "蒲城", 109.59, 34.95, -100, 100, 600, "关中",
     "Agricultural hub and military depot", 70000, 2000, 14),
    ("Weinan", "渭南", 109.50, 34.50, -200, -50, 900, "关中",
     "Major trading post on Wei River valley route", 130000, 3500, 7),

    ("Lanzhou", "兰州", 103.83, 36.06, -86, 0, 1500, "河西走廊",
     "Major crossing point of the Yellow River on the Silk Road", 400000, 7200, 2),
    ("Tianshui", "天水", 105.72, 34.58, -300, -100, 1200, "河西走廊",
     "Ancient Qin state origin, key Silk Road station", 200000, 4500, 5),
    ("Wuwei", "武威", 102.64, 37.93, -121, 100, 1100, "河西走廊",
     "Liangzhou, major military and trade center", 250000, 5800, 4),
    ("Zhangye", "张掖", 100.45, 38.93, -111, 50, 1200, "河西走廊",
     "Ganzhou, vital oasis city on Hexi Corridor", 180000, 4200, 8),
    ("Jiuquan", "酒泉", 98.49, 39.74, -111, 50, 1300, "河西走廊",
     "Suzhou, western gateway of Hexi Corridor", 120000, 3500, 10),
    ("Dunhuang", "敦煌", 94.66, 40.14, -111, -50, 1400, "河西走廊",
     "Critical junction where Silk Road splits into north and south routes", 150000, 8500, 2),
    ("Jiayuguan", "嘉峪关", 98.29, 39.77, 1372, 1400, 1500, "河西走廊",
     "Western end of Great Wall, Ming dynasty fortress", 80000, 2200, 15),
    ("Baiyin", "白银", 104.18, 36.55, -50, 200, 900, "河西走廊",
     "Mining town and caravan stop", 60000, 1500, 18),
    ("Jinchang", "金昌", 102.19, 38.52, -80, 50, 800, "河西走廊",
     "Frontier garrison and trade outpost", 50000, 1200, 20),
    ("Yongchang", "永昌", 101.97, 38.33, -80, 100, 700, "河西走廊",
     "Important military colony on the corridor", 45000, 1100, 22),
    ("Minqin", "民勤", 103.09, 38.62, -120, -50, 800, "河西走廊",
     "Oasis town between deserts", 40000, 900, 25),
    ("Shandan", "山丹", 101.09, 38.78, -100, 50, 900, "河西走廊",
     "Pastoral and military town along the corridor", 35000, 800, 28),
    ("Guazhou", "瓜州", 95.78, 40.52, -100, -20, 1200, "河西走廊",
     "Oasis town known for melons, waypoint to Dunhuang", 55000, 2800, 12),
    ("Yumen", "玉门", 97.04, 40.28, -100, -20, 900, "河西走廊",
     "Jade Gate Pass, legendary Silk Road checkpoint", 40000, 2000, 16),
    ("Anxi", "安西", 95.78, 40.52, -80, 100, 1000, "河西走廊",
     "Military outpost guarding route to the west", 30000, 1500, 20),

    ("Loulan", "楼兰", 89.80, 40.51, -200, -176, 400, "塔里木盆地",
     "Mysterious kingdom lost to desert, crucial Silk Road post", 50000, 4500, 4),
    ("Khotan", "于阗", 79.93, 37.11, -250, -100, 1000, "塔里木盆地",
     "Famous for jade and silk production on southern route", 120000, 6200, 3),
    ("Kucha", "龟兹", 82.93, 41.72, -200, -100, 900, "塔里木盆地",
     "Buddhist cultural center and music hub on northern route", 100000, 5500, 5),
    ("Kashgar", "疏勒", 75.99, 39.47, -200, -128, 1500, "塔里木盆地",
     "Key junction where routes converge before crossing Pamirs", 200000, 7800, 1),
    ("Gaochang", "高昌", 89.55, 42.95, -60, 100, 1400, "塔里木盆地",
     "Turpan oasis kingdom, strategic northern route city", 80000, 3800, 7),
    ("Qiemo", "且末", 85.53, 38.15, -200, -150, 700, "塔里木盆地",
     "Small kingdom on southern Silk Road route", 30000, 1200, 18),
    ("Yanqi", "焉耆", 86.54, 42.06, -200, -100, 800, "塔里木盆地",
     "Buddhist kingdom on the northern route", 60000, 2800, 12),
    ("Yarkand", "莎车", 77.28, 38.42, -150, -50, 1500, "塔里木盆地",
     "Major trading center on southern route to Central Asia", 130000, 5200, 6),
    ("Niya", "精绝", 82.70, 37.93, -200, -150, 300, "塔里木盆地",
     "Lost oasis kingdom revealed by archaeology", 20000, 800, 25),
    ("Pishan", "皮山", 78.30, 37.62, -150, -50, 900, "塔里木盆地",
     "Waypoint on the southern route near Khotan", 25000, 1000, 22),
    ("Ruoqiang", "婼羌", 88.50, 39.03, -200, -100, 600, "塔里木盆地",
     "Ancient kingdom near Lop Nur", 15000, 600, 30),
    ("Yuli", "尉犁", 86.26, 41.35, -100, 50, 700, "塔里木盆地",
     "Frontier garrison near Kucha", 20000, 800, 28),
    ("Luntai", "轮台", 84.25, 41.79, -60, 100, 800, "塔里木盆地",
     "Han dynasty military colony, later Tang garrison", 35000, 1500, 16),
    ("Aksu", "姑墨", 80.27, 41.17, -150, -50, 1200, "塔里木盆地",
     "Oasis city between Kashgar and Kucha on northern route", 70000, 3200, 9),
    ("Hami", "伊吾", 93.51, 42.83, -100, 50, 1400, "塔里木盆地",
     "Eastern gateway to Tarim Basin from Hexi Corridor", 60000, 2500, 10),
    ("Jimsar", "庭州", 89.18, 44.00, -60, 200, 800, "塔里木盆地",
     "Beiting, Tang dynasty protectorate headquarters", 50000, 2200, 14),
    ("Wensu", "温宿", 80.24, 41.28, -150, -50, 900, "塔里木盆地",
     "Agricultural oasis near Aksu", 30000, 1100, 20),
    ("Pulei", "蒲类", 93.50, 43.80, -100, 50, 700, "塔里木盆地",
     "Nomadic territory near Hami", 15000, 600, 35),
    ("Yutou", "尉头", 79.00, 40.50, -150, -50, 600, "塔里木盆地",
     "Mountain pass outpost between Kashgar and Pamirs", 10000, 450, 38),
    ("Quli", "渠犁", 84.50, 41.50, -60, 100, 600, "塔里木盆地",
     "Agricultural settlement near Luntai", 14000, 550, 40),

    ("Samarkand", "撒马尔罕", 66.96, 39.65, -700, -300, 1500, "河中地区",
     "Jewel of the Silk Road, center of Sogdian merchant network", 300000, 9000, 1),
    ("Bukhara", "布哈拉", 64.42, 39.77, -500, -200, 1500, "河中地区",
     "Major Islamic intellectual and trade center", 250000, 7500, 2),
    ("Khwarezm", "花剌子模", 59.05, 41.38, -700, -400, 1200, "河中地区",
     "Delta kingdom controlling Amu Darya trade routes", 150000, 4500, 8),
    ("Khujand", "忽毡", 69.60, 40.30, -500, -200, 1400, "河中地区",
     "Key city at Fergana Valley entrance, Alexander's extreme", 100000, 3800, 5),
    ("Fergana", "费尔干纳", 71.85, 40.40, -400, -200, 1500, "河中地区",
     "Valley of horses, source of Ferghana horses for China", 80000, 3200, 6),
    ("Termez", "铁尔梅兹", 67.27, 37.22, -300, -100, 1200, "河中地区",
     "Buddhist center and Amu Darya crossing point", 70000, 2800, 10),
    ("Tashkent", "塔什干", 69.28, 41.31, -300, -100, 1500, "河中地区",
     "Major oasis city at crossroads of Central Asian routes", 180000, 5000, 3),
    ("Penjikent", "片治肯特", 67.73, 39.50, -400, -200, 800, "河中地区",
     "Sogdian city with famous frescoes and temples", 40000, 1800, 15),
    ("Shahrisabz", "沙赫里萨布兹", 66.83, 39.05, -500, -200, 1400, "河中地区",
     "Birthplace of Timur, ancient Nautaca", 60000, 2200, 12),
    ("Karshi", "卡尔希", 65.72, 38.86, -300, -100, 1300, "河中地区",
     "Sogdian trading city known as Nasaf", 50000, 1800, 14),
    ("Hissar", "希萨尔", 68.55, 38.50, -200, 100, 1300, "河中地区",
     "Mountain fortress guarding Samarkand approach", 30000, 1000, 20),
    ("Jizzakh", "吉扎克", 67.84, 40.10, -200, 100, 1200, "河中地区",
     "Caravanserai town between Samarkand and Tashkent", 35000, 1200, 18),
    ("Chaghaniyan", "赤鄂衍那", 67.50, 38.00, -300, -100, 800, "河中地区",
     "Small principality on the Surkhan Darya", 20000, 800, 25),
    ("Shugnan", "识匿", 71.50, 37.60, -100, 200, 900, "河中地区",
     "Mountain kingdom controlling Pamir passes", 15000, 600, 30),
    ("Andijan", "宁远", 72.34, 40.78, -200, 100, 1400, "河中地区",
     "Eastern Fergana city, gateway to Kashgar", 55000, 2000, 11),

    ("Merv", "木鹿", 62.18, 37.66, -600, -300, 1200, "波斯",
     "Great oasis city, one of the largest in the medieval world", 250000, 7000, 2),
    ("Nishapur", "尼沙普尔", 58.80, 36.21, -300, -100, 1200, "波斯",
     "Major trading city in Khorasan, turquoise capital", 150000, 4800, 5),
    ("Herat", "赫拉特", 62.20, 34.34, -500, -200, 1500, "波斯",
     "Key city on route to India, center of arts and learning", 180000, 5200, 3),
    ("Balkh", "巴里黑", 66.90, 36.77, -600, -400, 1200, "波斯",
     "Ancient Bactra, mother of cities, Zoroastrian center", 200000, 6000, 4),
    ("Rayy", "雷伊", 51.42, 35.69, -300, -100, 1200, "波斯",
     "Major city near modern Tehran, important Silk Road node", 150000, 4500, 6),
    ("Isfahan", "伊斯法罕", 51.67, 32.65, -200, 100, 1500, "波斯",
     "Half the world, legendary center of Persian culture", 200000, 5500, 1),
    ("Tabriz", "大不里士", 46.30, 38.08, -100, 200, 1500, "波斯",
     "Major trading city connecting Persia to Anatolia", 180000, 5000, 5),
    ("Yazd", "亚兹德", 54.37, 31.90, -300, 0, 1500, "波斯",
     "Zoroastrian center, famous for silk and textiles", 80000, 2800, 10),
    ("Kerman", "克尔曼", 57.08, 30.28, -200, 100, 1400, "波斯",
     "Gateway to Persian Gulf ports, caravan city", 70000, 2500, 12),
    ("Sistan", "锡斯坦", 61.50, 31.00, -400, -200, 1000, "波斯",
     "Ancient Drangiana, crossroads of Central and South Asian routes", 50000, 1800, 15),
    ("Gurganj", "古尔甘吉", 59.10, 42.30, -400, -200, 1200, "波斯",
     "Capital of Khwarezm, major trading emporium on Amu Darya", 120000, 3800, 9),
    ("Qom", "库姆", 50.88, 34.64, 100, 400, 1400, "波斯",
     "Religious center and caravan stop south of Rayy", 60000, 1800, 14),
    ("Kashan", "卡尚", 51.45, 33.98, -100, 200, 1500, "波斯",
     "Known for textiles and ceramics on the Silk Road", 55000, 1600, 13),
    ("Zaranj", "扎兰吉", 62.00, 31.00, -300, -100, 1100, "波斯",
     "Sistan capital, controlling trade routes to India", 40000, 1400, 18),
    ("Sari", "萨里", 53.06, 36.56, -100, 200, 1300, "波斯",
     "Caspian trade city on the northern Persian route", 45000, 1500, 16),

    ("Baghdad", "巴格达", 44.37, 33.31, 762, 800, 1258, "美索不达米亚",
     "Abbasid capital, center of the Islamic Golden Age", 1000000, 9000, 1),
    ("Ctesiphon", "泰西封", 44.58, 33.09, -200, -100, 637, "美索不达米亚",
     "Parthian and Sassanid capital on the Tigris", 300000, 5500, 3),
    ("Babylon", "巴比伦", 44.42, 32.54, -2000, -600, -100, "美索不达米亚",
     "Ancient city of legendary wealth, early Silk Road terminus", 200000, 4000, 5),
    ("Mosul", "摩苏尔", 43.13, 36.35, -300, 100, 1400, "美索不达米亚",
     "Major trading city on the Tigris, crossroads of empires", 150000, 4200, 6),
    ("Basra", "巴士拉", 47.78, 30.51, 636, 700, 1300, "美索不达米亚",
     "Port city connecting Persian Gulf to overland routes", 200000, 5000, 2),
    ("Kirkuk", "基尔库克", 44.39, 35.47, -200, 100, 1200, "美索不达米亚",
     "Ancient city controlling northern Mesopotamian routes", 60000, 2000, 12),
    ("Erbil", "埃尔比勒", 44.01, 36.19, -500, -200, 1400, "美索不达米亚",
     "One of the oldest continuously inhabited cities", 80000, 2500, 9),
    ("Dura-Europos", "杜拉-欧罗普斯", 40.73, 35.00, -300, -200, 256, "美索不达米亚",
     "Frontier city on the Euphrates, melting pot of cultures", 30000, 1200, 15),
    ("Hatra", "哈特拉", 42.72, 35.58, -200, -100, 241, "美索不达米亚",
     "Fortified caravan city between Roman and Parthian empires", 50000, 1800, 11),
    ("Samarra", "萨马拉", 43.88, 34.20, 836, 850, 900, "美索不达米亚",
     "Abbasid capital briefly, massive palace complex", 100000, 3000, 7),
    ("Kufa", "库法", 44.40, 32.10, 638, 700, 900, "美索不达米亚",
     "Early Islamic garrison city and intellectual center", 80000, 2800, 10),
    ("Seleucia", "塞琉西亚", 44.55, 33.10, -300, -200, 200, "美索不达米亚",
     "Hellenistic city across the Tigris from Ctesiphon", 100000, 3500, 8),

    ("Antioch", "安条克", 36.17, 36.20, -300, -200, 1100, "地中海东岸",
     "Great metropolis, terminus of overland Silk Road to Mediterranean", 300000, 7500, 1),
    ("Palmyra", "巴尔米拉", 38.27, 34.55, -200, -100, 273, "地中海东岸",
     "Queen of the Desert, caravan city of legendary wealth", 200000, 8500, 2),
    ("Damascus", "大马士革", 36.29, 33.51, -1000, -500, 1500, "地中海东岸",
     "One of the oldest cities, key trade nexus", 250000, 6000, 3),
    ("Aleppo", "阿勒颇", 37.16, 36.20, -500, -200, 1500, "地中海东岸",
     "Great trading city at crossroads of empires", 200000, 5500, 4),
    ("Tyre", "提尔", 35.20, 33.27, -1000, -500, 800, "地中海东岸",
     "Phoenician port, purple dye and maritime trade", 80000, 4000, 8),
    ("Sidon", "西顿", 35.37, 33.56, -1000, -500, 700, "地中海东岸",
     "Ancient Phoenician port city, glass production", 50000, 2500, 12),
    ("Petra", "佩特拉", 35.44, 30.33, -300, -100, 400, "地中海东岸",
     "Nabataean rock city controlling incense trade routes", 30000, 3500, 7),
    ("Jerusalem", "耶路撒冷", 35.22, 31.78, -1000, -500, 1500, "地中海东岸",
     "Holy city, pilgrimage and trade center", 100000, 3000, 6),
    ("Caesarea", "凯撒利亚", 34.89, 32.50, -22, 0, 700, "地中海东岸",
     "Roman-built port, gateway to the Mediterranean", 50000, 2500, 10),
    ("Acre", "阿卡", 35.07, 32.92, -200, -50, 1300, "地中海东岸",
     "Crusader port city on the Mediterranean coast", 60000, 2800, 9),
    ("Gaza", "加沙", 34.35, 31.38, -500, -200, 1200, "地中海东岸",
     "Caravan city connecting Egypt to the Levant", 70000, 3200, 11),
    ("Homs", "霍姆斯", 36.72, 34.73, -200, 100, 1300, "地中海东岸",
     "Orontes valley city on the Silk Road branch to coast", 60000, 2000, 13),

    ("Constantinople", "君士坦丁堡", 28.98, 41.01, 330, 400, 1453, "安纳托利亚",
     "Byzantine capital, gateway between Europe and Asia", 500000, 9500, 1),
    ("Ephesus", "以弗所", 27.34, 38.00, -1000, -500, 600, "安纳托利亚",
     "Great Ionian city, temple of Artemis, major port", 200000, 4500, 3),
    ("Sardis", "萨迪斯", 28.04, 38.49, -600, -300, 600, "安纳托利亚",
     "Lydian capital, western terminus of Royal Road", 80000, 3000, 7),
    ("Tarsus", "塔尔苏斯", 35.00, 37.00, -500, -200, 1100, "安纳托利亚",
     "Birthplace of Paul, Cilician gateway city", 100000, 3200, 5),
    ("Ankara", "安卡拉", 32.86, 39.93, -200, 50, 1200, "安纳托利亚",
     "Crossroads city in central Anatolia", 80000, 2500, 6),
    ("Trabzon", "特拉布宗", 39.72, 41.00, -700, -400, 1400, "安纳托利亚",
     "Black Sea port, terminus of trade route from Persia", 70000, 3500, 4),
    ("Konya", "科尼亚", 32.48, 37.87, -200, 100, 1400, "安纳托利亚",
     "Iconium, Seljuk capital on the Silk Road", 100000, 3000, 5),
    ("Kayseri", "开塞利", 35.47, 38.73, -100, 100, 1300, "安纳托利亚",
     "Cappadocian city, caravanserai hub", 70000, 2200, 8),
    ("Antalya", "安塔利亚", 30.71, 36.89, -200, -50, 1300, "安纳托利亚",
     "Pamphylian port city on Mediterranean coast", 60000, 2000, 10),
    ("Sivas", "锡瓦斯", 37.02, 39.75, -100, 200, 1400, "安纳托利亚",
     "Anatolian crossroads, important under Seljuks", 50000, 1800, 9),

    ("Rome", "罗马", 12.50, 41.90, -753, -200, 476, "地中海西端",
     "Eternal City, ultimate western destination of Silk Road goods", 1000000, 8000, 1),
    ("Alexandria", "亚历山大", 29.92, 31.20, -331, -200, 1000, "地中海西端",
     "Great Hellenistic port, center of learning and trade", 500000, 9000, 2),
    ("Athens", "雅典", 23.73, 37.97, -1000, -500, 500, "地中海西端",
     "Birthplace of democracy, philosophical center", 200000, 3000, 8),
    ("Carthage", "迦太基", 10.33, 36.86, -800, -500, 146, "地中海西端",
     "Great maritime power, western Mediterranean trade hub", 300000, 5000, 5),
    ("Syracuse", "叙拉古", 15.29, 37.07, -700, -400, 300, "地中海西端",
     "Greek city-state in Sicily, scientific center", 150000, 2500, 10),
    ("Ravenna", "拉文纳", 12.20, 44.42, -50, 400, 750, "地中海西端",
     "Western Roman and Byzantine capital in Italy", 50000, 2000, 12),
    ("Naples", "那不勒斯", 14.25, 40.84, -600, -300, 1000, "地中海西端",
     "Major Italian port handling eastern luxury goods", 100000, 3000, 7),
    ("Thessaloniki", "塞萨洛尼基", 22.94, 40.64, -315, -100, 1400, "地中海西端",
     "Byzantine second city, Balkan trade hub", 120000, 3500, 6),
    ("Venice", "威尼斯", 12.34, 45.44, 421, 800, 1450, "地中海西端",
     "Maritime republic dominating eastern Mediterranean trade", 150000, 8500, 3),
    ("Genoa", "热那亚", 8.94, 44.41, -200, 100, 1450, "地中海西端",
     "Maritime republic rivaling Venice for eastern trade", 100000, 5500, 4),
    ("Milan", "米兰", 9.19, 45.46, -400, -100, 1400, "地中海西端",
     "Imperial city, overland trade hub in northern Italy", 120000, 4000, 9),
]

# =========================================
# 气候相关
# =========================================
REGIONS = ["关中", "河西走廊", "塔里木盆地", "河中地区", "波斯", "美索不达米亚", "地中海东岸", "安纳托利亚"]

# 气候阶段定义（基于历史气候重建）
CLIMATE_PHASES = [
    {"name": "Roman Warm Period", "start": -250, "end": 400, "temp_anomaly": 0.8, "precip_shift": 0.2},
    {"name": "Late Antique Little Ice Age", "start": 400, "end": 660, "temp_anomaly": -1.2, "precip_shift": -0.3},
    {"name": "Early Medieval Cool Period", "start": 660, "end": 900, "temp_anomaly": 0.0, "precip_shift": 0.1},
    {"name": "Medieval Warm Period", "start": 900, "end": 1300, "temp_anomaly": 0.7, "precip_shift": 0.3},
    {"name": "Little Ice Age Onset", "start": 1300, "end": 1500, "temp_anomaly": -0.6, "precip_shift": -0.2},
]

CLIMATE_PERIODS = []
year = -200
while year < 1500:
    end = min(year + 50, 1500)
    CLIMATE_PERIODS.append((year, end))
    year = end


def get_climate_phase(mid_year):
    """获取某一年份所属的气候阶段"""
    for phase in CLIMATE_PHASES:
        if phase["start"] <= mid_year < phase["end"]:
            return phase
    return CLIMATE_PHASES[-1]


def get_region_climate_profile(region):
    """各区域的气候基准特征：(温度调整, 降水基数, 降水波动)"""
    profiles = {
        "关中":        {"temp_adj": 0.3, "precip_base": 0.9, "precip_var": 0.6, "glacier_thresh": -0.3},
        "河西走廊":    {"temp_adj": 0.0, "precip_base": 0.6, "precip_var": 0.5, "glacier_thresh": -0.5},
        "塔里木盆地":  {"temp_adj": -0.2, "precip_base": 0.3, "precip_var": 0.4, "glacier_thresh": -0.4},
        "河中地区":    {"temp_adj": 0.1, "precip_base": 0.7, "precip_var": 0.5, "glacier_thresh": -0.6},
        "波斯":        {"temp_adj": 0.2, "precip_base": 0.6, "precip_var": 0.4, "glacier_thresh": -0.5},
        "美索不达米亚": {"temp_adj": 0.5, "precip_base": 0.5, "precip_var": 0.4, "glacier_thresh": -0.7},
        "地中海东岸":   {"temp_adj": 0.4, "precip_base": 0.7, "precip_var": 0.3, "glacier_thresh": -0.6},
        "安纳托利亚":   {"temp_adj": 0.1, "precip_base": 0.8, "precip_var": 0.4, "glacier_thresh": -0.4},
    }
    return profiles.get(region, {"temp_adj": 0.0, "precip_base": 0.7, "precip_var": 0.5, "glacier_thresh": -0.5})


def generate_climate_data():
    """生成古气候数据：温度距平、降水指数、冰川标记"""
    rows = []
    for region in REGIONS:
        profile = get_region_climate_profile(region)
        for ps, pe in CLIMATE_PERIODS:
            mid = (ps + pe) / 2.0
            phase = get_climate_phase(mid)

            # 温度距平：阶段值 + 区域调整 + 随机波动
            temp_anomaly = round(
                phase["temp_anomaly"] + profile["temp_adj"] + random.uniform(-0.25, 0.25),
                2
            )

            # 降水指数：基准 + 阶段降水偏移 + 区域波动 + 随机
            precip = round(
                max(0.3, min(1.8,
                    profile["precip_base"] + phase["precip_shift"] +
                    random.uniform(-profile["precip_var"] * 0.3, profile["precip_var"] * 0.3)
                )),
                2
            )

            # 冰川前进判定
            glacier = temp_anomaly < profile["glacier_thresh"]

            # 注释
            notes_parts = []
            notes_parts.append(phase["name"])
            if glacier:
                if 530 < mid < 660:
                    notes_parts.append("Late Antique Little Ice Age glacier advance")
                elif 1300 < mid <= 1500:
                    notes_parts.append("Early LIA glacier advance")
                else:
                    notes_parts.append("Cold period glacial advance")
            if region == "塔里木盆地" and precip < 0.5:
                notes_parts.append("Severe aridity event")
            if region == "关中" and precip > 1.2:
                notes_parts.append("Enhanced East Asian monsoon")

            notes = "; ".join(notes_parts)

            rows.append((
                ps, pe, region,
                temp_anomaly, precip,
                glacier,
                "glacial advance" if glacier else None,
                phase["name"],
                notes
            ))
    return rows


# =========================================
# 贸易数据
# =========================================
def haversine_km(lon1, lat1, lon2, lat2):
    """计算两点间球面距离（公里）"""
    R = 6371.0
    lat1_rad = math.radians(lat1)
    lat2_rad = math.radians(lat2)
    dlat = math.radians(lat2 - lat1)
    dlon = math.radians(lon2 - lon1)
    a = math.sin(dlat / 2) ** 2 + math.cos(lat1_rad) * math.cos(lat2_rad) * math.sin(dlon / 2) ** 2
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    return R * c


def make_linestring_wkt(lon1, lat1, lon2, lat2, midpoints=3):
    """生成两点间的LineString WKT，加入中间弯曲点使路线更自然"""
    points = [(lon1, lat1)]
    for i in range(1, midpoints + 1):
        t = i / (midpoints + 1)
        mid_lon = lon1 + (lon2 - lon1) * t + random.uniform(-0.5, 0.5)
        mid_lat = lat1 + (lat2 - lat1) * t + random.uniform(-0.3, 0.3)
        points.append((mid_lon, mid_lat))
    points.append((lon2, lat2))
    coords = ", ".join(f"{lon} {lat}" for lon, lat in points)
    return f"SRID=4326;LINESTRING({coords})"


def get_city_dynasty_activity(city, dynasty):
    """计算某城市在某朝代的活跃系数（0-1）"""
    c_founded, c_pstart, c_pend = city[4], city[5], city[6]
    d_start, d_end = dynasty["start"], dynasty["end"]

    if c_pend < d_start or c_pstart > d_end:
        return 0.0

    overlap_start = max(c_pstart, d_start)
    overlap_end = min(c_pend, d_end)
    overlap = max(0, overlap_end - overlap_start)
    dynasty_len = d_end - d_start

    if dynasty_len <= 0:
        return 0.0
    return min(1.0, overlap / dynasty_len)


def generate_trade_connections(city_records, city_ids):
    """按朝代生成贸易连接数据"""
    connections = []
    region_cities = {}
    for i, c in enumerate(city_records):
        region = c[7]
        region_cities.setdefault(region, []).append(i)

    region_order = ["关中", "河西走廊", "塔里木盆地", "河中地区", "波斯",
                    "美索不达米亚", "地中海东岸", "安纳托利亚", "地中海西端"]

    # 每个朝代生成贸易网络
    for dynasty in DYNASTIES:
        ps, pe = dynasty["start"], dynasty["end"]
        trade_factor = dynasty["trade_factor"]
        sea_factor = dynasty["sea_factor"]

        # 区域间贸易（丝绸之路主干道）
        for ri in range(len(region_order) - 1):
            r1 = region_order[ri]
            r2 = region_order[ri + 1]
            c1_list = region_cities.get(r1, [])
            c2_list = region_cities.get(r2, [])
            if not c1_list or not c2_list:
                continue

            # 根据朝代贸易强度决定连接对数
            base_pairs = int(4 * trade_factor)
            n_pairs = random.randint(max(2, base_pairs - 1), min(9, base_pairs + 2))
            n_pairs = min(n_pairs, len(c1_list), len(c2_list))

            chosen1 = random.sample(c1_list, n_pairs)
            chosen2 = random.sample(c2_list, n_pairs)

            for ci1 in chosen1:
                for ci2 in chosen2:
                    # 计算城市活跃系数
                    activity1 = get_city_dynasty_activity(city_records[ci1], dynasty)
                    activity2 = get_city_dynasty_activity(city_records[ci2], dynasty)
                    if activity1 < 0.3 or activity2 < 0.3:
                        continue

                    # 基础贸易量 + 随机
                    base_vol = random.randint(600, 5500)
                    vol = int(base_vol * trade_factor * (activity1 + activity2) / 2)

                    # 路线类型
                    is_sea = r2 in ("地中海西端", "地中海东岸") and random.random() < (0.2 * sea_factor)
                    route_type = "sea" if is_sea else "land"

                    # 商品类型
                    commodity = random.choice(COMMODITIES)

                    # 可靠性评分
                    reliability = round(random.uniform(0.6, 1.0), 2)

                    # 空间几何
                    lon1, lat1 = city_records[ci1][2], city_records[ci1][3]
                    lon2, lat2 = city_records[ci2][2], city_records[ci2][3]
                    geom = make_linestring_wkt(lon1, lat1, lon2, lat2)

                    connections.append((
                        city_ids[ci1], city_ids[ci2],
                        ps, pe, vol, route_type,
                        commodity, dynasty["name"], reliability,
                        geom
                    ))

        # 区域内贸易
        for region, city_idx_list in region_cities.items():
            if len(city_idx_list) < 2:
                continue
            n_intra = random.randint(2, min(6, len(city_idx_list) // 2 + 1))
            for _ in range(n_intra):
                a, b = random.sample(city_idx_list, 2)
                activity_a = get_city_dynasty_activity(city_records[a], dynasty)
                activity_b = get_city_dynasty_activity(city_records[b], dynasty)
                if activity_a < 0.3 or activity_b < 0.3:
                    continue

                vol = int(random.randint(300, 3000) * trade_factor * (activity_a + activity_b) / 2)
                commodity = random.choice(COMMODITIES)
                reliability = round(random.uniform(0.7, 1.0), 2)

                lon1, lat1 = city_records[a][2], city_records[a][3]
                lon2, lat2 = city_records[b][2], city_records[b][3]
                geom = make_linestring_wkt(lon1, lat1, lon2, lat2, midpoints=1)

                connections.append((
                    city_ids[a], city_ids[b],
                    ps, pe, vol, "land",
                    commodity, dynasty["name"], reliability,
                    geom
                ))

    # 去重（保留贸易量更大的）
    deduped = {}
    for conn in connections:
        c1, c2, ps, pe = conn[0], conn[1], conn[2], conn[3]
        key = (min(c1, c2), max(c1, c2), ps, pe)
        if key not in deduped or conn[4] > deduped[key][4]:
            deduped[key] = conn

    return list(deduped.values())


# =========================================
# 城市衰落赋值
# =========================================
def assign_decline(prosperity_end, prosperity_start, city_name=None):
    """为城市分配衰落年份和原因"""
    r = random.random()

    # 10% 城市不衰落
    if r < 0.10:
        return None, None

    max_decline = min(prosperity_end + 350, 1550)
    min_decline = prosperity_end

    if min_decline >= max_decline:
        decline_year = prosperity_end + random.randint(10, 50)
    else:
        decline_year = random.randint(min_decline, max_decline)

    # 原因分布
    if r < 0.10 + 0.38:
        reason = "war"
    elif r < 0.10 + 0.38 + 0.30:
        reason = "trade_route"
    else:
        reason = "climate"

    return decline_year, reason


# =========================================
# 主流程
# =========================================
def main():
    random.seed(RANDOM_SEED)
    print("=" * 60)
    print("丝绸之路古代城市数据模拟器")
    print(f"随机种子: {RANDOM_SEED}")
    print(f"数据库: {DB_HOST}:{DB_PORT}/{DB_NAME}")
    print("=" * 60)

    # 连接数据库（带重试）
    print("\n[1/5] 连接数据库...")
    max_retries = 10
    conn = None
    for attempt in range(1, max_retries + 1):
        try:
            conn = psycopg2.connect(DATABASE_URL)
            conn.autocommit = True
            break
        except psycopg2.OperationalError as e:
            if attempt < max_retries:
                print(f"  连接失败 ({attempt}/{max_retries})，2秒后重试...")
                import time
                time.sleep(2)
            else:
                print(f"  错误：无法连接数据库: {e}")
                sys.exit(1)

    cur = conn.cursor()
    print("  ✓ 数据库连接成功")

    # 检查表是否存在（如果已有数据则跳过）
    cur.execute("""
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'cities'
        );
    """)
    tables_exist = cur.fetchone()[0]
    if tables_exist:
        cur.execute("SELECT COUNT(*) FROM cities;")
        count = cur.fetchone()[0]
        if count > 0:
            print(f"  ⚠ 数据库已存在 {count} 条城市数据，跳过数据生成")
            cur.close()
            conn.close()
            print("\n数据生成完成（已有数据，未覆盖）。")
            return

    # 插入城市
    print(f"\n[2/5] 插入 {len(CITIES_RAW)} 个城市...")
    city_ids = []
    for i, c in enumerate(CITIES_RAW):
        name, name_cn, lon, lat, founded, p_start, p_end, region, desc, pop, trade, rank = c
        decline_year, decline_reason = assign_decline(p_end, p_start, name)
        pop_var = int(pop + random.randint(-int(pop * 0.15), int(pop * 0.15)))
        trade_var = int(trade + random.randint(-int(trade * 0.1), int(trade * 0.1)))

        # 确定主要朝代
        dynasty_name = None
        for d in DYNASTIES:
            if d["start"] <= p_start <= d["end"] or d["start"] <= p_end <= d["end"]:
                dynasty_name = d["name"]
                break

        cur.execute("""
            INSERT INTO cities (
                name, name_cn, longitude, latitude, geom,
                founded_year, prosperity_start, prosperity_end,
                decline_year, decline_reason, region, dynasty, description,
                population_peak, trade_volume, importance_rank
            ) VALUES (
                %s, %s, %s, %s,
                ST_SetSRID(ST_MakePoint(%s, %s), 4326),
                %s, %s, %s, %s, %s, %s, %s, %s, %s, %s
            ) RETURNING id;
        """, (
            name, name_cn, lon, lat, lon, lat,
            founded, p_start, p_end,
            decline_year, decline_reason,
            region, dynasty_name, desc,
            pop_var, trade_var, rank
        ))
        city_ids.append(cur.fetchone()[0])
        if (i + 1) % 20 == 0:
            print(f"  已插入 {i+1}/{len(CITIES_RAW)} 个城市")
    print(f"  ✓ {len(CITIES_RAW)} 个城市插入完成")

    # 插入气候数据
    print(f"\n[3/5] 生成并插入气候数据...")
    climate_rows = generate_climate_data()
    for i, row in enumerate(climate_rows):
        cur.execute("""
            INSERT INTO climate_data (
                period_start, period_end, region,
                temperature_anomaly, precipitation_index,
                glacier_advance, glacier_note, climate_phase, notes
            ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s);
        """, row)
        if (i + 1) % 50 == 0:
            print(f"  已插入 {i+1}/{len(climate_rows)} 条气候记录")
    print(f"  ✓ {len(climate_rows)} 条气候记录插入完成")

    # 生成贸易连接
    print("\n[4/5] 生成贸易连接网络...")
    city_records = []
    for i, c in enumerate(CITIES_RAW):
        city_records.append((
            c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]
        ))
    trade_rows = generate_trade_connections(city_records, city_ids)

    print(f"  共生成 {len(trade_rows)} 条贸易连接，开始插入...")
    for i, conn in enumerate(trade_rows):
        cur.execute("""
            INSERT INTO trade_connections (
                city_from, city_to, period_start, period_end,
                trade_volume, route_type, commodity, dynasty,
                reliability_score, geom
            ) VALUES (
                %s, %s, %s, %s, %s, %s, %s, %s, %s,
                ST_GeomFromEWKT(%s)
            );
        """, conn)
        if (i + 1) % 200 == 0:
            print(f"  已插入 {i+1}/{len(trade_rows)} 条连接")
    print(f"  ✓ {len(trade_rows)} 条贸易连接插入完成")

    # 验证数据
    print("\n[5/5] 数据验证...")
    cur.execute("SELECT COUNT(*) FROM cities;")
    n_cities = cur.fetchone()[0]
    print(f"  cities: {n_cities} 行")

    cur.execute("SELECT COUNT(*) FROM climate_data;")
    n_climate = cur.fetchone()[0]
    print(f"  climate_data: {n_climate} 行")

    cur.execute("SELECT COUNT(*) FROM trade_connections;")
    n_trade = cur.fetchone()[0]
    print(f"  trade_connections: {n_trade} 行")

    # 衰落原因分布
    cur.execute("""
        SELECT decline_reason, COUNT(*) FROM cities
        WHERE decline_reason IS NOT NULL
        GROUP BY decline_reason ORDER BY COUNT(*) DESC;
    """)
    print("\n  城市衰落原因分布:")
    for reason, cnt in cur.fetchall():
        pct = round(cnt / n_cities * 100, 1)
        reason_cn = {"war": "战争", "climate": "气候", "trade_route": "贸易路线变迁"}.get(reason, reason)
        print(f"    {reason_cn}: {cnt} ({pct}%)")

    # 朝代贸易量统计
    cur.execute("""
        SELECT dynasty, COUNT(*), SUM(trade_volume)
        FROM trade_connections
        WHERE dynasty IS NOT NULL
        GROUP BY dynasty ORDER BY MIN(period_start);
    """)
    print("\n  各朝代贸易统计:")
    for dynasty, count, total_vol in cur.fetchall():
        print(f"    {dynasty}: {count} 条连接, 贸易总量 {total_vol:,}")

    # 空间索引验证
    cur.execute("""
        SELECT indexname FROM pg_indexes
        WHERE tablename = 'cities' AND indexname LIKE '%geom%';
    """)
    geom_idx = cur.fetchone()
    if geom_idx:
        print(f"\n  ✓ 空间索引已建立: {geom_idx[0]}")
    else:
        print(f"\n  ⚠ 未找到空间索引")

    cur.close()
    conn.close()
    print("\n" + "=" * 60)
    print("数据生成完成！")
    print(f"  城市: {n_cities} 个")
    print(f"  气候记录: {n_climate} 条")
    print(f"  贸易连接: {n_trade} 条")
    print(f"  朝代: {len(DYNASTIES)} 个")
    print("=" * 60)


if __name__ == "__main__":
    main()
