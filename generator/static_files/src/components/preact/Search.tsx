import { computed, effect, signal } from "@preact/signals";
import { useRef } from "preact/hooks";
import Fuse from "fuse.js";
import { IconSearch } from "../icons/IconSearch";

// Updated Fuse.js configuration
const options = {
  keys: [
    "frontmatter.title",
    "frontmatter.description",
    "frontmatter.desc",
    "frontmatter.tags",
    "frontmatter.url",
  ],
  includeMatches: true,
  minMatchCharLength: 2,
  threshold: 0.5,
};

const query = signal("");
const isOpen = signal(false);
const currentPage = signal(1);
const MAXPOSTSPERPAGE = 5;
const MINQUERYLENGTH = 4;

function handleOnSearch(e: any) {
  query.value = e.target.value;
  // Reset to first page when query changes
  currentPage.value = 1;
}
// function handleOnSearch(e: any) {
//   const newQuery = e.target.value;

//   // If query is being cleared or changed, reset results immediately
//   if (newQuery !== query.value) {
//     query.value = newQuery;
//     currentPage.value = 1; // Reset to first page
//   }
// }

type Post = {
  frontmatter: {
    title: string;
    description?: string;
    desc?: string;
    tags?: string[];
    url?: string;
  };
  collection: string;
  slug: string;
};

type Props = {
  searchList: Post[];
};

// Pagination Component
type PaginationProps = {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
};

function Pagination({
  currentPage,
  totalPages,
  onPageChange,
}: PaginationProps) {
  if (totalPages <= 1) return null;

  return (
    <div className="flex justify-center mt-2 space-x-2">
      <button
        onClick={() => onPageChange(currentPage - 1)}
        disabled={currentPage === 1}
        className="btn btn-xs btn-outline"
      >
        Prev
      </button>

      <div className="flex space-x-1">
        {/* Show page numbers with ellipsis for large page counts */}
        {Array.from({ length: Math.min(totalPages, 5) }, (_, i) => {
          let pageNum;

          if (totalPages <= 5) {
            // Show all pages if 5 or fewer
            pageNum = i + 1;
          } else if (currentPage <= 3) {
            // Near start
            pageNum = i + 1;
            if (i === 4) return <span key={i}>...</span>;
          } else if (currentPage >= totalPages - 2) {
            // Near end
            pageNum = totalPages - 4 + i;
            if (i === 0) {
              return <span key={i}>...</span>;
            }
          } else {
            // Middle
            pageNum = currentPage - 2 + i;
            if (i === 0) return <span key={i}>...</span>;
            if (i === 4) return <span key={i}>...</span>;
          }

          return (
            <button
              key={i}
              onClick={() => onPageChange(pageNum)}
              className={`btn btn-xs ${
                currentPage === pageNum ? "btn-active" : "btn-outline"
              }`}
            >
              {pageNum}
            </button>
          );
        })}
      </div>

      <button
        onClick={() => onPageChange(currentPage + 1)}
        disabled={currentPage === totalPages}
        className="btn btn-xs btn-outline"
      >
        Next
      </button>
    </div>
  );
}

const _base = import.meta.env.BASE_URL;
const base = _base === "/" ? "" : _base.replace(/\/$/, "");

