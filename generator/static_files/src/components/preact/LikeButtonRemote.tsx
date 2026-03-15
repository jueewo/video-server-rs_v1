import type { ComponentChildren } from "preact";
import type { Signal } from "@preact/signals";
import { signal, effect, computed, useSignal } from "@preact/signals";

import confetti from "canvas-confetti";
import { IconHeart } from "../icons/IconHeart.tsx";

// ... see: https://github.com/saadeghi/daisyui/discussions/1488

type Props = {
  label?: string;
  showicon: boolean;
  url: string;

  websiteid: string;
  pageid: string;
  token: string;
  userid?: string;
};

// const data = await fetch("/api/const/bannerposts").then((response) =>
//   response.json()
// );

// const data = await fetch("http://localhost:4321/api/const/bannerposts").then(
//   (response) => response.json()
// );

export default function LikeButton({
  label,
  showicon,
  url,
  websiteid,
  pageid,
  token,
  userid,
}: Props) {
  const likeclicked = () => {
    // console.log("HELLO");
    confetti();
    postLike();
    // fetchData();
  };

  const likes = useSignal(0);
  const alreadyliked = useSignal(false);
  const loading = useSignal(false);
  const loaded = useSignal(false);

  // const websiteid = "website1";
  // const pageid = "page1";
  // const token = "34jkjfi442332112fjf432";
  // const url_default =
  //   "http://localhost/api/likes/" + websiteid + "/" + pageid;
  // const url_default =
  //   "https://webhook-01.appkask.com/api/likes/" + websiteid + "/" + pageid;
  const ext_api_url =
    url + "/api/likes/" + websiteid + "/" + pageid + "/ext";

  const fetchData = async () => {
    loading.value = true;
    const resp = await fetch(ext_api_url, {
      method: "GET",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    });
    if (resp.ok) {
      const data = await resp.json();
      likes.value = data.likes;
    }
    loading.value = false;
  };

  const postLike = async () => {
    const resp = await fetch(ext_api_url, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    });
    if (resp.ok) {
      const data = await resp.json();
      likes.value = data.likes;
    }
    alreadyliked.value = true;
  };

  // console.log("Likes:", likes.value, likes.value < 0);
  // if (likes.value < 0) {
  //   console.log("Fetching data...");
  //   fetchData;
  // }
  if (!loaded.value) {
    fetchData().catch((e) => console.error(e));
    loaded.value = true;
  }
  // fetchData();

  return (
    <>
      <div class="flex justify-center">
        {!alreadyliked.value ? (
          <div
            class="btn btn-primary transition ease-in-out delay-150 hover:-translate-y-1 hover:scale-110 hover:btn-accent duration-300"
            onClick={likeclicked}
          >
            {showicon && <IconHeart setclass="w-8 h-8" />}

            <span>{label ? label : "Like"}</span>
            <span
              v-if="url"
              class="rounded-full  p-2 text-sm ml-4 bg-gray-300/30"
            >
              {loading.value ? (
                <span class="loading loading-dots loading-xs"></span>
              ) : (
                <span>{likes}</span>
              )}
            </span>
          </div>
        ) : (
          <div class="btn btn-primary transition ease-in-out delay-150  duration-300">
            {showicon && <IconHeart setclass="w-8 h-8" />}

            <span>{label ? label : "Like"}</span>
            <span
              v-if="url"
              class="rounded-full  p-2 text-sm ml-4 bg-gray-300/30"
            >
              {likes}
            </span>
          </div>
        )}
      </div>
    </>
  );
}
