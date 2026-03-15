// Theme toggle functionality
function initThemeToggle() {
  const lightTheme = "corporate";
  const darkTheme = "business";

  // Handle main theme controller
  const themeController = document.querySelector(".theme-controller");
  if (themeController) {
    // Set initial state based on current theme
    const currentTheme = localStorage.getItem("theme") || lightTheme;
    const isDark = currentTheme === darkTheme;
    themeController.checked = isDark;

    // Add event listener for theme changes
    themeController.addEventListener("change", function () {
      const newTheme = this.checked ? darkTheme : lightTheme;
      document.documentElement.setAttribute("data-theme", newTheme);
      localStorage.setItem("theme", newTheme);

      // Sync with menu controller
      const menuController = document.querySelector(".theme-controller-menu");
      if (menuController) {
        menuController.checked = this.checked;
      }
    });
  }

  // Handle menu theme controller
  const menuThemeController = document.querySelector(".theme-controller-menu");
  if (menuThemeController) {
    // Set initial state
    const currentTheme = localStorage.getItem("theme") || lightTheme;
    menuThemeController.checked = currentTheme === darkTheme;

    // Add event listener
    menuThemeController.addEventListener("change", function () {
      const newTheme = this.checked ? darkTheme : lightTheme;
      document.documentElement.setAttribute("data-theme", newTheme);
      localStorage.setItem("theme", newTheme);

      // Sync with main controller
      const mainController = document.querySelector(".theme-controller");
      if (mainController) {
        mainController.checked = this.checked;
      }
    });
  }
}

// Initialize when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initThemeToggle);
} else {
  initThemeToggle();
}