function getLink(post: Post) {
  // post.slug includes language prefix (e.g., "en/impressum")
  // extract lang from slug prefix, then build: base/lang/collection/slug-without-lang
  const lang = post.slug.split("/")[0] || "en";
  const slugWithoutLang = post.slug.replace(/^[^/]+\//, ""); // Remove "en/" from "en/impressum"
  return `${base}/${lang}/${post.collection}/${slugWithoutLang}`;
}

function Search({ searchList }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const fuse = new Fuse(searchList, options);

  // Compute search results
  // const allResults = computed(() =>
  //   query.value ? fuse.search(query.value).map((result) => result.item) : [],
  // );
  const allResults = computed(() => {
    // If no query or query is being typed, return empty results immediately
    if (!query.value || query.value.trim().length < MINQUERYLENGTH) {
      return [];
    }

    // Only search when we have at least 2 characters
    return fuse.search(query.value).map((result) => result.item);
  });

  // Calculate total pages
  const totalPages = computed(() =>
    Math.ceil(allResults.value.length / MAXPOSTSPERPAGE),
  );

  // Get posts for current page
  const paginatedPosts = computed(() => {
    const startIndex = (currentPage.value - 1) * MAXPOSTSPERPAGE;
    return allResults.value.slice(startIndex, startIndex + MAXPOSTSPERPAGE);
  });

  // Auto-focus effect when dropdown opens
  effect(() => {
    if (isOpen.value && inputRef.current) {
      // Use requestAnimationFrame to ensure DOM is updated
      requestAnimationFrame(() => {
        inputRef.current?.focus();
      });
    }
  });

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages.value) {
      currentPage.value = page;
    }
  }

  // Handle dropdown toggle
  function handleDropdownToggle(e: Event) {
    e.preventDefault();
    e.stopPropagation();

    isOpen.value = !isOpen.value;

    // If opening, focus the input immediately
    if (isOpen.value) {
      // Multiple attempts to ensure focus works
      setTimeout(() => inputRef.current?.focus(), 0);
      setTimeout(() => inputRef.current?.focus(), 10);
      setTimeout(() => inputRef.current?.focus(), 50);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }

  // Handle dropdown close
  function handleDropdownClose() {
    isOpen.value = false;
    query.value = ""; // Clear search when closing
    currentPage.value = 1; // Reset to first page
  }

  // Handle clicks outside dropdown
  function handleOutsideClick(e: Event) {
    if (
      dropdownRef.current &&
      !dropdownRef.current.contains(e.target as Node)
    ) {
      handleDropdownClose();
    }
  }

  // Add/remove outside click listener
  effect(() => {
    if (isOpen.value) {
      document.addEventListener("click", handleOutsideClick);
      return () => document.removeEventListener("click", handleOutsideClick);
    }
  });

  return (
    <div ref={dropdownRef} className="relative">
      {/* Search Button */}
      <button
        className="btn bg-base-100 border-none hover:bg-base-200"
        onClick={handleDropdownToggle}
        type="button"
        aria-label="Search"
      >
        <IconSearch setclass="w-10 h-10" />
      </button>

      {/* Dropdown Content - Controlled by JavaScript */}
      {isOpen.value && (
        <div className="absolute right-0 top-full mt-3 z-50 p-2 shadow bg-base-100 rounded-box min-w-48 xs:w-48 lg:w-96 border border-base-300">
          <div className="space-y-2">
            {/*<input
              ref={inputRef}
              type="text"
              placeholder="Search..."
              value={query.value}
              onChange={handleOnSearch}
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  handleDropdownClose();
                }
              }}
              className="input input-bordered w-full"
              autoFocus
            />*/}
            <input
              ref={inputRef}
              type="text"
              placeholder="Search..."
              value={query.value}
              onInput={handleOnSearch} // Changed from onChange to onInput
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  handleDropdownClose();
                }
              }}
              className="input input-bordered w-full"
              autoFocus
            />

            {allResults.value.length > 0 && (
              <div className="text-sm text-gray-500 px-2">
                Found: {allResults.value.length} results
                {totalPages.value > 1 &&
                  ` (Page ${currentPage.value}/${totalPages.value})`}
              </div>
            )}

            {/* Top Pagination */}
            {allResults.value.length > MAXPOSTSPERPAGE && (
              <div className="px-2">
                <Pagination
                  currentPage={currentPage.value}
                  totalPages={totalPages.value}
                  onPageChange={goToPage}
                />
              </div>
            )}

            {/* Search Results */}
            <div className="max-h-96 overflow-y-auto space-y-2">
              {paginatedPosts.value.length > 0 &&
                paginatedPosts.value.map((post, index) => (
                  <a
                    key={index}
                    href={getLink(post)}
                    onClick={handleDropdownClose}
                    className="block"
                  >
                    <div className="card bg-base-200 hover:bg-base-300 transition-colors cursor-pointer">
                      <div className="card-body p-4">
                        <h3 className="card-title text-sm">
                          {post.frontmatter.title}
                          <div className="badge badge-sm badge-secondary">
                            {post.collection}
                          </div>
                        </h3>
                        <p className="text-xs opacity-70">
                          {post.frontmatter.description ||
                            post.frontmatter.desc}
                        </p>
                      </div>
                    </div>
                  </a>
                ))}
            </div>

            {/* Bottom Pagination */}
            {/*{allResults.value.length > MAXPOSTSPERPAGE && (
              <div className="px-2">
                <Pagination
                  currentPage={currentPage.value}
                  totalPages={totalPages.value}
                  onPageChange={goToPage}
                />
              </div>
            )}*/}
            {/* Search Results Status */}
            {query.value &&
              query.value.trim().length >= MINQUERYLENGTH &&
              allResults.value.length > 0 && (
                <div className="text-sm text-gray-500 px-2">
                  Found: {allResults.value.length} results
                  {totalPages.value > 1 &&
                    ` (Page ${currentPage.value}/${totalPages.value})`}
                </div>
              )}

            {/* Show "searching..." indicator for short queries */}
            {query.value &&
              query.value.trim().length > 0 &&
              query.value.trim().length < MINQUERYLENGTH && (
                <div className="text-sm text-gray-500 px-2">
                  Type at least 4 characters to search...
                </div>
              )}

            {/* Top Pagination */}
            {allResults.value.length > MAXPOSTSPERPAGE && (
              <div className="px-2">
                <Pagination
                  currentPage={currentPage.value}
                  totalPages={totalPages.value}
                  onPageChange={goToPage}
                />
              </div>
            )}

            {/*{query.value && allResults.value.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                No results found for "{query.value}"
              </div>
            )}*/}
            {/* No results message */}
            {query.value &&
              query.value.trim().length >= MINQUERYLENGTH &&
              allResults.value.length === 0 && (
                <div className="text-center py-8 text-gray-500">
                  No results found for "{query.value}"
                </div>
              )}
          </div>
        </div>
      )}
    </div>
  );
}

export default Search;
