import type { ComponentChildren } from "preact";
import type { Signal } from "@preact/signals";
import { signal, effect, computed, useSignal } from "@preact/signals";

import confetti from "canvas-confetti";
import { IconHeart } from "../icons/IconHeart.tsx";

type Props = {
  label?: string;
  showicon: boolean;
  url: string;

  websiteid: string;
  pageid: string;
  token: string;
  userid?: string;
};

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
    confetti();
    postLike();
  };

  const likes = useSignal(0);
  const alreadyliked = useSignal(false);
  const loading = useSignal(false);
  const loaded = useSignal(false);

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

  if (!loaded.value) {
    fetchData().catch((e) => console.error(e));
    loaded.value = true;
  }

  return (
    <>
      <div class="flex justify-center">
        {!alreadyliked.value ? (
          <div
            class="btn-primary-theme transition ease-in-out delay-150 hover:-translate-y-1 hover:scale-110 duration-300 cursor-pointer flex items-center gap-2"
            onClick={likeclicked}
          >
            {showicon && <IconHeart setclass="w-8 h-8" />}

            <span>{label ? label : "Like"}</span>
            <span
              class="rounded-full p-2 text-sm ml-4 bg-white/20"
            >
              {loading.value ? (
                <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
              ) : (
                <span>{likes}</span>
              )}
            </span>
          </div>
        ) : (
          <div class="btn-primary-theme flex items-center gap-2">
            {showicon && <IconHeart setclass="w-8 h-8" />}

            <span>{label ? label : "Like"}</span>
            <span
              class="rounded-full p-2 text-sm ml-4 bg-white/20"
            >
              {likes}
            </span>
          </div>
        )}
      </div>
    </>
  );
}
