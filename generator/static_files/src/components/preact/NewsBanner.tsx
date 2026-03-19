import { useState, useEffect } from "preact/hooks";

type NewsItem = {
  title: string;
  desc: string;
  img?: string;
  link: string;
  button: string;
};

type Props = {
  title: string;
  desc?: string;
  showbuttons?: boolean;
  lang?: string;
  base?: string;
};

export default function NewsBanner({
  title,
  desc,
  showbuttons = false,
  lang,
  base = "",
}: Props) {
  const [loadeddata, setLoadeddata] = useState<NewsItem[]>([]);

  useEffect(() => {
    fetch(base + "/api/const/bannerposts")
      .then((r) => r.ok ? r.json() : { posts: [] })
      .then((data) => setLoadeddata((data.posts || []).map((p: NewsItem) => ({
        ...p,
        img: p.img && !p.img.startsWith("http") ? base + p.img : p.img,
        link: p.link && !p.link.startsWith("http") ? base + p.link : p.link,
      }))))
      .catch(() => {});
  }, []);

  return (
    <>
      <div class="w-3/4 mx-20 mt-20 hero">
        <div class="hero-content text-center">
          <div class="max-w-md">
            <h1 class="text-5xl font-bold">{title}</h1>
            {desc && desc.length > 0 && <p class="pt-6">{desc}</p>}
          </div>
        </div>
      </div>

      {loadeddata && loadeddata.length > 0 && (
        <div class="w-screen carousel carousel-center p-4 space-x-4 bg-neutral rounded-box my-9">
          {loadeddata.map((newsitem, index) => (
            <div key={index} class="carousel-item w-4/5">
              <div
                id={`item-${index}`}
                class={`card h-64 w-full bg-gradient-to-r from-primary to-blue-900 text-white ${newsitem.img ? "card-side" : ""}`}
              >
                {newsitem.img && (
                  <figure>
                    <img src={newsitem.img} class="rounded-box h-64 w-64" />
                  </figure>
                )}
                <div class="card-body">
                  <h2 class="card-title">{newsitem.title}</h2>
                  <p>{newsitem.desc}</p>
                  <div class="card-actions justify-end">
                    <a href={newsitem.link}>
                      <button class="btn">{newsitem.button} ❯</button>
                    </a>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {loadeddata && loadeddata.length > 0 && showbuttons && (
        <div class="flex justify-center w-full py-2 gap-2">
          {loadeddata.map((_, index) => (
            <a key={index} class="btn btn-xs" href={`#item-${index}`}>
              {index + 1}
            </a>
          ))}
        </div>
      )}

      {(!loadeddata || loadeddata.length === 0) && (
        <div class="p-4 space-x-4 bg-neutral rounded-box my-9 w-full">
          <div class="card w-full h-64 bg-base-200 text-base-content">
            <div class="card-body items-center text-center">
              <h2 class="card-title mt-8">Loading news data</h2>
              <span class="loading loading-ball loading-lg"></span>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
