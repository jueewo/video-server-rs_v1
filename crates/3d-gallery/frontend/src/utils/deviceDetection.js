/**
 * deviceDetection.js
 *
 * Utility for detecting device capabilities and determining optimal quality settings
 * for 3D gallery performance, especially for mobile VR devices like HTC Vive Flow.
 */

/**
 * Device type enumeration
 */
export const DeviceType = {
  DESKTOP: 'desktop',
  MOBILE: 'mobile',
  MOBILE_VR: 'mobile_vr',
  DESKTOP_VR: 'desktop_vr',
  UNKNOWN: 'unknown',
};

/**
 * Quality profile enumeration
 */
export const QualityProfile = {
  HIGH: 'high',
  MEDIUM: 'medium',
  LOW: 'low',
  ULTRA_LOW: 'ultra_low',
};

/**
 * Detect if device is mobile based on user agent and screen size
 */
function isMobileDevice() {
  const userAgent = navigator.userAgent || navigator.vendor || window.opera;

  // Check user agent for mobile indicators
  const mobileRegex = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i;
  const isMobileUA = mobileRegex.test(userAgent);

  // Check screen size (mobile typically < 768px width)
  const isSmallScreen = window.innerWidth < 768;

  // Check for touch support
  const hasTouch = 'ontouchstart' in window || navigator.maxTouchPoints > 0;

  return isMobileUA || (isSmallScreen && hasTouch);
}

/**
 * Detect if device is a VR headset
 */
function isVRDevice() {
  const userAgent = navigator.userAgent || '';

  // Check for VR-specific user agent strings
  const vrRegex = /VR|Oculus|Quest|Vive|Pico|MetaQuest|OculusBrowser/i;
  const isVRUA = vrRegex.test(userAgent);

  // Check for WebXR support (indicates VR capability)
  const hasWebXR = 'xr' in navigator;

  // Check for specific VR display APIs
  const hasVRDisplay = 'getVRDisplays' in navigator;

  return isVRUA || hasWebXR || hasVRDisplay;
}

/**
 * Detect if device is a standalone mobile VR headset (HTC Vive Flow, Quest, etc.)
 */
function isMobileVR() {
  const userAgent = navigator.userAgent || '';

  // Standalone mobile VR headsets
  const mobileVRRegex = /Quest|Vive Flow|Pico|Go|MetaQuest/i;

  return mobileVRRegex.test(userAgent) && isVRDevice();
}

/**
 * Detect GPU tier (approximate)
 */
function getGPUTier() {
  // Try to get GPU info from WebGL
  try {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');

    if (!gl) {
      return 'unknown';
    }

    const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
    if (debugInfo) {
      const renderer = gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL);

      // High-end GPUs
      if (/RTX|GTX 1[6-9]|GTX [2-9]0|RX [5-7]|Vega|RDNA/i.test(renderer)) {
        return 'high';
      }

      // Mid-range GPUs
      if (/GTX|RX|Mali-G|Adreno [5-9]|Apple GPU/i.test(renderer)) {
        return 'medium';
      }

      // Low-end/integrated GPUs
      if (/Intel|Adreno [1-4]|Mali-[4-5]/i.test(renderer)) {
        return 'low';
      }
    }

    return 'medium'; // Default fallback
  } catch (e) {
    console.warn('Could not detect GPU tier:', e);
    return 'medium';
  }
}

/**
 * Get device memory (in GB) if available
 */
function getDeviceMemory() {
  // Navigator.deviceMemory is available in some browsers (Chrome, Edge)
  if ('deviceMemory' in navigator) {
    return navigator.deviceMemory; // Returns GB as number
  }
  return null;
}

/**
 * Detect hardware concurrency (CPU cores)
 */
function getHardwareConcurrency() {
  return navigator.hardwareConcurrency || 4; // Default to 4 if unknown
}

/**
 * Detect the device type
 *
 * @returns {string} Device type from DeviceType enum
 */
export function detectDeviceType() {
  const isVR = isVRDevice();
  const isMobile = isMobileDevice();

  if (isMobileVR()) {
    return DeviceType.MOBILE_VR;
  } else if (isVR) {
    return DeviceType.DESKTOP_VR;
  } else if (isMobile) {
    return DeviceType.MOBILE;
  } else {
    return DeviceType.DESKTOP;
  }
}

/**
 * Get comprehensive device capabilities
 *
 * @returns {Object} Device capabilities and detected features
 */
export function getDeviceCapabilities() {
  const deviceType = detectDeviceType();
  const gpuTier = getGPUTier();
  const deviceMemory = getDeviceMemory();
  const cpuCores = getHardwareConcurrency();

  const capabilities = {
    deviceType,
    isMobile: deviceType === DeviceType.MOBILE || deviceType === DeviceType.MOBILE_VR,
    isVR: deviceType === DeviceType.MOBILE_VR || deviceType === DeviceType.DESKTOP_VR,
    isMobileVR: deviceType === DeviceType.MOBILE_VR,
    gpuTier,
    deviceMemory,
    cpuCores,
    userAgent: navigator.userAgent,
    screenWidth: window.innerWidth,
    screenHeight: window.innerHeight,
    pixelRatio: window.devicePixelRatio || 1,
    hasWebXR: 'xr' in navigator,
    hasTouch: 'ontouchstart' in window || navigator.maxTouchPoints > 0,
  };

  console.log('🔍 Device Capabilities Detected:', capabilities);

  return capabilities;
}

