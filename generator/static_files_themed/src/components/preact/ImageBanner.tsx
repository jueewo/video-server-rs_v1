import type { ComponentChildren } from "preact";
import type { Signal } from "@preact/signals";
import { signal, effect, computed } from "@preact/signals";

type Post = {
  draft: boolean;
  title: string;
  desc: string;
  img: string;
  link?: {
    path: string;
    label?: string;
  };
};

type Props = {
  children?: ComponentChildren;
  posts: Post[];
  carouselId?: string;
  isAutoPlay?: boolean;
  autoPlayMilliseconds?: number;
  classNameCarousel?: string;
  classNameForImage?: string;
  showTitle?: boolean;
};

//.. ICON PLAY
const play = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    strokeWidth={1.5}
    stroke="currentColor"
    className="w-6 h-6"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653Z"
    />
  </svg>
);

//.. ICON PAUSE
const pause = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    stroke-width={1.5}
    stroke="currentColor"
    class="w-6 h-6"
  >
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      d="M15.75 5.25v13.5m-7.5-13.5v13.5"
    />
  </svg>
);

const TIMERVALUE = 3000;

const firststart = signal(true);
const actindex = signal(0);

const isPaused = signal(false);
const isScrolling = signal(false);

let timer: ReturnType<typeof setInterval> | null = null;

const stopTimer = () => {
  if (timer) clearInterval(timer);
};

const startTimer = (
  posts: object[],
  carouselId: string,
  autoPlayMilliseconds: number,
) => {
  timer = setInterval(() => {
    const nrpics = posts.length;
    if (actindex.value < nrpics - 1) actindex.value++;
    else actindex.value = 0;
    const carousel = document.getElementById(carouselId || "carouselid");
    if (carousel) {
      const target = document.querySelector<HTMLDivElement>(
        `#post${actindex.value}`,
      )!;
      const left = target.offsetLeft;
      carousel.scrollTo({ left: left });
    }
  }, autoPlayMilliseconds || TIMERVALUE);
};

export default function ImageBanner({
  posts,
  carouselId,
  isAutoPlay,
  autoPlayMilliseconds,
  classNameCarousel,
  classNameForImage,
  showTitle,
  children,
}: Props) {
  effect(() => {
    if (firststart.value && isAutoPlay) {
      firststart.value = false;
      startTimer(
        posts,
        carouselId || "carouselid",
        autoPlayMilliseconds || TIMERVALUE,
      );
    }
  });

  effect(() => {
    const carousel = document.getElementById(carouselId || "carouselid");
    if (carousel) {
      const target = document.querySelector<HTMLDivElement>(
        `#post${actindex.value}`,
      );
      if (target) {
        isScrolling.value = true;
        carousel.scrollTo({ left: target.offsetLeft, behavior: "smooth" });
        setTimeout(() => {
          isScrolling.value = false;
        }, 500);
      }
    }
  });

  effect(() => {
    const carousel = document.getElementById(carouselId || "carouselid");
    if (!carousel) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (isScrolling.value) return;

        entries.forEach((entry) => {
          if (entry.isIntersecting && entry.intersectionRatio > 0.5) {
            const id = entry.target.id;
            const index = parseInt(id.replace("post", ""));
            if (
              index !== actindex.value &&
              index >= 0 &&
              index < posts.length
            ) {
              actindex.value = index;
              if (isAutoPlay && !isPaused.value) {
                stopTimer();
                startTimer(
                  posts,
                  carouselId || "carouselid",
                  autoPlayMilliseconds || 3000,
                );
              }
            }
          }
        });
      },
      { root: carousel, threshold: 0.5 },
    );

    const items = carousel.querySelectorAll(".carousel-item");
    items.forEach((item) => observer.observe(item));

    return () => observer.disconnect();
  });

  return (
    <div class="flex flex-col items-center justify-center">
      {children && (
        <div class="w-full bg-primary flex items-center justify-center">
          {children}
        </div>
      )}
      <div
        id={carouselId || "carouselid"}
        className={
          classNameCarousel
            ? "flex overflow-x-auto snap-x snap-mandatory w-full scrollbar-none ".concat(classNameCarousel)
            : "flex overflow-x-auto snap-x snap-mandatory w-full scrollbar-none"
        }
      >
        {posts.map((post, index) => (
          <div
            id={`post${index}`}
            className="carousel-item flex-shrink-0 w-full snap-center justify-center"
          >
            <div class="w-full">
              <a
                href={post.link?.path || ""}
                aria-label={post.link?.label || ""}
              >
                <div class="text-2xl w-full flex justify-center text-text">
                  {showTitle && <div>{showTitle && post.title}</div>}
                </div>
                <img
                  src={post.img}
                  alt={post.title}
                  className={
                    classNameForImage
                      ? "w-full ".concat(classNameForImage)
                      : "w-full"
                  }
                />
              </a>
            </div>
          </div>
        ))}
      </div>

      <div className="flex items-center justify-center w-full pt-4 md:pt-12 gap-2">
        {posts.map((post, index) => (
          <>
            <button
              onClick={() => {
                actindex.value = index;
                if (isAutoPlay && !isPaused.value) {
                  stopTimer();
                  startTimer(
                    posts,
                    carouselId || "carouselid",
                    autoPlayMilliseconds || 3000,
                  );
                }
              }}
              className={
                index === actindex.value ? "border-4 border-primary rounded" : "rounded"
              }
            >
              <img
                src={post.img}
                className="w-20"
                alt={"thumbnail_pic" + index + 1}
              />
            </button>
          </>
        ))}

        <div
          class="p-2 rounded-full border border-border text-text-muted hover:bg-surface-alt transition-colors cursor-pointer"
          onClick={() => {
            isPaused.value
              ? startTimer(
                  posts,
                  carouselId || "carouselid",
                  autoPlayMilliseconds || 3000,
                )
              : stopTimer();
            isPaused.value = !isPaused.value;
          }}
        >
          {isPaused.value ? play() : pause()}
        </div>
      </div>
    </div>
  );
}
