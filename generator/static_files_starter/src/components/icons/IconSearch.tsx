interface IconProps {
  setclass?: string;
}

export function IconSearch(props: IconProps) {
  const setclass = props.setclass || "w-6 h-6";
  return (
    <>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class={setclass}
      >
        <circle cx="11" cy="11" r="6" />
        <path stroke-linecap="round" d="M20 20l-4.35-4.35" />
      </svg>
    </>
  );
}
// hero icons (https://heroicons.com): search  | class={setclass}
