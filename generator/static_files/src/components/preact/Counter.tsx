import type { ComponentChildren } from "preact";
import type { Signal } from "@preact/signals";
// import { lazy, Suspense } from "preact/compat";
// import "./Counter.css";

// const Message = lazy(async () => import("./Message"));
// const Fallback = () => <p>Loading...</p>;

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
        <button class="btn" onClick={subtract}>
          -
        </button>
        <pre class="text-2xl mx-4">{count}</pre>
        <button class="btn" onClick={add}>
          +
        </button>
      </div>
      {/* <Suspense fallback={Fallback}>
        <Message>{children}</Message>
      </Suspense> */}
    </>
  );
}
