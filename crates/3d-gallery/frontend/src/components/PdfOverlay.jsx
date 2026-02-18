import { useEffect, useRef, useState } from "preact/hooks";
import * as pdfjsLib from "pdfjs-dist";

// Share the same worker URL as PdfPresentation.js
pdfjsLib.GlobalWorkerOptions.workerSrc = `https://cdn.jsdelivr.net/npm/pdfjs-dist@${pdfjsLib.version}/build/pdf.worker.min.js`;

/**
 * Fullscreen PDF viewer overlay shown when a PDF presentation is clicked.
 *
 * Props:
 *   url    {string}  - URL to the PDF file (may include ?code=... for access-gated PDFs)
 *   title  {string}  - document title shown in the header
 *   onClose {()=>void} - called when the user closes the overlay
 */
export function PdfOverlay({ url, title, onClose }) {
  const canvasRef = useRef(null);
  const [pdfDoc, setPdfDoc] = useState(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const renderTaskRef = useRef(null);

  // Load the PDF document
  useEffect(() => {
    if (!url) return;
    setLoading(true);
    setError(null);
    setCurrentPage(1);

    pdfjsLib
      .getDocument(url)
      .promise.then((doc) => {
        setPdfDoc(doc);
        setTotalPages(doc.numPages);
        setLoading(false);
      })
      .catch((err) => {
        console.error("PdfOverlay: failed to load PDF:", err);
        setError("Failed to load PDF document.");
        setLoading(false);
      });
  }, [url]);

  // Render page whenever pdfDoc or currentPage changes
  useEffect(() => {
    if (!pdfDoc || !canvasRef.current) return;

    // Cancel any in-progress render
    if (renderTaskRef.current) {
      renderTaskRef.current.cancel();
    }

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");

    pdfDoc.getPage(currentPage).then((page) => {
      // Scale to fit the canvas container (max 900×650 logical pixels)
      const maxW = Math.min(window.innerWidth * 0.88, 900);
      const maxH = Math.min(window.innerHeight * 0.75, 650);

      const viewport = page.getViewport({ scale: 1.0 });
      const scale = Math.min(maxW / viewport.width, maxH / viewport.height);
      const scaled = page.getViewport({ scale });

      canvas.width = scaled.width;
      canvas.height = scaled.height;

      const task = page.render({ canvasContext: ctx, viewport: scaled });
      renderTaskRef.current = task;

      task.promise.catch((err) => {
        // Ignore cancelled render errors (normal when navigating quickly)
        if (err?.name !== "RenderingCancelledException") {
          console.error("PdfOverlay render error:", err);
        }
      });
    });
  }, [pdfDoc, currentPage]);

  // Keyboard navigation
  useEffect(() => {
    const handler = (e) => {
      if (e.key === "ArrowRight" || e.key === "ArrowDown") {
        setCurrentPage((p) => Math.min(p + 1, totalPages));
      } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
        setCurrentPage((p) => Math.max(p - 1, 1));
      } else if (e.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [totalPages, onClose]);

  const overlay = {
    position: "fixed",
    top: 0,
    left: 0,
    width: "100%",
    height: "100%",
    background: "rgba(0,0,0,0.92)",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "flex-start",
    zIndex: 2000,
    padding: "20px",
    overflowY: "auto",
  };

  const toolbar = {
    display: "flex",
    alignItems: "center",
    gap: "12px",
    marginBottom: "16px",
    color: "white",
    flexWrap: "wrap",
    justifyContent: "center",
    width: "100%",
    maxWidth: "900px",
  };

  const btnStyle = {
    padding: "8px 20px",
    background: "rgba(255,255,255,0.15)",
    border: "1px solid rgba(255,255,255,0.4)",
    borderRadius: "20px",
    color: "white",
    fontSize: "15px",
    cursor: "pointer",
  };

  return (
    <div style={overlay} onClick={onClose}>
      {/* Inner container — clicks don't propagate to backdrop */}
      <div
        style={{ display: "flex", flexDirection: "column", alignItems: "center", width: "100%" }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Toolbar */}
        <div style={toolbar}>
          <span style={{ fontSize: "18px", fontWeight: "bold", flex: 1, textAlign: "center" }}>
            {title}
          </span>
          <button
            style={btnStyle}
            disabled={currentPage <= 1}
            onClick={() => setCurrentPage((p) => Math.max(p - 1, 1))}
          >
            ◄ Prev
          </button>
          <span style={{ color: "white", fontSize: "15px", whiteSpace: "nowrap" }}>
            {currentPage} / {totalPages}
          </span>
          <button
            style={btnStyle}
            disabled={currentPage >= totalPages}
            onClick={() => setCurrentPage((p) => Math.min(p + 1, totalPages))}
          >
            Next ►
          </button>
          <a
            href={url}
            target="_blank"
            rel="noreferrer"
            style={{ ...btnStyle, textDecoration: "none" }}
          >
            ↗ Open
          </a>
          <button style={btnStyle} onClick={onClose}>
            Close (ESC)
          </button>
        </div>

        {/* Content */}
        {loading && (
          <p style={{ color: "white", fontSize: "18px", marginTop: "40px" }}>
            Loading PDF…
          </p>
        )}
        {error && (
          <p style={{ color: "#e94560", fontSize: "18px", marginTop: "40px" }}>
            {error}
          </p>
        )}
        {!loading && !error && (
          <canvas
            ref={canvasRef}
            style={{
              borderRadius: "6px",
              boxShadow: "0 4px 24px rgba(0,0,0,0.6)",
              maxWidth: "100%",
            }}
          />
        )}

        {/* Page info + keyboard hint */}
        {!loading && !error && (
          <p style={{ color: "rgba(255,255,255,0.5)", fontSize: "13px", marginTop: "12px" }}>
            Use ← → arrow keys to navigate pages • Click backdrop to close
          </p>
        )}
      </div>
    </div>
  );
}