/**
 * Determine optimal quality profile based on device capabilities
 *
 * @param {Object} capabilities - Device capabilities from getDeviceCapabilities()
 * @returns {string} Quality profile from QualityProfile enum
 */
export function determineQualityProfile(capabilities = null) {
  if (!capabilities) {
    capabilities = getDeviceCapabilities();
  }

  const { deviceType, gpuTier, deviceMemory, cpuCores } = capabilities;

  // Mobile VR headsets (HTC Vive Flow, Quest, etc.) - need aggressive optimization
  if (deviceType === DeviceType.MOBILE_VR) {
    // Quest 2/3 or newer devices with good specs
    if (deviceMemory >= 6 || cpuCores >= 8) {
      return QualityProfile.MEDIUM;
    }
    // Older mobile VR (Vive Flow, Quest 1, Go)
    return QualityProfile.ULTRA_LOW;
  }

  // Desktop VR - can handle higher quality
  if (deviceType === DeviceType.DESKTOP_VR) {
    if (gpuTier === 'high') {
      return QualityProfile.HIGH;
    }
    return QualityProfile.MEDIUM;
  }

  // Mobile phones/tablets
  if (deviceType === DeviceType.MOBILE) {
    if (deviceMemory >= 4 && gpuTier !== 'low') {
      return QualityProfile.MEDIUM;
    }
    return QualityProfile.LOW;
  }

  // Desktop
  if (deviceType === DeviceType.DESKTOP) {
    if (gpuTier === 'high' && (deviceMemory === null || deviceMemory >= 8)) {
      return QualityProfile.HIGH;
    }
    if (gpuTier === 'medium' || (deviceMemory !== null && deviceMemory >= 4)) {
      return QualityProfile.MEDIUM;
    }
    return QualityProfile.LOW;
  }

  // Unknown - use safe defaults
  return QualityProfile.MEDIUM;
}

/**
 * Get quality settings based on profile
 *
 * @param {string} profile - Quality profile from QualityProfile enum
 * @returns {Object} Quality settings for the gallery
 */
export function getQualitySettings(profile) {
  const settings = {
    // Texture settings
    maxTextureSize: 2048,
    useHighResTextures: true,
    useThumbnailsOnly: false,
    textureLoadDistance: 20,

    // Lighting settings
    maxLights: 6,
    useSpotlights: true,
    useShadows: true,
    shadowMapSize: 1024,

    // Rendering settings
    antialiasing: true,
    maxFPS: 60,
    usePostProcessing: false,

    // Performance settings
    frustumCulling: true,
    occlusionCulling: false,
    lazyLoadAssets: true,
    maxVisibleItems: 50,

    // Mobile-specific
    disablePDF: false,
    disableVideo: false,
    simplifiedGeometry: false,
  };

  switch (profile) {
    case QualityProfile.HIGH:
      settings.maxTextureSize = 4096;
      settings.maxLights = 8;
      settings.shadowMapSize = 2048;
      settings.antialiasing = true;
      settings.usePostProcessing = true;
      settings.maxVisibleItems = 100;
      break;

    case QualityProfile.MEDIUM:
      settings.maxTextureSize = 2048;
      settings.maxLights = 4;
      settings.shadowMapSize = 1024;
      settings.antialiasing = true;
      settings.maxVisibleItems = 50;
      break;

    case QualityProfile.LOW:
      settings.maxTextureSize = 1024;
      settings.useHighResTextures = false;
      settings.maxLights = 3;
      settings.useSpotlights = false;
      settings.useShadows = false;
      settings.shadowMapSize = 512;
      settings.antialiasing = false;
      settings.maxVisibleItems = 30;
      settings.disablePDF = true; // PDF.js too heavy for low-end
      break;

    case QualityProfile.ULTRA_LOW:
      // Aggressive optimization for mobile VR (HTC Vive Flow)
      settings.maxTextureSize = 512;
      settings.useHighResTextures = false;
      settings.useThumbnailsOnly = true;
      settings.textureLoadDistance = 10;
      settings.maxLights = 1; // Single hemispheric light only
      settings.useSpotlights = false;
      settings.useShadows = false;
      settings.shadowMapSize = 256;
      settings.antialiasing = false;
      settings.maxFPS = 30; // Target 30fps for stability
      settings.maxVisibleItems = 20;
      settings.disablePDF = true; // Disable PDF support entirely
      settings.disableVideo = false; // Keep video but with lower quality
      settings.simplifiedGeometry = true; // Use simple frames
      settings.occlusionCulling = true; // Enable for better performance
      break;
  }

  console.log(`⚙️ Quality Settings [${profile}]:`, settings);

  return settings;
}

/**
 * Get automatic quality settings based on current device
 *
 * @returns {Object} Complete quality configuration
 */
export function getAutoQualitySettings() {
  const capabilities = getDeviceCapabilities();
  const profile = determineQualityProfile(capabilities);
  const settings = getQualitySettings(profile);

  return {
    capabilities,
    profile,
    settings,
  };
}

/**
 * Check if device is likely to struggle with the gallery
 *
 * @returns {boolean} True if performance warnings should be shown
 */
export function shouldShowPerformanceWarning() {
  const capabilities = getDeviceCapabilities();
  const profile = determineQualityProfile(capabilities);

  // Show warning for ultra-low profile or low-end mobile VR
  return profile === QualityProfile.ULTRA_LOW ||
         (capabilities.isMobileVR && capabilities.deviceMemory < 4);
}
