/*
import {registerRoute} from 'workbox-routing';
import {CacheFirst} from 'workbox-strategies';
import {CacheableResponsePlugin} from 'workbox-cacheable-response';

registerRoute(
    ({url}) => url.pathname.endsWith('pbf'),
    new CacheFirst({
        cacheName: 'pbf-cache',
        plugins: [
            new CacheableResponsePlugin({
                statuses: [0, 200],
            })
        ]
    })
);
*/
