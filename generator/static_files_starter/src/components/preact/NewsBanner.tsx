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
      <div class="w-3/4 mx-20 mt-20">
        <div class="text-center">
          <div class="max-w-md mx-auto">
            <h1 class="text-5xl font-bold font-heading text-text">{title}</h1>
            {desc && desc.length > 0 && <p class="pt-6 text-text-muted">{desc}</p>}
          </div>
        </div>
      </div>

      {loadeddata && loadeddata.length > 0 && (
        <div class="w-screen flex overflow-x-auto gap-4 p-4 bg-surface-alt rounded-xl my-9 scrollbar-thin">
          {loadeddata.map((newsitem, index) => (
            <div key={index} class="flex-shrink-0 w-4/5">
              <div
                id={`item-${index}`}
                class={`card-starter h-64 w-full bg-gradient-to-r from-primary to-blue-900 text-white flex ${newsitem.img ? "flex-row" : ""}`}
              >
                {newsitem.img && (
                  <figure class="flex-shrink-0">
                    <img src={newsitem.img} class="rounded-lg h-64 w-64 object-cover" />
                  </figure>
                )}
                <div class="p-5 flex flex-col justify-between flex-1">
                  <div>
                    <h2 class="text-lg font-bold font-heading">{newsitem.title}</h2>
                    <p class="mt-2 text-white/80">{newsitem.desc}</p>
                  </div>
                  <div class="flex justify-end">
                    <a href={newsitem.link}>
                      <button class="btn-starter bg-white/20 hover:bg-white/30 text-white">{newsitem.button} &#10095;</button>
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
            <a key={index} class="px-2 py-1 text-xs rounded border border-border text-text-muted hover:bg-surface-alt transition-colors" href={`#item-${index}`}>
              {index + 1}
            </a>
          ))}
        </div>
      )}

      {(!loadeddata || loadeddata.length === 0) && (
        <div class="p-4 bg-surface-alt rounded-xl my-9 w-full">
          <div class="w-full h-64 bg-surface border border-border rounded-xl">
            <div class="flex flex-col items-center justify-center h-full text-text-muted">
              <h2 class="text-lg font-bold mt-8">Loading news data</h2>
              <div class="mt-4 w-8 h-8 border-4 border-primary/30 border-t-primary rounded-full animate-spin"></div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
