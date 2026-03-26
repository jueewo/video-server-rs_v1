import { IconInfo } from "../../icons/IconInfo.js";

// Field Type: text, textarea, radio, checkbox, select, date, time, datetime-local, email, file, hidden, image, month, number, password, range, reset, search, submit, tel, url, week

type Props = {
  fielddata: any;
};

export function Field(props: Props) {
  const field = props.fielddata;

  const showtitle = ["text", "email", "textarea", "file"];

  return (
    <>
      {showtitle.includes(field.type) && (
        <label class="block text-sm font-medium text-text mb-1">
          {field.title}
          {field.required && <span class="text-red-500 ml-1">*</span>}
        </label>
      )}

      {field.type === "text" && field.element === "input" && (
        <>
          <span>
            <input
              type={field.type}
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
            />
          </span>
        </>
      )}

      {field.type === "text" && field.element === "textarea" && (
        <>
          <div>
            <textarea
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full h-24 px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
            />
          </div>
        </>
      )}

      {field.type === "email" && (
        <>
          <span>
            <input
              type={field.type}
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
            />
          </span>
        </>
      )}

      {field.type === "date" && (
        <>
          <span>
            <input
              type={field.type}
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              placeholder={field.placeholder}
              class="w-full px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
            />
          </span>
        </>
      )}

      {field.type === "checkbox" && (
        <span>
          <label class="flex items-center cursor-pointer gap-3 py-2">
            <input
              type={field.type}
              id={field.id}
              name={field.name}
              required={field.required ? true : false}
              class="w-4 h-4 rounded border-border text-primary focus:ring-primary/50"
            />
            <span class="text-sm text-text flex items-center">
              {field.title}

              <span
                class="ml-2"
                title={field.placeholder}
              >
                <IconInfo />
              </span>
              {field.required && <span class="text-red-500 mx-2">*</span>}
            </span>
          </label>
        </span>
      )}

      {field.type === "range" && (
        <span>
          <label class="block text-sm font-medium text-text mb-1">
            <span class="flex items-center">
              {field.title}
              {field.required && <span class="text-red-500 ml-1">*</span>}
              <span class="ml-2" title={field.placeholder}>
              </span>
            </span>
          </label>

          <input
            type={field.type}
            id={field.id}
            name={field.name}
            required={field.required ? true : false}
            placeholder={field.placeholder}
            class="w-full accent-primary"
            min="0"
            max="10"
            step="1"
            value="-1"
          />
          <div class="w-full flex justify-between text-xs px-2 text-text-subtle">
            {[...Array(11).keys()].map((nr) => (
              <span>|</span>
            ))}
          </div>
        </span>
      )}

      {field.type === "file" && (
        <>
          <span>
            <label class="flex items-center cursor-pointer gap-3 py-2">
              <span class="text-sm text-text flex items-center">
                {field.title}
                <span class="ml-2" title={field.placeholder}>
                </span>
              </span>

              {field.multiple ? (
                <input
                  type={field.type}
                  id={field.id}
                  name={field.name}
                  required={field.required ? true : false}
                  multiple
                  class="text-sm text-text-muted file:mr-4 file:py-1 file:px-3 file:rounded-lg file:border-0 file:text-sm file:font-medium file:bg-primary/10 file:text-primary hover:file:bg-primary/20"
                />
              ) : (
                <input
                  type={field.type}
                  id={field.id}
                  name={field.name}
                  required={field.required ? true : false}
                  class="text-sm text-text-muted file:mr-4 file:py-1 file:px-3 file:rounded-lg file:border-0 file:text-sm file:font-medium file:bg-primary/10 file:text-primary hover:file:bg-primary/20"
                />
              )}
            </label>
          </span>
        </>
      )}
    </>
  );
}
