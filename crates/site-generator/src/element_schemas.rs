/// Element schema registry.
///
/// Each `ElementSchema` describes one page element type:
/// the fields it accepts, whether they're required, and their expected type.
/// Used by the validator to report warnings/errors during site generation.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    String,
    Bool,
    Number,
    StringArray,
    Array,
    Object,
    Any,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldDef {
    pub name: &'static str,
    pub required: bool,
    pub field_type: FieldType,
}

#[derive(Debug)]
pub struct ElementSchema {
    pub element: &'static str,
    pub description: &'static str,
    pub fields: &'static [FieldDef],
}

impl ElementSchema {
    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn required_fields(&self) -> impl Iterator<Item = &FieldDef> {
        self.fields.iter().filter(|f| f.required)
    }
}

macro_rules! field {
    ($name:literal, required, $t:ident) => {
        FieldDef { name: $name, required: true, field_type: FieldType::$t }
    };
    ($name:literal, optional, $t:ident) => {
        FieldDef { name: $name, required: false, field_type: FieldType::$t }
    };
}

// ── Schema definitions ────────────────────────────────────────────────────────

static TITLE_HERO_FIELDS: &[FieldDef] = &[
    field!("title",  required, String),
    field!("id",     optional, String),
    field!("h1",     optional, Bool),
    field!("h2",     optional, Bool),
    field!("desc",   optional, StringArray),
    field!("desc2",  optional, String),
    field!("image",  optional, String),
    field!("img",    optional, String),
];

static HERO_FIELDS: &[FieldDef] = &[
    field!("title",         required, String),
    field!("desc",          optional, StringArray),
    field!("image",         optional, String),
    field!("button",        optional, String),
    field!("link",          optional, Any),
    field!("fullscreen",    optional, Bool),
    field!("ext",           optional, Bool),
    field!("image_zoomable", optional, Bool),
];

static HERO2_FIELDS: &[FieldDef] = &[
    field!("title",       required, String),
    field!("desc",        optional, StringArray),
    field!("image",       optional, String),
    field!("image_alt",   optional, String),
    field!("bgimage",     optional, String),
    field!("bgimage_alt", optional, String),
    field!("button",      optional, String),
    field!("link",        optional, Any),
    field!("tags",        optional, StringArray),
    field!("fullscreen",  optional, Bool),
];

static SECTION_FIELDS: &[FieldDef] = &[
    field!("elements",    required, Array),
    field!("styleclass",  optional, String),
    field!("scrolleffect", optional, String),
    field!("alt",         optional, Bool),
    field!("parallax",    optional, Bool),
    field!("bgimage",     optional, String),
    field!("bgimage_alt", optional, String),
];

static COLLECTION_FIELDS: &[FieldDef] = &[
    field!("collection",        required, String),
    field!("title",             optional, String),
    field!("card",              optional, String),
    field!("show_default_lang", optional, Bool),
    field!("just_unique",       optional, Bool),
    field!("filter_by_featured", optional, Bool),
    field!("filter_featured",   optional, Bool),
    field!("filter_by_filtertag", optional, Bool),
    field!("filter_filtertag",  optional, String),
];

static STAT_DATA_FIELDS: &[FieldDef] = &[
    field!("id",     optional, String),
    field!("dataid", optional, String),
    field!("data",   optional, Array),
];

static CAROUSEL_FIELDS: &[FieldDef] = &[
    field!("id",     optional, String),
    field!("dataid", optional, String),
    field!("data",   optional, Array),
];

static SLIDING_GALLERY_FIELDS: &[FieldDef] = &[
    field!("id",     optional, String),
    field!("dataid", optional, String),
];

static TEAM_GRID_FIELDS: &[FieldDef] = &[
    field!("title",  optional, String),
    field!("data",   optional, Array),
    field!("filter", optional, String),
];

static PROCESS_FIELDS: &[FieldDef] = &[
    field!("id",       optional, String),
    field!("datafile", optional, String),
];

static PRESENTATION_FIELDS: &[FieldDef] = &[
    field!("id",       optional, String),
    field!("datafile", optional, String),
];

static MD_TEXT_FIELDS: &[FieldDef] = &[
    field!("id",          optional, String),
    field!("mdcollslug",  required, String),
    field!("image",       optional, String),
    field!("title",       optional, String),
    field!("fullscreen",  optional, Bool),
];

