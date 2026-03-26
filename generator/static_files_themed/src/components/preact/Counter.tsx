import type { ComponentChildren } from "preact";
import type { Signal } from "@preact/signals";

type Props = {
  children: ComponentChildren;
  count: Signal<number>;
};

export default function Counter({ children, count }: Props) {
  const add = () => count.value++;
  const subtract = () => count.value--;

  return (
    <>
      <div class="flex items-center justify-center">
        <button class="btn-theme" onClick={subtract}>
          -
        </button>
        <pre class="text-2xl mx-4">{count}</pre>
        <button class="btn-theme" onClick={add}>
          +
        </button>
      </div>
    </>
  );
}
