import { h } from "preact";
import { useState, useEffect } from "preact/hooks";
import { formdefs } from "../../assets/formdefs.js";

interface EntityFormProps {
  id: string;
  formid: string;
  addonvalues?: string;
  title_override?: string;
  desc_override?: string;
  fullpage?: boolean;
}

interface ResponseMessage {
  status?: string;
  msg?: string;
}

interface FormDef {
  id: string;
  hiddenfield?: string;
  entityname: string;
  title: string;
  buttontxt: string;
  fields: Array<{
    id: string;
    title: string;
    element: string;
    type: string;
    name: string;
    placeholder: string;
    required: boolean;
  }>;
}

export default function EntityForm(props: EntityFormProps) {
  const {
    id,
    formid,
    addonvalues = "",
    title_override,
    desc_override,
    fullpage = false,
  } = props;

  const [responseMessage, setResponseMessage] =
    useState<ResponseMessage | null>(null);
  const [isfullpage] = useState<boolean>(fullpage);
  const [addons, setAddons] = useState<Record<string, any> | null>(null);

  const formdef: FormDef = formdefs[formid];
  const debug = false;

  useEffect(() => {
    if (addonvalues && addonvalues.length > 0) {
      try {
        const parsedAddons = JSON.parse(addonvalues);
        setAddons(parsedAddons);
      } catch (error) {
        // parsing failed
      }
    }
  }, [formid, addonvalues]);

  const submit = async (e: Event) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget as HTMLFormElement);
    const formdata: Record<string, any> = {};
    let valuehiddenfield = "";

    for (let field of formData) {
      const [key, value] = field;
      if (key !== formdef.hiddenfield) {
        formdata[key] = value;
      } else {
        valuehiddenfield = value.toString();
        if (valuehiddenfield.length > 0) {
          formdata[key] = value;
        }
      }
    }

    if (addons) {
      const keys = Object.keys(addons);
      for (let key of keys) {
        formdata[key] = addons[key];
      }
    }

    // Just if hiddenfield is empty (check for bots)
    if (valuehiddenfield.length === 0) {
      const response = await fetch("/api/entity/" + formdef.entityname, {
        method: "POST",
        body: JSON.stringify(formdata),
      });
      const data = await response.json();
      setResponseMessage(data);
    } else {
      const response = await fetch("/api/entity/honeypot", {
        method: "POST",
        body: JSON.stringify(formdata),
      });
      const data = await response.json();
      setResponseMessage(data);
    }
  };

  if (!formdef) {
    return <div class="text-text-muted">Form definition not found for formid: {formid}</div>;
  }

  return (
    <div class="relative flex flex-col justify-center overflow-visible">
      <div class="w-96 p-6 m-auto rounded-xl shadow-md border border-border bg-surface lg:max-w-lg">
        <h1 class="text-3xl font-semibold text-center text-primary font-heading">
          {title_override ? title_override : formdef.title}
        </h1>
        {desc_override && <p class="text-text-muted mt-2">{desc_override}</p>}

        {(!responseMessage || responseMessage.status !== "ok") && (
          <form onSubmit={submit} class="w-80 space-y-4 mt-4">
            <input type="text" hidden name="source" value={id} />
            <input type="text" hidden name="formid" value={formdef.id} />

            {formdef.hiddenfield && formdef.hiddenfield.length > 0 && (
              <div>
                <label class="block text-sm font-medium text-text-muted hidden">
                  {formdef.hiddenfield}
                </label>
                <input
                  type="text"
                  id={formdef.hiddenfield}
                  name={formdef.hiddenfield}
                  placeholder={formdef.hiddenfield}
                  class="w-full px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
                  hidden
                />
              </div>
            )}

            {formdef.fields.map((field) => (
              <div key={field.id}>
                {field.element === "input" && (
                  <>
                    {field.type !== "checkbox" ? (
                      <div>
                        <label class="block text-sm font-medium text-text mb-1">
                          {field.title}
                        </label>
                        <input
                          type={field.type}
                          id={field.id}
                          name={field.name}
                          required={field.required}
                          placeholder={field.placeholder}
                          class="w-full px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
                        />
                      </div>
                    ) : (
                      <label class="flex items-center cursor-pointer gap-3 py-2">
                        <input
                          type={field.type}
                          id={field.id}
                          name={field.name}
                          required={field.required}
                          class="w-4 h-4 rounded border-border text-primary focus:ring-primary/50"
                        />
                        <span class="text-sm text-text flex items-center">
                          {field.title}
                          <span
                            class="ml-2"
                            title={field.placeholder}
                          >
                            <svg
                              xmlns="http://www.w3.org/2000/svg"
                              fill="none"
                              viewBox="0 0 24 24"
                              stroke-width="1.5"
                              stroke="currentColor"
                              class="w-5 h-5 text-text-muted"
                            >
                              <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z"
                              />
                            </svg>
                          </span>
                        </span>
                      </label>
                    )}
                  </>
                )}

                {field.element === "textarea" && (
                  <div>
                    <label class="block text-sm font-medium text-text mb-1">
                      {field.title}
                    </label>
                    <textarea
                      id={field.id}
                      name={field.name}
                      required={field.required}
                      placeholder={field.placeholder}
                      class="w-full h-24 px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
                    />
                  </div>
                )}
              </div>
            ))}

            {debug && (
              <div>
                <label class="block text-sm font-medium text-text mb-1">
                  Firstname
                </label>
                <input
                  type="text"
                  id="firstname"
                  name="firstname"
                  placeholder="firstname"
                  class="w-full px-3 py-2 border border-border rounded-lg bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
              </div>
            )}

            <div class="flex flex-col items-center">
              <button class="btn-primary-theme">{formdef.buttontxt}</button>
            </div>

            <div class="text-xs mt-8 text-text-subtle text-center">
              By clicking `{formdef.buttontxt}`, you agree to our{" "}
              <a
                href="/info/privacy"
                class="text-xs text-text-muted underline hover:text-primary"
              >
                Privacy Policy
              </a>
            </div>
          </form>
        )}

        {responseMessage && responseMessage.msg && (
          <div class="mt-6 p-3 bg-green-100 text-green-800 rounded-lg text-sm">
            <span>{responseMessage.msg}</span>
          </div>
        )}
      </div>

      <style>{`
        input:placeholder-shown {
          font-size: 0.7em;
        }
        textarea:placeholder-shown {
          font-size: 0.7em;
        }
      `}</style>
    </div>
  );
}
