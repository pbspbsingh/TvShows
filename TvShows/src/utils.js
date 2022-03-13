import AsyncStorage from '@react-native-async-storage/async-storage';
import { ToastAndroid } from 'react-native';

const HOSTS = [];

const HOST_KEY = '@__host_key__';

let currentHost = null;

let abortController = null;

if (__DEV__) {
    HOSTS.push('localhost:3000');
} else {
    const prod = ['192.168.1.2:3000', '10.3.141.1:3000'];
    HOSTS.push(...prod);
}

export async function get(path, params = {}) {
    if (currentHost == null) {
        currentHost = parseInt(await AsyncStorage.getItem(HOST_KEY) || '0');
    }
    if (path.startsWith('/')) {
        path = path.substring(1);
    }
    const queryStr = Object.entries(params)
        .map(([key, value], _i) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`).join('&');
    let networkRetries = 0;
    let error = null;
    while (networkRetries < HOSTS.length) {
        try {
            if (abortController != null) {
                console.info('Aborting previous fetch call');
                abortController.abort();
            }
            abortController = new AbortController();
            const url = `http://${HOSTS[currentHost]}/${path}`;
            console.info('Fetching:', `${url}?${queryStr}`);
            const response = await fetch(`${url}?${queryStr}`, { method: 'GET', signal: abortController.signal });
            abortController = null;
            if (response.status != 200) {
                throw new Error(`${url}: ${response.status}, "${await response.text()}"`);
            }
            return await response.json();
        } catch (e) {
            error = e;
            if (e instanceof Error && e.message == 'Network request failed') {
                ToastAndroid.show(`Network error: ${HOSTS[currentHost]}, retries: ${networkRetries + 1}`, ToastAndroid.LONG);
                networkRetries++;
                currentHost = (++currentHost) % HOSTS.length;
                await AsyncStorage.setItem(HOST_KEY, currentHost.toString());
            } else {
                console.error('Error while making nework call', e);
                break;
            }
        }
    }
    throw error;
}

export function hostname() {
    if (currentHost == null) {
        return HOSTS[0];
    }
    return HOSTS[currentHost];
}

export function abortGet() {
    if (abortController != null) {
        console.info('Aborting previous fetch call');
        abortController.abort();
    }
    abortController = null;
}
