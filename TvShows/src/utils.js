import { ToastAndroid } from 'react-native';

let abortController = null;

export async function get(path, params = {}) {
    if (path.startsWith('/')) {
        path = path.substring(1);
    }
    
    if (abortController != null) {
        console.info('Aborting previous fetch call');
        abortController.abort();
    }

    abortController = new AbortController();
    const url = `http://${hostname()}/${path}`;
    const queryStr = Object.entries(params)
        .map(([key, value], _i) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`).join('&');
    console.info('Fetching:', `${url}?${queryStr}`);

    const response = await fetch(`${url}?${queryStr}`, { method: 'GET', signal: abortController.signal });
    abortController = null;
    if (response.status != 200) {
        throw new Error(`${url}: ${response.status}, "${await response.text()}"`);
    }
    return await response.json();
}

export const hostname = () => __DEV__ ? 'localhost:3000' : '192.168.1.2:3000';

// export const hostname = () => '192.168.1.2:3000';

export function abortGet() {
    if (abortController != null) {
        console.info('Aborting previous fetch call');
        abortController.abort();
    }
    abortController = null;
}
