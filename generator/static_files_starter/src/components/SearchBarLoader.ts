// SearchBarLoader.ts - Client-side search component loader

export async function initializeSearch(searchData: string) {
  const searchPosts = JSON.parse(searchData);
  const searchTrigger = document.getElementById("search-trigger");
  const searchContainer = document.getElementById("search-container");

  if (!searchTrigger || !searchContainer) {
    console.error("Search elements not found");
    return;
  }

  let searchLoaded = false;
  let searchInitialized = false;
  let prefetchTriggered = false;

  // Function to load and initialize search
  async function loadSearch() {
    if (searchLoaded) return;

    searchLoaded = true;

    // Show loading state
    const originalHTML = searchTrigger.innerHTML;
    searchTrigger.innerHTML =
      '<span class="inline-block w-10 h-10 border-4 border-primary/30 border-t-primary rounded-full animate-spin"></span>';
    (searchTrigger as HTMLButtonElement).disabled = true;

    try {
      // Dynamically import all required modules
      const [{ default: Search }, preactModule] = await Promise.all([
        import("./preact/Search"),
        import("preact"),
      ]);

      const { render, h } = preactModule;

      // Hide the trigger button and show the Search component button
      searchTrigger.style.display = "none";

      // Render the search component
      if (!searchInitialized) {
        searchInitialized = true;
        render(h(Search, { searchList: searchPosts }), searchContainer);

        // Trigger the search to open after rendering
        setTimeout(() => {
          const searchButton = searchContainer.querySelector("button");
          if (searchButton) {
            searchButton.click();
          }
        }, 50);
      }
    } catch (error) {
      console.error("Failed to load search component:", error);

      // Restore button on error
      searchTrigger.innerHTML = originalHTML;
      (searchTrigger as HTMLButtonElement).disabled = false;

      // Show error message
      searchContainer.innerHTML = `
                <div class="p-3 bg-red-100 text-red-800 rounded-lg text-sm" style="position: absolute; right: 0; top: 100%; margin-top: 1rem; z-index: 50; max-width: 300px;">
                    <span>Failed to load search. Please refresh the page.</span>
                </div>
            `;
      setTimeout(() => {
        searchContainer.innerHTML = "";
      }, 3000);
    }
  }

  // Prefetch function
  function prefetchSearch() {
    if (!prefetchTriggered && !searchLoaded) {
      prefetchTriggered = true;
      // Start loading modules in background
      Promise.all([import("./preact/Search"), import("preact")]).catch(() => {
        // Silent fail for prefetch
      });
    }
  }

  // Load search on button click
  searchTrigger.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    loadSearch();
  });

  // Optional: Prefetch on hover for better UX
  searchTrigger.addEventListener("mouseenter", prefetchSearch);
}
