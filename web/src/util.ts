export async function get<T>(
    path: string,
    params: { [key: string]: string } = {}
): Promise<T> {
    if (path.startsWith('/')) {
        path = path.substring(1);
    }
    
    const queryParam = Object.entries(params)
        .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
        .join('\n');
    const response = await fetch(`/${path}?${queryParam}`);
    if (response.status === 200) {
        return await response.json();
    } else {
        throw new Error(`Error: ${await response.text()}`)
    }
}