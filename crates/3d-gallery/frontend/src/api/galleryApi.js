/**
 * Gallery API Client
 *
 * Handles communication with the backend API to fetch gallery data.
 */

/**
 * Fetch gallery data from the backend
 *
 * @param {string} accessCode - The access code for the gallery
 * @param {string} apiEndpoint - The API endpoint URL
 * @returns {Promise<Object>} Gallery data including items, scene, and permissions
 * @throws {Error} If the request fails or access is denied
 */
export async function fetchGalleryData(accessCode, apiEndpoint) {
  if (!accessCode) {
    throw new Error('Access code is required');
  }

  try {
    const url = `${apiEndpoint}?code=${encodeURIComponent(accessCode)}`;
    const response = await fetch(url);

    if (!response.ok) {
      if (response.status === 401 || response.status === 403) {
        throw new Error('Invalid or expired access code');
      } else if (response.status === 404) {
        throw new Error('Gallery not found');
      } else {
        throw new Error(`Failed to load gallery: ${response.statusText}`);
      }
    }

    const data = await response.json();

    // Validate response structure
    if (!data || typeof data !== 'object') {
      throw new Error('Invalid response format');
    }

    return data;
  } catch (error) {
    if (error instanceof TypeError) {
      // Network error
      throw new Error('Network error: Unable to connect to server');
    }
    throw error;
  }
}

/**
 * Preload media assets (images, videos)
 *
 * @param {Array} items - Array of media items to preload
 * @returns {Promise<void>}
 */
export async function preloadMediaAssets(items) {
  if (!items || !Array.isArray(items)) {
    return;
  }

  const promises = items.map(item => {
    return new Promise((resolve, reject) => {
      if (item.type === 'image') {
        const img = new Image();
        img.onload = () => resolve();
        img.onerror = () => reject(new Error(`Failed to load image: ${item.url}`));
        img.src = item.url;
      } else if (item.type === 'video') {
        // For videos, just check if URL is accessible
        fetch(item.url, { method: 'HEAD' })
          .then(() => resolve())
          .catch(() => reject(new Error(`Failed to load video: ${item.url}`)));
      } else {
        resolve(); // Unknown type, skip
      }
    });
  });

  try {
    await Promise.allSettled(promises);
  } catch (error) {
    console.warn('Some assets failed to preload:', error);
  }
}
