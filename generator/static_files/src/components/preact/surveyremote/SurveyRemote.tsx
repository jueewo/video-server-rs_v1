import { h } from "preact";
import { useState, useEffect } from "preact/hooks";
import { Field } from "./SurveyField";

type Props = {
  surveyid: string;
  sourceid: string;
  showicon?: boolean;
  title?: string;
  submitbuttontxt?: string;
  thankyoumessage?: string;
  url: string;
};

interface FieldData {
  id: string;
  title: string;
  element: string;
  type: string;
  name: string;
  placeholder: string;
  required: boolean;
  multiple?: boolean;
}

interface PageData {
  label: string;
  fields: FieldData[];
}

interface SurveyData {
  id?: string;
  pages?: PageData[];
}

export default function SurveyComponent({
  surveyid,
  sourceid,
  title,
  showicon,
  submitbuttontxt,
  thankyoumessage,
  url,
}: Props) {
  const submit = submitbuttontxt ? submitbuttontxt : "Submit";
  const thankyou = thankyoumessage
    ? thankyoumessage
    : "Thank you for your submission";

  const [survey, setSurvey] = useState<SurveyData>({});
  const [alreadysubmitted, setAlreadysubmitted] = useState(false);
  const [loading, setLoading] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const sourceid_result = sourceid ? sourceid : "generic";
  const token = "34jkjfi442332112fjf432";

  const BASEURL = url;
  const url_results = BASEURL + "/api/results/" + sourceid_result;
  const url_survey = BASEURL + "/api/surveys/" + surveyid;

  const fetchSurveyData = async () => {
    setLoading(true);
    try {
      const resp = await fetch(url_survey, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
      });

      if (resp.ok) {
        const data = await resp.json();
        setSurvey(data);
        setLoaded(true);
      } else {
        console.error("Failed to fetch survey:", resp.status, resp.statusText);
      }
    } catch (error) {
      console.error("Error fetching survey:", error);
    } finally {
      setLoading(false);
    }
  };

  // Only fetch data on client side after component mounts
  useEffect(() => {
    if (!loaded && !loading) {
      fetchSurveyData();
    }
  }, []);

  async function onSubmit(e: Event) {
    e.preventDefault();
    const formData = new FormData(e.currentTarget as HTMLFormElement);

    const surveydatajson: Record<string, string> = {};

    for (let field of formData) {
      const [key, value] = field;
      surveydatajson[key] = value.toString();
      console.log(key, value);
    }

    try {
      const resp = await fetch(url_results, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          surveyid: surveyid,
          surveydata: surveydatajson,
        }),
      });

      if (resp.ok) {
        const data = await resp.json();
        console.log("Data:", data);
        setAlreadysubmitted(true);
      } else {
        console.error("Failed to submit survey:", resp.status);
      }
    } catch (error) {
      console.error("Error submitting survey:", error);
    }
  }

  return (
    <div>
      {loading && (
        <div class="flex justify-center p-20">
          <span class="loading loading-dots loading-lg"></span>
        </div>
      )}
      {!loading && survey?.id ? (
        <div class="w-full p-8 ">
          {!alreadysubmitted ? (
            <div>
              {title && (
                <div class="flex justify-center">
                  <div class="text-2xl">{title}</div>
                </div>
              )}
              <div class="flex justify-center">
                <form onSubmit={onSubmit}>
                  <div id="pages">
                    {survey?.pages &&
                      survey.pages.length > 0 &&
                      survey.pages.map((page, index) => (
                        <div key={`page-${index}`} id={page.label}>
                          {page.label && page.label.length > 0 && (
                            <div class="bg-primary w-full py-2 mt-12 mb-4">
                              {page.label}
                            </div>
                          )}

                          {page.fields.map((field) => (
                            <div key={field.id} class="py-1">
                              <Field fielddata={field} />
                            </div>
                          ))}
                        </div>
                      ))}
                  </div>

                  <div class="flex justify-center">
                    <button class="btn btn-primary" type="submit">
                      {submit}
                    </button>
                  </div>
                </form>
              </div>
            </div>
          ) : (
            <div>{thankyou}</div>
          )}
        </div>
      ) : (
        !loading && (
          <div class="flex justify-center my-8">
            <div role="alert" class="alert alert-warning">
              NO DATA AVAILABLE
            </div>
          </div>
        )
      )}
    </div>
  );
}