static TITLE_ALERT_BANNER_FIELDS: &[FieldDef] = &[
    field!("title", required, String),
    field!("desc",  optional, StringArray),
    field!("desc2", optional, String),
    field!("h2",    optional, Bool),
];

static NEWS_BANNER_FIELDS: &[FieldDef] = &[
    field!("title",       optional, String),
    field!("desc",        optional, StringArray),
    field!("showbuttons", optional, Bool),
];

static FAQ_FIELDS: &[FieldDef] = &[
    field!("faqdata", required, Array),
];

static CTA_FIELDS: &[FieldDef] = &[
    field!("id",    optional, String),
    field!("pages", optional, Array),
];

static SURVEY_FIELDS: &[FieldDef] = &[
    field!("id",    optional, String),
    field!("pages", optional, Array),
];

static CTA_REMOTE_FIELDS: &[FieldDef] = &[
    field!("surveyid",        required, String),
    field!("sourceid",        optional, String),
    field!("title",           optional, String),
    field!("send",            optional, String),
    field!("thankyoumessage", optional, String),
];

static LIKE_BUTTON_FIELDS: &[FieldDef] = &[
    field!("sourceid", required, String),
    field!("url",      optional, String),
];

static VIDEO_FIELDS: &[FieldDef] = &[
    field!("title",         optional, String),
    field!("videoUrl",      required, String),
    field!("posterImage",   optional, String),
    field!("fallbackImage", optional, String),
    field!("autoplay",      optional, Bool),
    field!("loop",          optional, Bool),
];

static HELLO_FIELDS: &[FieldDef] = &[
    field!("title", optional, String),
];

// ── Registry ──────────────────────────────────────────────────────────────────

pub static ELEMENT_SCHEMAS: &[ElementSchema] = &[
    ElementSchema { element: "TitleHero",       description: "Page title with optional description and image", fields: TITLE_HERO_FIELDS },
    ElementSchema { element: "Hero",            description: "Hero with image, text, and optional button link", fields: HERO_FIELDS },
    ElementSchema { element: "Hero2",           description: "Hero with optional background image and side image", fields: HERO2_FIELDS },
    ElementSchema { element: "Section",         description: "Container section wrapping child elements", fields: SECTION_FIELDS },
    ElementSchema { element: "Collection",      description: "Renders a content collection as a card grid", fields: COLLECTION_FIELDS },
    ElementSchema { element: "StatData",        description: "Statistics display from inline or remote data", fields: STAT_DATA_FIELDS },
    ElementSchema { element: "Carousel",        description: "Image/card carousel from inline or remote data", fields: CAROUSEL_FIELDS },
    ElementSchema { element: "SlidingGallery",  description: "Horizontally sliding image gallery", fields: SLIDING_GALLERY_FIELDS },
    ElementSchema { element: "TeamGrid",        description: "Team member grid from data", fields: TEAM_GRID_FIELDS },
    ElementSchema { element: "Process",         description: "Process steps from a data file", fields: PROCESS_FIELDS },
    ElementSchema { element: "Presentation",    description: "Slide-style presentation from a data file", fields: PRESENTATION_FIELDS },
    ElementSchema { element: "MdText",          description: "Renders an MDX entry from the mdcontent collection", fields: MD_TEXT_FIELDS },
    ElementSchema { element: "TitleAlertBanner", description: "Alert banner with title", fields: TITLE_ALERT_BANNER_FIELDS },
    ElementSchema { element: "NewsBanner",      description: "Dynamic news/blog banner", fields: NEWS_BANNER_FIELDS },
    ElementSchema { element: "FAQ",             description: "FAQ accordion from inline faqdata", fields: FAQ_FIELDS },
    ElementSchema { element: "CTA",             description: "Call-to-action from inline data", fields: CTA_FIELDS },
    ElementSchema { element: "Survey",          description: "Survey form from inline data", fields: SURVEY_FIELDS },
    ElementSchema { element: "CTARemote",       description: "Remote CTA / survey via datatool API", fields: CTA_REMOTE_FIELDS },
    ElementSchema { element: "LikeButton",      description: "Remote like/vote button via datatool API", fields: LIKE_BUTTON_FIELDS },
    ElementSchema { element: "Video",           description: "HLS/MP4 video player", fields: VIDEO_FIELDS },
    ElementSchema { element: "Hello",           description: "Simple greeting / placeholder element", fields: HELLO_FIELDS },
];

pub fn find_schema(element_type: &str) -> Option<&'static ElementSchema> {
    ELEMENT_SCHEMAS.iter().find(|s| s.element == element_type)
}
