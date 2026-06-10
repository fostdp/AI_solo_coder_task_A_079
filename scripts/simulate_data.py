# -*- coding: utf-8 -*-
import psycopg2
import random
import math

DATABASE_URL = "postgresql://postgres:postgres@localhost:5432/silk_road"

CITIES_RAW = [
    ("Chang'an", "长安", 108.94, 34.26, -1046, -202, 907, "关中",
     "Capital of Han and Tang dynasties, eastern terminus of the Silk Road", 1000000, 9500),
    ("Xianyang", "咸阳", 108.71, 34.33, -350, -221, -206, "关中",
     "Capital of Qin dynasty, predecessor to Chang'an", 500000, 6800),
    ("Yongcheng", "雍城", 107.39, 34.52, -771, -677, -383, "关中",
     "Capital of Qin state during Spring and Autumn period", 120000, 3200),
    ("Huayin", "华阴", 110.09, 34.57, -200, -100, 600, "关中",
     "Gateway city at the foot of Mount Hua", 80000, 2800),
    ("Baoji", "宝鸡", 107.14, 34.37, -600, -400, 400, "关中",
     "Strategic western gateway of Guanzhong", 150000, 4100),
    ("Fengxiang", "凤翔", 107.40, 34.52, -677, -600, 200, "关中",
     "Ancient Qin capital, later important market town", 90000, 2500),
    ("Tongchuan", "铜川", 108.95, 34.90, -150, 50, 700, "关中",
     "Ceramics production center on Silk Road corridor", 60000, 1800),
    ("Lintong", "临潼", 109.21, 34.37, -300, -200, 800, "关中",
     "Site of Terracotta Army, important garrison town", 110000, 3000),
    ("Pucheng", "蒲城", 109.59, 34.95, -100, 100, 600, "关中",
     "Agricultural hub and military depot", 70000, 2000),
    ("Weinan", "渭南", 109.50, 34.50, -200, -50, 900, "关中",
     "Major trading post on Wei River valley route", 130000, 3500),

    ("Lanzhou", "兰州", 103.83, 36.06, -86, 0, 1500, "河西走廊",
     "Major crossing point of the Yellow River on the Silk Road", 400000, 7200),
    ("Tianshui", "天水", 105.72, 34.58, -300, -100, 1200, "河西走廊",
     "Ancient Qin state origin, key Silk Road station", 200000, 4500),
    ("Wuwei", "武威", 102.64, 37.93, -121, 100, 1100, "河西走廊",
     "Liangzhou, major military and trade center", 250000, 5800),
    ("Zhangye", "张掖", 100.45, 38.93, -111, 50, 1200, "河西走廊",
     "Ganzhou, vital oasis city on Hexi Corridor", 180000, 4200),
    ("Jiuquan", "酒泉", 98.49, 39.74, -111, 50, 1300, "河西走廊",
     "Suzhou, western gateway of Hexi Corridor", 120000, 3500),
    ("Dunhuang", "敦煌", 94.66, 40.14, -111, -50, 1400, "河西走廊",
     "Critical junction where Silk Road splits into north and south routes", 150000, 8500),
    ("Jiayuguan", "嘉峪关", 98.29, 39.77, 1372, 1400, 1500, "河西走廊",
     "Western end of Great Wall, Ming dynasty fortress", 80000, 2200),
    ("Baiyin", "白银", 104.18, 36.55, -50, 200, 900, "河西走廊",
     "Mining town and caravan stop", 60000, 1500),
    ("Jinchang", "金昌", 102.19, 38.52, -80, 50, 800, "河西走廊",
     "Frontier garrison and trade outpost", 50000, 1200),
    ("Yongchang", "永昌", 101.97, 38.33, -80, 100, 700, "河西走廊",
     "Important military colony on the corridor", 45000, 1100),
    ("Minqin", "民勤", 103.09, 38.62, -120, -50, 800, "河西走廊",
     "Oasis town between deserts", 40000, 900),
    ("Shandan", "山丹", 101.09, 38.78, -100, 50, 900, "河西走廊",
     "Pastoral and military town along the corridor", 35000, 800),
    ("Guazhou", "瓜州", 95.78, 40.52, -100, -20, 1200, "河西走廊",
     "Oasis town known for melons, waypoint to Dunhuang", 55000, 2800),
    ("Yumen", "玉门", 97.04, 40.28, -100, -20, 900, "河西走廊",
     "Jade Gate Pass, legendary Silk Road checkpoint", 40000, 2000),
    ("Anxi", "安西", 95.78, 40.52, -80, 100, 1000, "河西走廊",
     "Military outpost guarding route to the west", 30000, 1500),

    ("Loulan", "楼兰", 89.80, 40.51, -200, -176, 400, "塔里木盆地",
     "Mysterious kingdom lost to desert, crucial Silk Road post", 50000, 4500),
    ("Khotan", "于阗", 79.93, 37.11, -250, -100, 1000, "塔里木盆地",
     "Famous for jade and silk production on southern route", 120000, 6200),
    ("Kucha", "龟兹", 82.93, 41.72, -200, -100, 900, "塔里木盆地",
     "Buddhist cultural center and music hub on northern route", 100000, 5500),
    ("Kashgar", "疏勒", 75.99, 39.47, -200, -128, 1500, "塔里木盆地",
     "Key junction where routes converge before crossing Pamirs", 200000, 7800),
    ("Gaochang", "高昌", 89.55, 42.95, -60, 100, 1400, "塔里木盆地",
     "Turpan oasis kingdom, strategic northern route city", 80000, 3800),
    ("Qiemo", "且末", 85.53, 38.15, -200, -150, 700, "塔里木盆地",
     "Small kingdom on southern Silk Road route", 30000, 1200),
    ("Yanqi", "焉耆", 86.54, 42.06, -200, -100, 800, "塔里木盆地",
     "Buddhist kingdom on the northern route", 60000, 2800),
    ("Yarkand", "莎车", 77.28, 38.42, -150, -50, 1500, "塔里木盆地",
     "Major trading center on southern route to Central Asia", 130000, 5200),
    ("Niya", "精绝", 82.70, 37.93, -200, -150, 300, "塔里木盆地",
     "Lost oasis kingdom revealed by archaeology", 20000, 800),
    ("Pishan", "皮山", 78.30, 37.62, -150, -50, 900, "塔里木盆地",
     "Waypoint on the southern route near Khotan", 25000, 1000),
    ("Ruoqiang", "婼羌", 88.50, 39.03, -200, -100, 600, "塔里木盆地",
     "Ancient kingdom near Lop Nur", 15000, 600),
    ("Yuli", "尉犁", 86.26, 41.35, -100, 50, 700, "塔里木盆地",
     "Frontier garrison near Kucha", 20000, 800),
    ("Luntai", "轮台", 84.25, 41.79, -60, 100, 800, "塔里木盆地",
     "Han dynasty military colony, later Tang garrison", 35000, 1500),
    ("Aksu", "姑墨", 80.27, 41.17, -150, -50, 1200, "塔里木盆地",
     "Oasis city between Kashgar and Kucha on northern route", 70000, 3200),
    ("Hami", "伊吾", 93.51, 42.83, -100, 50, 1400, "塔里木盆地",
     "Eastern gateway to Tarim Basin from Hexi Corridor", 60000, 2500),
    ("Jimsar", "庭州", 89.18, 44.00, -60, 200, 800, "塔里木盆地",
     "Beiting, Tang dynasty protectorate headquarters", 50000, 2200),
    ("Wensu", "温宿", 80.24, 41.28, -150, -50, 900, "塔里木盆地",
     "Agricultural oasis near Aksu", 30000, 1100),
    ("Pulei", "蒲类", 93.50, 43.80, -100, 50, 700, "塔里木盆地",
     "Nomadic territory near Hami", 15000, 600),
    ("Yutou", "尉头", 79.00, 40.50, -150, -50, 600, "塔里木盆地",
     "Mountain pass outpost between Kashgar and Pamirs", 10000, 450),
    ("Quli", "渠犁", 84.50, 41.50, -60, 100, 600, "塔里木盆地",
     "Agricultural settlement near Luntai", 14000, 550),

    ("Samarkand", "撒马尔罕", 66.96, 39.65, -700, -300, 1500, "河中地区",
     "Jewel of the Silk Road, center of Sogdian merchant network", 300000, 9000),
    ("Bukhara", "布哈拉", 64.42, 39.77, -500, -200, 1500, "河中地区",
     "Major Islamic intellectual and trade center", 250000, 7500),
    ("Khwarezm", "花剌子模", 59.05, 41.38, -700, -400, 1200, "河中地区",
     "Delta kingdom controlling Amu Darya trade routes", 150000, 4500),
    ("Khujand", "忽毡", 69.60, 40.30, -500, -200, 1400, "河中地区",
     "Key city at Fergana Valley entrance, Alexander's extreme", 100000, 3800),
    ("Fergana", "费尔干纳", 71.85, 40.40, -400, -200, 1500, "河中地区",
     "Valley of horses, source of Ferghana horses for China", 80000, 3200),
    ("Termez", "铁尔梅兹", 67.27, 37.22, -300, -100, 1200, "河中地区",
     "Buddhist center and Amu Darya crossing point", 70000, 2800),
    ("Tashkent", "塔什干", 69.28, 41.31, -300, -100, 1500, "河中地区",
     "Major oasis city at crossroads of Central Asian routes", 180000, 5000),
    ("Penjikent", "片治肯特", 67.73, 39.50, -400, -200, 800, "河中地区",
     "Sogdian city with famous frescoes and temples", 40000, 1800),
    ("Shahrisabz", "沙赫里萨布兹", 66.83, 39.05, -500, -200, 1400, "河中地区",
     "Birthplace of Timur, ancient Nautaca", 60000, 2200),
    ("Karshi", "卡尔希", 65.72, 38.86, -300, -100, 1300, "河中地区",
     "Sogdian trading city known as Nasaf", 50000, 1800),
    ("Hissar", "希萨尔", 68.55, 38.50, -200, 100, 1300, "河中地区",
     "Mountain fortress guarding Samarkand approach", 30000, 1000),
    ("Jizzakh", "吉扎克", 67.84, 40.10, -200, 100, 1200, "河中地区",
     "Caravanserai town between Samarkand and Tashkent", 35000, 1200),
    ("Chaghaniyan", "赤鄂衍那", 67.50, 38.00, -300, -100, 800, "河中地区",
     "Small principality on the Surkhan Darya", 20000, 800),
    ("Shugnan", "识匿", 71.50, 37.60, -100, 200, 900, "河中地区",
     "Mountain kingdom controlling Pamir passes", 15000, 600),
    ("Andijan", "宁远", 72.34, 40.78, -200, 100, 1400, "河中地区",
     "Eastern Fergana city, gateway to Kashgar", 55000, 2000),

    ("Merv", "木鹿", 62.18, 37.66, -600, -300, 1200, "波斯",
     "Great oasis city, one of the largest in the medieval world", 250000, 7000),
    ("Nishapur", "尼沙普尔", 58.80, 36.21, -300, -100, 1200, "波斯",
     "Major trading city in Khorasan, turquoise capital", 150000, 4800),
    ("Herat", "赫拉特", 62.20, 34.34, -500, -200, 1500, "波斯",
     "Key city on route to India, center of arts and learning", 180000, 5200),
    ("Balkh", "巴里黑", 66.90, 36.77, -600, -400, 1200, "波斯",
     "Ancient Bactra, mother of cities, Zoroastrian center", 200000, 6000),
    ("Rayy", "雷伊", 51.42, 35.69, -300, -100, 1200, "波斯",
     "Major city near modern Tehran, important Silk Road node", 150000, 4500),
    ("Isfahan", "伊斯法罕", 51.67, 32.65, -200, 100, 1500, "波斯",
     "Half the world, legendary center of Persian culture", 200000, 5500),
    ("Tabriz", "大不里士", 46.30, 38.08, -100, 200, 1500, "波斯",
     "Major trading city connecting Persia to Anatolia", 180000, 5000),
    ("Yazd", "亚兹德", 54.37, 31.90, -300, 0, 1500, "波斯",
     "Zoroastrian center, famous for silk and textiles", 80000, 2800),
    ("Kerman", "克尔曼", 57.08, 30.28, -200, 100, 1400, "波斯",
     "Gateway to Persian Gulf ports, caravan city", 70000, 2500),
    ("Sistan", "锡斯坦", 61.50, 31.00, -400, -200, 1000, "波斯",
     "Ancient Drangiana, crossroads of Central and South Asian routes", 50000, 1800),
    ("Gurganj", "古尔甘吉", 59.10, 42.30, -400, -200, 1200, "波斯",
     "Capital of Khwarezm, major trading emporium on Amu Darya", 120000, 3800),
    ("Qom", "库姆", 50.88, 34.64, 100, 400, 1400, "波斯",
     "Religious center and caravan stop south of Rayy", 60000, 1800),
    ("Kashan", "卡尚", 51.45, 33.98, -100, 200, 1500, "波斯",
     "Known for textiles and ceramics on the Silk Road", 55000, 1600),
    ("Zaranj", "扎兰吉", 62.00, 31.00, -300, -100, 1100, "波斯",
     "Sistan capital, controlling trade routes to India", 40000, 1400),
    ("Sari", "萨里", 53.06, 36.56, -100, 200, 1300, "波斯",
     "Caspian trade city on the northern Persian route", 45000, 1500),

    ("Baghdad", "巴格达", 44.37, 33.31, 762, 800, 1258, "美索不达米亚",
     "Abbasid capital, center of the Islamic Golden Age", 1000000, 9000),
    ("Ctesiphon", "泰西封", 44.58, 33.09, -200, -100, 637, "美索不达米亚",
     "Parthian and Sassanid capital on the Tigris", 300000, 5500),
    ("Babylon", "巴比伦", 44.42, 32.54, -2000, -600, -100, "美索不达米亚",
     "Ancient city of legendary wealth, early Silk Road terminus", 200000, 4000),
    ("Mosul", "摩苏尔", 43.13, 36.35, -300, 100, 1400, "美索不达米亚",
     "Major trading city on the Tigris, crossroads of empires", 150000, 4200),
    ("Basra", "巴士拉", 47.78, 30.51, 636, 700, 1300, "美索不达米亚",
     "Port city connecting Persian Gulf to overland routes", 200000, 5000),
    ("Kirkuk", "基尔库克", 44.39, 35.47, -200, 100, 1200, "美索不达米亚",
     "Ancient city controlling northern Mesopotamian routes", 60000, 2000),
    ("Erbil", "埃尔比勒", 44.01, 36.19, -500, -200, 1400, "美索不达米亚",
     "One of the oldest continuously inhabited cities", 80000, 2500),
    ("Dura-Europos", "杜拉-欧罗普斯", 40.73, 35.00, -300, -200, 256, "美索不达米亚",
     "Frontier city on the Euphrates, melting pot of cultures", 30000, 1200),
    ("Hatra", "哈特拉", 42.72, 35.58, -200, -100, 241, "美索不达米亚",
     "Fortified caravan city between Roman and Parthian empires", 50000, 1800),
    ("Samarra", "萨马拉", 43.88, 34.20, 836, 850, 900, "美索不达米亚",
     "Abbasid capital briefly, massive palace complex", 100000, 3000),
    ("Kufa", "库法", 44.40, 32.10, 638, 700, 900, "美索不达米亚",
     "Early Islamic garrison city and intellectual center", 80000, 2800),
    ("Seleucia", "塞琉西亚", 44.55, 33.10, -300, -200, 200, "美索不达米亚",
     "Hellenistic city across the Tigris from Ctesiphon", 100000, 3500),

    ("Antioch", "安条克", 36.17, 36.20, -300, -200, 1100, "地中海东岸",
     "Great metropolis, terminus of overland Silk Road to Mediterranean", 300000, 7500),
    ("Palmyra", "巴尔米拉", 38.27, 34.55, -200, -100, 273, "地中海东岸",
     "Queen of the Desert, caravan city of legendary wealth", 200000, 8500),
    ("Damascus", "大马士革", 36.29, 33.51, -1000, -500, 1500, "地中海东岸",
     "One of the oldest cities, key trade nexus", 250000, 6000),
    ("Aleppo", "阿勒颇", 37.16, 36.20, -500, -200, 1500, "地中海东岸",
     "Great trading city at crossroads of empires", 200000, 5500),
    ("Tyre", "提尔", 35.20, 33.27, -1000, -500, 800, "地中海东岸",
     "Phoenician port, purple dye and maritime trade", 80000, 4000),
    ("Sidon", "西顿", 35.37, 33.56, -1000, -500, 700, "地中海东岸",
     "Ancient Phoenician port city, glass production", 50000, 2500),
    ("Petra", "佩特拉", 35.44, 30.33, -300, -100, 400, "地中海东岸",
     "Nabataean rock city controlling incense trade routes", 30000, 3500),
    ("Jerusalem", "耶路撒冷", 35.22, 31.78, -1000, -500, 1500, "地中海东岸",
     "Holy city, pilgrimage and trade center", 100000, 3000),
    ("Caesarea", "凯撒利亚", 34.89, 32.50, -22, 0, 700, "地中海东岸",
     "Roman-built port, gateway to the Mediterranean", 50000, 2500),
    ("Acre", "阿卡", 35.07, 32.92, -200, -50, 1300, "地中海东岸",
     "Crusader port city on the Mediterranean coast", 60000, 2800),
    ("Gaza", "加沙", 34.35, 31.38, -500, -200, 1200, "地中海东岸",
     "Caravan city connecting Egypt to the Levant", 70000, 3200),
    ("Homs", "霍姆斯", 36.72, 34.73, -200, 100, 1300, "地中海东岸",
     "Orontes valley city on the Silk Road branch to coast", 60000, 2000),

    ("Constantinople", "君士坦丁堡", 28.98, 41.01, 330, 400, 1453, "安纳托利亚",
     "Byzantine capital, gateway between Europe and Asia", 500000, 9500),
    ("Ephesus", "以弗所", 27.34, 38.00, -1000, -500, 600, "安纳托利亚",
     "Great Ionian city, temple of Artemis, major port", 200000, 4500),
    ("Sardis", "萨迪斯", 28.04, 38.49, -600, -300, 600, "安纳托利亚",
     "Lydian capital, western terminus of Royal Road", 80000, 3000),
    ("Tarsus", "塔尔苏斯", 35.00, 37.00, -500, -200, 1100, "安纳托利亚",
     "Birthplace of Paul, Cilician gateway city", 100000, 3200),
    ("Ankara", "安卡拉", 32.86, 39.93, -200, 50, 1200, "安纳托利亚",
     "Crossroads city in central Anatolia", 80000, 2500),
    ("Trabzon", "特拉布宗", 39.72, 41.00, -700, -400, 1400, "安纳托利亚",
     "Black Sea port, terminus of trade route from Persia", 70000, 3500),
    ("Konya", "科尼亚", 32.48, 37.87, -200, 100, 1400, "安纳托利亚",
     "Iconium, Seljuk capital on the Silk Road", 100000, 3000),
    ("Kayseri", "开塞利", 35.47, 38.73, -100, 100, 1300, "安纳托利亚",
     "Cappadocian city, caravanserai hub", 70000, 2200),
    ("Antalya", "安塔利亚", 30.71, 36.89, -200, -50, 1300, "安纳托利亚",
     "Pamphylian port city on Mediterranean coast", 60000, 2000),
    ("Sivas", "锡瓦斯", 37.02, 39.75, -100, 200, 1400, "安纳托利亚",
     "Anatolian crossroads, important under Seljuks", 50000, 1800),

    ("Rome", "罗马", 12.50, 41.90, -753, -200, 476, "地中海西端",
     "Eternal City, ultimate western destination of Silk Road goods", 1000000, 8000),
    ("Alexandria", "亚历山大", 29.92, 31.20, -331, -200, 1000, "地中海西端",
     "Great Hellenistic port, center of learning and trade", 500000, 9000),
    ("Athens", "雅典", 23.73, 37.97, -1000, -500, 500, "地中海西端",
     "Birthplace of democracy, philosophical center", 200000, 3000),
    ("Carthage", "迦太基", 10.33, 36.86, -800, -500, 146, "地中海西端",
     "Great maritime power, western Mediterranean trade hub", 300000, 5000),
    ("Syracuse", "叙拉古", 15.29, 37.07, -700, -400, 300, "地中海西端",
     "Greek city-state in Sicily, scientific center", 150000, 2500),
    ("Ravenna", "拉文纳", 12.20, 44.42, -50, 400, 750, "地中海西端",
     "Western Roman and Byzantine capital in Italy", 50000, 2000),
    ("Naples", "那不勒斯", 14.25, 40.84, -600, -300, 1000, "地中海西端",
     "Major Italian port handling eastern luxury goods", 100000, 3000),
    ("Thessaloniki", "塞萨洛尼基", 22.94, 40.64, -315, -100, 1400, "地中海西端",
     "Byzantine second city, Balkan trade hub", 120000, 3500),
    ("Venice", "威尼斯", 12.34, 45.44, 421, 800, 1450, "地中海西端",
     "Maritime republic dominating eastern Mediterranean trade", 150000, 8500),
    ("Genoa", "热那亚", 8.94, 44.41, -200, 100, 1450, "地中海西端",
     "Maritime republic rivaling Venice for eastern trade", 100000, 5500),
    ("Milan", "米兰", 9.19, 45.46, -400, -100, 1400, "地中海西端",
     "Imperial city, overland trade hub in northern Italy", 120000, 4000),
]

