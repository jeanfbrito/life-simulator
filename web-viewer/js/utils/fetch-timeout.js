/**
 * Fetch with automatic timeout using AbortController
 * Prevents UI hangs due to unresponsive server requests
 */

/**
 * Fetch with automatic timeout using AbortController
 * @param {string} url - URL to fetch
 * @param {object} options - Fetch options
 * @param {number} timeout - Timeout in milliseconds (default: 5000)
 * @returns {Promise<Response>}
 * @throws {Error} Throws timeout error if request exceeds timeout duration
 */
export async function fetchWithTimeout(url, options = {}, timeout = 5000) {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    try {
        const response = await fetch(url, {
            ...options,
            signal: controller.signal
        });
        clearTimeout(timeoutId);
        return response;
    } catch (error) {
        clearTimeout(timeoutId);
        if (error.name === 'AbortError') {
            throw new Error(`Request timeout after ${timeout}ms`);
        }
        throw error;
    }
}
