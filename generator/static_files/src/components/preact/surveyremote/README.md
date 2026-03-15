## SURVEY-REMOTE component (v0.1.0)

Used to collect data using the `webhook-01` server. Remote calls are loading a survey from the server, and submitting the results to the server.

Usage:
in `ElementRenderer.astro`

```
<SurveyRemote
          url={Config.datatool.url}
          surveyid={element.props.surveyid}
          sourceid={element.props.sourceid}
          title={element.props.title}
          submitbuttontxt={element.props.send}
          thankyoumessage={element.props.thankyoumessage}
          showicon client:load />


```

with

```
element: {
  "draft": false,
  "element": "CTARemote",
  "slot": null,
  "wrapper": null,
  "content": {},
  "props": {
    "surveyid": "survey_contact_1",
    "sourceid": "astro-cms-site-usermenu_",
    "title": "Contact us, we love to hear from you.",
    "submitbuttontxt": "Send",
    "thankyoumessage": "Thank you for your message!"
  }
},
```
