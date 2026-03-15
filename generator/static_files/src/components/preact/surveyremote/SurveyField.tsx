// import { JSX } from "preact";
// import { IS_BROWSER } from "$fresh/runtime.ts";

import { IconInfo } from "../../icons/IconInfo.js";

// import { IconInfo } from "./icons/IconInfo.tsx";

// Field Type: text, textarea, radio, checkbox, select, date, time, datetime-local, email, file, hidden, image, month, number, password, range, reset, search, submit, tel, url, week

type Props = {
  fielddata: any;
};

export function Field(props: Props) {
  const field = props.fielddata;

  const showtitle = ["text", "email", "textarea", "file"];

  return (
    <>
      {/* <div class="py-4 bg-red-500">
        Field >> {field.title} | {field.type}  | {field.element}
      </div> */}

      {showtitle.includes(field.type) && (
        <label class="label">
          <span class="text-base label-text">{field.title}</span>
          {field.required && <span class="text-red-500">*</span>}
        </label>
      )}

      {field.type === "text" && field.element === "input" && (
        <>
          {/* <label class="label">
            <span class="text-base label-text">{field.title}</span>
            {field.required && <span class="text-red-500">*</span>}
          </label> */}
          <span>
            <input
              type={field.type}
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full input input-bordered input-primary"
            />
          </span>
        </>
      )}

      {field.type === "text" && field.element === "textarea" && (
        <>
          {/* <label class="label">
            <span class="text-base label-text">{field.title}</span>
            {field.required && <span class="text-red-500">*</span>}
          </label> */}
          <div>
            <textarea
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full h-24 input input-bordered input-primary"
            />
          </div>
        </>
      )}

      {field.type === "email" && (
        <>
          {/* <label class="label">
            <span class="text-base label-text">{field.title}</span>
            {field.required && <span class="text-red-500">*</span>}
          </label> */}
          <span>
            <input
              type={field.type}
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full input input-bordered input-primary"
            />
          </span>
        </>
      )}

      {field.type === "date" && (
        <>
          {/* <label class="label">
            <span class="text-base label-text">{field.title}</span>
            {field.required && <span class="text-red-500">*</span>}
          </label> */}
          <span>
            <input
              type={field.type}
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full input input-bordered input-primary"
            />
          </span>
        </>
      )}

      {field.type === "checkbox" && (
        <span>
          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text flex items-center">
                {field.title}

                <div
                  class="tooltip tooltip-top mx-2"
                  data-tip={field.placeholder}
                >
                  <IconInfo />
                </div>
                {field.required && <span class="text-red-500 mx-2">*</span>}
              </span>

              <input
                type={field.type}
                id={field.id}
                name={field.name}
                required={field.required ? true : false}
                class="checkbox"
              />
            </label>
          </div>
        </span>
      )}

      {field.type === "range" && (
        <span>
          <label class="label">
            <span class="text-base label-text flex items-center">
              {field.title}
              {field.required && <span class="text-red-500">*</span>}
              <div class="tooltip tooltip-top" data-tip={field.placeholder}>
                {/* <IconInfo w="w-8" h="h-8" classparam="w-8 h-8" /> */}
              </div>
            </span>
          </label>

          <input
            type={field.type}
            id={field.id}
            name={field.name}
            required={field.required ? true : false}
            placeholder={field.placeholder}
            class="range range-primary"
            min="0"
            max="10"
            step="1"
            value="-1"
          />
          <div class="w-full flex justify-between text-xs px-2">
            {[...Array(11).keys()].map((nr) => (
              <span>|</span>
            ))}
          </div>
        </span>
      )}

      {field.type === "file" && (
        <>
          {/* <label class="label">
              <span class="text-base label-text">
                {field.title}
                {field.required && <span class="text-red-500">*</span>}
              </span>
            </label> */}
          <span>
            <div class="form-control">
              <label class="label cursor-pointer">
                <span class="label-text flex items-center">
                  {field.title}

                  <div class="tooltip tooltip-top" data-tip={field.placeholder}>
                    {/* <IconInfo w="w-8" h="h-8" classparam="w-8 h-8" /> */}
                  </div>
                </span>

                {field.multiple ? (
                  <input
                    type={field.type}
                    id={field.id}
                    name={field.name}
                    required={field.required ? true : false}
                    multiple
                    class="fileupload"
                  />
                ) : (
                  <input
                    type={field.type}
                    id={field.id}
                    name={field.name}
                    required={field.required ? true : false}
                    class="fileupload"
                  />
                )}
              </label>
            </div>
          </span>
        </>
      )}
    </>
  );
}