REGIONS = ["关中", "河西走廊", "塔里木盆地", "河中地区", "美索不达米亚"]

CLIMATE_PERIODS = []
year = -200
while year < 1500:
    end = min(year + 50, 1500)
    CLIMATE_PERIODS.append((year, end))
    year = end


def get_base_anomaly(period_start, period_end):
    mid = (period_start + period_end) / 2.0
    anomaly = 0.0
    if -250 <= mid <= 400:
        anomaly += 0.8
    elif 400 < mid <= 530:
        anomaly += 0.2
    elif 530 < mid <= 660:
        anomaly -= 1.2
    elif 660 < mid <= 900:
        anomaly += 0.0
    elif 900 < mid <= 1300:
        anomaly += 0.7
    elif 1300 < mid <= 1500:
        anomaly -= 0.6
    return anomaly


def get_region_adjustments(region):
    adj = {
        "关中": (0.3, -0.5, 0.8),
        "河西走廊": (0.0, -1.0, 0.5),
        "塔里木盆地": (-0.2, -1.5, 0.3),
        "河中地区": (0.1, -0.8, 0.5),
        "美索不达米亚": (0.5, -1.2, 0.2),
    }
    return adj.get(region, (0.0, -1.0, 0.5))


def generate_climate_data():
    rows = []
    for region in REGIONS:
        temp_adj, precip_adj, precip_base = get_region_adjustments(region)
        for ps, pe in CLIMATE_PERIODS:
            base = get_base_anomaly(ps, pe)
            anomaly = round(base + temp_adj + random.uniform(-0.3, 0.3), 2)
            precip = round(max(0.5, precip_base + precip_adj * (1 if base > 0 else -0.5)
                               + random.uniform(-0.5, 0.5)), 2)
            glacier = anomaly < -0.5
            note = ""
            if glacier:
                if 530 < (ps + pe) / 2 < 660:
                    note = "Late Antique Little Ice Age advance"
                elif 1300 < (ps + pe) / 2 <= 1500:
                    note = "Early Little Ice Age advance"
                else:
                    note = "Cold period glacial advance"

            notes_parts = []
            mid = (ps + pe) / 2
            if -250 <= mid <= 400:
                notes_parts.append("Roman Warm Period")
            if 530 < mid <= 660:
                notes_parts.append("Late Antique Little Ice Age")
            if 900 < mid <= 1300:
                notes_parts.append("Medieval Warm Period")
            if 1300 < mid <= 1500:
                notes_parts.append("Little Ice Age precursor")
            if region == "塔里木盆地" and precip < 1.5:
                notes_parts.append("Severe aridity")
            if region == "关中" and precip > 3.0:
                notes_parts.append("Enhanced monsoon")
            notes = "; ".join(notes_parts) if notes_parts else None

            rows.append((ps, pe, region, anomaly, precip, glacier, note if note else None, notes))
    return rows


