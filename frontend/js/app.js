const API_BASE = '/api';

document.addEventListener('DOMContentLoaded', function() {
    SilkroadMap.init(API_BASE);
    CityDetail.init(API_BASE);

    SilkroadMap.onCityClick = function(cityId) {
        CityDetail.showCityPanel(cityId);
    };

    SilkroadMap.onYearChange = function(year) {
        const panel = document.getElementById('city-panel');
        if (!panel.classList.contains('hidden')) {
            const cityId = CityDetail.currentCityId;
            if (cityId) {
                const city = SilkroadMap.getCityById(cityId);
                if (city) {
                    CityDetail.drawCityTimeline(city);
                }
            }
        }
    };

    Promise.all([
        SilkroadMap.loadCities(),
        SilkroadMap.loadArrows()
    ]).then(function() {
        console.log('Silk Road Analysis System loaded');
    }).catch(function(err) {
        console.warn('Data load failed, using fallback data', err);
    });
});
