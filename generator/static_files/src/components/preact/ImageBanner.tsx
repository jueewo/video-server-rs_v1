import type { ComponentChildren } from "preact";
import type { Signal } from "@preact/signals";
import { signal, effect, computed } from "@preact/signals";

// import { heroposts } from "../../assets/heroposts.js";

// ... see: https://github.com/saadeghi/daisyui/discussions/1488

type Post = {
  draft: boolean;
  title: string;
  desc: string;
  img: string;
  // button: string;
  link?: {
    path: string; //.. relative to /src/content .. starting with /
    label?: string;
  };
};

type Props = {
  children?: ComponentChildren;
  // count: Signal<number>;
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

// const data = await fetch("/api/const/bannerposts").then((response) =>
//   response.json()
// );

// const data = await fetch("http://localhost:4321/api/const/bannerposts").then(
//   (response) => response.json()
// );
//
const TIMERVALUE = 3000; //.. in ms

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
    // console.log("counter : ", actindex.value);
    const carousel = document.getElementById(carouselId || "carouselid");
    if (carousel) {
      const target = document.querySelector<HTMLDivElement>(
        `#post${actindex.value}`,
      )!;
      const left = target.offsetLeft;
      carousel.scrollTo({ left: left });
      // console.log("left : ", left);
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
    //..first start ensures that the timer get just started once
    if (firststart.value && isAutoPlay) {
      firststart.value = false;
      // console.log("starting timer for : ", carouselId || "carouselid");
      startTimer(
        posts,
        carouselId || "carouselid",
        autoPlayMilliseconds || TIMERVALUE,
      );
    }
  });

  // this effect handles scrolling on actindex changes
  effect(() => {
    const carousel = document.getElementById(carouselId || "carouselid");
    if (carousel) {
      const target = document.querySelector<HTMLDivElement>(
        `#post${actindex.value}`,
      );
      if (target) {
        // console.log(
        //   "Scrolling to index:",
        //   actindex.value,
        //   "offsetLeft:",
        //   target.offsetLeft,
        // );
        isScrolling.value = true;
        carousel.scrollTo({ left: target.offsetLeft, behavior: "smooth" });
        // Reset the flag after scroll completes
        setTimeout(() => {
          isScrolling.value = false;
        }, 500);
      }
    }
  });

  // Add this effect to update actindex on manual scroll (e.g., swipe)
  // Replace scroll listener with IntersectionObserver for better swipe detection
  effect(() => {
    const carousel = document.getElementById(carouselId || "carouselid");
    if (!carousel) return;

    const observer = new IntersectionObserver(
      (entries) => {
        // Only update actindex if we're not programmatically scrolling
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
              // console.log("IntersectionObserver updating to index:", index);
              actindex.value = index;
              // Reset the timer if autoplay is active and not paused
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
      {/* <div class="flex items-center justify-center"> */}
      {/* {JSON.stringify(data.posts)} */}

      {children && (
        // <div class="w-full h-28 bg-primary flex items-center justify-center">
        <div class="w-full  bg-primary flex items-center justify-center">
          {children}
        </div>
      )}
      {/* HP: {JSON.stringify(heroposts)} */}
      <div
        id={carouselId || "carouselid"}
        className={
          classNameCarousel
            ? "carousel w-full ".concat(classNameCarousel)
            : "carousel w-full"
        }
      >
        {posts.map((post, index) => (
          <div
            id={`post${index}`}
            className="carousel-item w-full  justify-center"
          >
            <div class="w-2/3 ">
              <a
                href={post.link?.path || ""}
                aria-label={post.link?.label || ""}
              >
                <div class="text-2xl w-full flex justify-center">
                  {showTitle && <div>{showTitle && post.title}</div>}
                  {/* <div>{showTitle && post.title}</div> */}
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
            {/* <div>ai : {actindex.value}</div> */}
            <button
              onClick={() => {
                // console.log("Clicked index:", index);

                actindex.value = index;
                // Reset the timer if autoplay is active and not paused
                if (isAutoPlay && !isPaused.value) {
                  stopTimer();
                  startTimer(
                    posts,
                    carouselId || "carouselid",
                    autoPlayMilliseconds || 3000,
                  );
                }
              }}
              // Remove href to prevent page scrolling
              className={
                index === actindex.value ? "border border-4 border-primary" : ""
              }
            >
              <img
                src={post.img}
                className="w-20 "
                alt={"thumbnail_pic" + index + 1}
              />
            </button>
          </>
        ))}

        <div
          class="btn btn-sm btn-circle"
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
      {/* </div> */}
    </div>
  );
}
