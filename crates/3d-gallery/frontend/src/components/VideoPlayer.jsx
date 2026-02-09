import { useEffect, useRef } from "preact/hooks";
import Hls from "hls.js";

/**
 * VideoPlayer component with HLS.js support
 * Handles both HLS streams (.m3u8) and direct video files
 */
export function VideoPlayer({ url, autoPlay = true, style = {} }) {
  const videoRef = useRef(null);
  const hlsRef = useRef(null);

  useEffect(() => {
    const video = videoRef.current;
    if (!video || !url) return;

    // Check if URL is HLS stream
    if (url.includes(".m3u8")) {
      // HLS stream
      if (Hls.isSupported()) {
        // Use HLS.js for browsers that support it
        const hls = new Hls({
          debug: false,
          enableWorker: true,
          lowLatencyMode: false,
          backBufferLength: 90,
        });

        hlsRef.current = hls;
        hls.loadSource(url);
        hls.attachMedia(video);

        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          console.log("✓ HLS manifest loaded in overlay");
          if (autoPlay) {
            video
              .play()
              .catch((err) => console.warn("Autoplay prevented:", err));
          }
        });

        hls.on(Hls.Events.ERROR, (event, data) => {
          console.error("HLS overlay error:", data.type, data.details);
          if (data.fatal) {
            switch (data.type) {
              case Hls.ErrorTypes.NETWORK_ERROR:
                console.error("Fatal network error, trying to recover...");
                hls.startLoad();
                break;
              case Hls.ErrorTypes.MEDIA_ERROR:
                console.error("Fatal media error, trying to recover...");
                hls.recoverMediaError();
                break;
              default:
                console.error(
                  "Cannot recover from fatal error, destroying HLS instance",
                );
                hls.destroy();
                break;
            }
          }
        });
      } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
        // Safari native HLS support
        video.src = url;
        console.log("✓ Using native HLS in overlay (Safari)");
        if (autoPlay) {
          video
            .play()
            .catch((err) => console.warn("Autoplay prevented:", err));
        }
      } else {
        console.error("HLS not supported in this browser");
      }
    } else {
      // Direct video file (mp4, webm, etc.)
      video.src = url;
      if (autoPlay) {
        video.play().catch((err) => console.warn("Autoplay prevented:", err));
      }
    }

    // Cleanup function
    return () => {
      if (hlsRef.current) {
        console.log("Destroying HLS instance in overlay");
        hlsRef.current.destroy();
        hlsRef.current = null;
      }
      if (video) {
        video.pause();
        video.src = "";
      }
    };
  }, [url, autoPlay]);

  return (
    <video
      ref={videoRef}
      controls
      playsInline
      style={{
        maxWidth: "100%",
        maxHeight: "80vh",
        objectFit: "contain",
        borderRadius: "8px",
        boxShadow: "0 4px 20px rgba(0, 0, 0, 0.5)",
        ...style,
      }}
    />
  );
}
