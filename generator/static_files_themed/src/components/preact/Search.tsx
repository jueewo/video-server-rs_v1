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
        className="px-2 py-1 text-xs rounded border border-border text-text-muted hover:bg-surface-alt disabled:opacity-50 transition-colors"
      >
        Prev
      </button>

      <div className="flex space-x-1">
        {Array.from({ length: Math.min(totalPages, 5) }, (_, i) => {
          let pageNum;

          if (totalPages <= 5) {
            pageNum = i + 1;
          } else if (currentPage <= 3) {
            pageNum = i + 1;
            if (i === 4) return <span key={i}>...</span>;
          } else if (currentPage >= totalPages - 2) {
            pageNum = totalPages - 4 + i;
            if (i === 0) {
              return <span key={i}>...</span>;
            }
          } else {
            pageNum = currentPage - 2 + i;
            if (i === 0) return <span key={i}>...</span>;
            if (i === 4) return <span key={i}>...</span>;
          }

          return (
            <button
              key={i}
              onClick={() => onPageChange(pageNum)}
              className={`px-2 py-1 text-xs rounded border transition-colors ${
                currentPage === pageNum
                  ? "bg-primary text-white border-primary"
                  : "border-border text-text-muted hover:bg-surface-alt"
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
        className="px-2 py-1 text-xs rounded border border-border text-text-muted hover:bg-surface-alt disabled:opacity-50 transition-colors"
      >
        Next
      </button>
    </div>
  );
}

const _base = import.meta.env.BASE_URL;
const base = _base === "/" ? "" : _base.replace(/\/$/, "");

function getLink(post: Post) {
  const lang = post.slug.split("/")[0] || "en";
  const slugWithoutLang = post.slug.replace(/^[^/]+\//, "");
  return `${base}/${lang}/${post.collection}/${slugWithoutLang}`;
}

function Search({ searchList }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const fuse = new Fuse(searchList, options);

  const allResults = computed(() => {
    if (!query.value || query.value.trim().length < MINQUERYLENGTH) {
      return [];
    }
    return fuse.search(query.value).map((result) => result.item);
  });

  const totalPages = computed(() =>
    Math.ceil(allResults.value.length / MAXPOSTSPERPAGE),
  );

  const paginatedPosts = computed(() => {
    const startIndex = (currentPage.value - 1) * MAXPOSTSPERPAGE;
    return allResults.value.slice(startIndex, startIndex + MAXPOSTSPERPAGE);
  });

  effect(() => {
    if (isOpen.value && inputRef.current) {
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

  function handleDropdownToggle(e: Event) {
    e.preventDefault();
    e.stopPropagation();

    isOpen.value = !isOpen.value;

    if (isOpen.value) {
      setTimeout(() => inputRef.current?.focus(), 0);
      setTimeout(() => inputRef.current?.focus(), 10);
      setTimeout(() => inputRef.current?.focus(), 50);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }

  function handleDropdownClose() {
    isOpen.value = false;
    query.value = "";
    currentPage.value = 1;
  }

  function handleOutsideClick(e: Event) {
    if (
      dropdownRef.current &&
      !dropdownRef.current.contains(e.target as Node)
    ) {
      handleDropdownClose();
    }
  }

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
        className="p-2 rounded-md text-text-muted hover:text-text hover:bg-surface-raised transition-colors"
        onClick={handleDropdownToggle}
        type="button"
        aria-label="Search"
      >
        <IconSearch setclass="w-10 h-10" />
      </button>

      {/* Dropdown Content */}
      {isOpen.value && (
        <div className="absolute right-0 top-full mt-3 z-50 p-2 shadow-xl bg-surface border border-border rounded-xl min-w-48 xs:w-48 lg:w-96">
          <div className="space-y-2">
            <input
              ref={inputRef}
              type="text"
              placeholder="Search..."
              value={query.value}
              onInput={handleOnSearch}
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  handleDropdownClose();
                }
              }}
              className="w-full px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
              autoFocus
            />

            {allResults.value.length > 0 && (
              <div className="text-sm text-text-subtle px-2">
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
                    <div className="rounded-lg bg-surface-alt hover:bg-surface-raised transition-colors cursor-pointer">
                      <div className="p-4">
                        <h3 className="text-sm font-bold text-text flex items-center gap-2">
                          {post.frontmatter.title}
                          <span className="tag-theme bg-secondary/10 text-secondary text-xs">
                            {post.collection}
                          </span>
                        </h3>
                        <p className="text-xs text-text-muted mt-1">
                          {post.frontmatter.description ||
                            post.frontmatter.desc}
                        </p>
                      </div>
                    </div>
                  </a>
                ))}
            </div>

            {/* Search Results Status */}
            {query.value &&
              query.value.trim().length >= MINQUERYLENGTH &&
              allResults.value.length > 0 && (
                <div className="text-sm text-text-subtle px-2">
                  Found: {allResults.value.length} results
                  {totalPages.value > 1 &&
                    ` (Page ${currentPage.value}/${totalPages.value})`}
                </div>
              )}

            {/* Short query indicator */}
            {query.value &&
              query.value.trim().length > 0 &&
              query.value.trim().length < MINQUERYLENGTH && (
                <div className="text-sm text-text-subtle px-2">
                  Type at least 4 characters to search...
                </div>
              )}

            {/* Bottom Pagination */}
            {allResults.value.length > MAXPOSTSPERPAGE && (
              <div className="px-2">
                <Pagination
                  currentPage={currentPage.value}
                  totalPages={totalPages.value}
                  onPageChange={goToPage}
                />
              </div>
            )}

            {/* No results message */}
            {query.value &&
              query.value.trim().length >= MINQUERYLENGTH &&
              allResults.value.length === 0 && (
                <div className="text-center py-8 text-text-subtle">
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
