import { render } from "preact";
import GalleryApp from "./GalleryApp";

// Get configuration from global variable (set in template)
const config = window.GALLERY_CONFIG || {};
const accessCode = config.accessCode;
const apiEndpoint = config.apiEndpoint || "/api/3d/gallery";

// Hide loading screen once app is ready
function hideLoadingScreen() {
  const loadingScreen = document.getElementById("loading-screen");
  if (loadingScreen) {
    loadingScreen.classList.add("hidden");
  }
}

// Show error in UI
function showError(title, message) {
  const root = document.getElementById("gallery-root");
  if (root) {
    render(
      <div class="error-container">
        <div class="error-icon">⚠️</div>
        <h1 class="error-title">{title}</h1>
        <p class="error-message">{message}</p>
      </div>,
      root,
    );
  }
  hideLoadingScreen();
}

// Validate configuration
if (!accessCode) {
  showError(
    "Access Code Required",
    "No access code provided. Please use a valid gallery link with an access code.",
  );
} else {
  // Render the main app
  const root = document.getElementById("gallery-root");
  if (root) {
    render(
      <GalleryApp
        accessCode={accessCode}
        apiEndpoint={apiEndpoint}
        onReady={hideLoadingScreen}
        onError={showError}
      />,
      root,
    );
  } else {
    console.error("Gallery root element not found");
  }
}