def assign_decline(prosperity_end, prosperity_start):
    r = random.random()
    if r < 0.10:
        return None, None
    max_decline = min(prosperity_end + 300, 1550)
    min_decline = prosperity_end
    if min_decline >= max_decline:
        decline_year = prosperity_end + random.randint(10, 50)
    else:
        decline_year = random.randint(min_decline, max_decline)
    if r < 0.10 + 0.40:
        reason = "war"
    elif r < 0.10 + 0.40 + 0.30:
        reason = "trade_route"
    else:
        reason = "climate"
    return decline_year, reason


def generate_trade_connections(city_records):
    connections = []
    region_cities = {}
    for i, c in enumerate(city_records):
        region = c[7]
        region_cities.setdefault(region, []).append(i)

    region_order = ["关中", "河西走廊", "塔里木盆地", "河中地区", "波斯",
                    "美索不达米亚", "地中海东岸", "安纳托利亚", "地中海西端"]
    periods = [
        (-200, 200, "Han-Roman era"),
        (200, 500, "Late Antiquity"),
        (500, 800, "Early Islamic era"),
        (800, 1200, "Islamic Golden Age"),
        (1200, 1500, "Mongol-Late Medieval era"),
    ]

    for ps, pe, era_name in periods:
        for ri in range(len(region_order) - 1):
            r1 = region_order[ri]
            r2 = region_order[ri + 1]
            c1_list = region_cities.get(r1, [])
            c2_list = region_cities.get(r2, [])
            if not c1_list or not c2_list:
                continue
            n_pairs = random.randint(3, min(8, len(c1_list), len(c2_list)))
            chosen1 = random.sample(c1_list, min(n_pairs, len(c1_list)))
            chosen2 = random.sample(c2_list, min(n_pairs, len(c2_list)))
            for ci1 in chosen1:
                for ci2 in chosen2:
                    vol = random.randint(500, 5000)
                    rt = "land"
                    if r2 in ("地中海西端", "地中海东岸") and random.random() < 0.3:
                        rt = "sea"
                    connections.append((ci1, ci2, ps, pe, vol, rt))

        for region, city_ids in region_cities.items():
            if len(city_ids) < 2:
                continue
            n_intra = random.randint(2, min(5, len(city_ids) // 2 + 1))
            for _ in range(n_intra):
                a, b = random.sample(city_ids, 2)
                vol = random.randint(300, 3000)
                connections.append((a, b, ps, pe, vol, "land"))

    deduped = {}
    for c1, c2, ps, pe, vol, rt in connections:
        key = (min(c1, c2), max(c1, c2), ps, pe)
        if key not in deduped or deduped[key][4] < vol:
            deduped[key] = (c1, c2, ps, pe, vol, rt)

    return list(deduped.values())


def main():
    print("[1/6] Connecting to PostgreSQL...")
    conn = psycopg2.connect(DATABASE_URL)
    conn.autocommit = True
    cur = conn.cursor()

    print("[2/6] Creating tables...")
    cur.execute("CREATE EXTENSION IF NOT EXISTS postgis;")

    cur.execute("DROP TABLE IF EXISTS trade_connections;")
    cur.execute("DROP TABLE IF EXISTS climate_data;")
    cur.execute("DROP TABLE IF EXISTS cities;")

    cur.execute("""
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
    """)

    cur.execute("""
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
    """)

    cur.execute("""
        CREATE TABLE trade_connections (
            id SERIAL PRIMARY KEY,
            city_from INTEGER REFERENCES cities(id),
            city_to INTEGER REFERENCES cities(id),
            period_start INTEGER NOT NULL,
            period_end INTEGER NOT NULL,
            trade_volume INTEGER,
            route_type VARCHAR(50)
        );
    """)

    cur.execute("CREATE INDEX idx_cities_geom ON cities USING GIST(geom);")
    cur.execute("CREATE INDEX idx_cities_founded ON cities(founded_year);")
    cur.execute("CREATE INDEX idx_cities_decline ON cities(decline_year);")
    cur.execute("CREATE INDEX idx_climate_period ON climate_data(period_start, period_end);")
    cur.execute("CREATE INDEX idx_climate_region ON climate_data(region);")
    cur.execute("CREATE INDEX idx_trade_cities ON trade_connections(city_from, city_to);")
    cur.execute("CREATE INDEX idx_trade_period ON trade_connections(period_start, period_end);")

    print("[3/6] Inserting {} cities...".format(len(CITIES_RAW)))
    city_ids = []
    for i, c in enumerate(CITIES_RAW):
        name, name_cn, lon, lat, founded, p_start, p_end, region, desc, pop, trade = c
        decline_year, decline_reason = assign_decline(p_end, p_start)
        pop_var = pop + random.randint(-int(pop * 0.15), int(pop * 0.15))
        trade_var = trade + random.randint(-int(trade * 0.1), int(trade * 0.1))
        cur.execute("""
            INSERT INTO cities (name, name_cn, longitude, latitude, geom,
                founded_year, prosperity_start, prosperity_end,
                decline_year, decline_reason, region, description,
                population_peak, trade_volume)
            VALUES (%s, %s, %s, %s,
                ST_SetSRID(ST_MakePoint(%s, %s), 4326),
                %s, %s, %s, %s, %s, %s, %s, %s, %s)
            RETURNING id;
        """, (name, name_cn, lon, lat, lon, lat,
              founded, p_start, p_end, decline_year, decline_reason,
              region, desc, pop_var, trade_var))
        city_ids.append(cur.fetchone()[0])
        if (i + 1) % 20 == 0:
            print("  Inserted {}/{} cities".format(i + 1, len(CITIES_RAW)))
    print("  Inserted {}/{} cities".format(len(CITIES_RAW), len(CITIES_RAW)))

    print("[4/6] Generating and inserting climate data...")
    climate_rows = generate_climate_data()
    for i, row in enumerate(climate_rows):
        cur.execute("""
            INSERT INTO climate_data (period_start, period_end, region,
                temperature_anomaly, precipitation_index,
                glacier_advance, glacier_note, notes)
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s);
        """, row)
        if (i + 1) % 40 == 0:
            print("  Inserted {}/{} climate records".format(i + 1, len(climate_rows)))
    print("  Inserted {}/{} climate records".format(len(climate_rows), len(climate_rows)))

    print("[5/6] Generating and inserting trade connections...")
    city_records = [(c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], c[8], c[9], c[10])
                    for c in CITIES_RAW]
    trade_rows = generate_trade_connections(city_records)
    for i, (ci1, ci2, ps, pe, vol, rt) in enumerate(trade_rows):
        fid = city_ids[ci1]
        tid = city_ids[ci2]
        vol_var = vol + random.randint(-int(vol * 0.1), int(vol * 0.1))
        cur.execute("""
            INSERT INTO trade_connections (city_from, city_to,
                period_start, period_end, trade_volume, route_type)
            VALUES (%s, %s, %s, %s, %s, %s);
        """, (fid, tid, ps, pe, vol_var, rt))
        if (i + 1) % 100 == 0:
            print("  Inserted {}/{} trade connections".format(i + 1, len(trade_rows)))
    print("  Inserted {}/{} trade connections".format(len(trade_rows), len(trade_rows)))

    print("[6/6] Verifying data...")
    cur.execute("SELECT COUNT(*) FROM cities;")
    print("  cities: {} rows".format(cur.fetchone()[0]))
    cur.execute("SELECT COUNT(*) FROM climate_data;")
    print("  climate_data: {} rows".format(cur.fetchone()[0]))
    cur.execute("SELECT COUNT(*) FROM trade_connections;")
    print("  trade_connections: {} rows".format(cur.fetchone()[0]))

    cur.execute("""
        SELECT decline_reason, COUNT(*) FROM cities
        GROUP BY decline_reason ORDER BY COUNT(*) DESC;
    """)
    print("  Decline reason distribution:")
    for reason, cnt in cur.fetchall():
        pct = round(cnt / len(CITIES_RAW) * 100, 1)
        print("    {}: {} ({}%)".format(reason, cnt, pct))

    cur.close()
    conn.close()
    print("Done! Data simulation complete.")


if __name__ == "__main__":
    random.seed(42)
    main()
