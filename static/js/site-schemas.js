/**
 * site-schemas.js
 *
 * Shared FIELD_SCHEMAS, COMPLEX_NOTE_TYPES, makeTemplate, and getElementLabel
 * used by both the page element editor (editor.html) and the collection entry
 * editor (entry_editor.html).
 */

// ── Field schemas ─────────────────────────────────────────────────────────────
var FIELD_SCHEMAS = {
    'Page-Metatags': [
        { path: 'content.title',       type: 'text', label: 'Page title' },
        { path: 'content.description', type: 'text', label: 'Description' },
        { path: 'content.keywords',    type: 'text', label: 'Keywords' },
        { path: 'content.author',      type: 'text', label: 'Author' },
    ],
    Hero: [
        { path: 'title',          type: 'text',       label: 'Title' },
        { path: 'desc',           type: 'text-array', label: 'Description' },
        { path: 'button',         type: 'text',       label: 'Button text' },
        { path: 'image',          type: 'image',      label: 'Image' },
        { path: 'link',           type: 'text',       label: 'Button link (URL)' },
        { path: 'fullscreen',     type: 'boolean',    label: 'Full screen' },
        { path: 'image_zoomable', type: 'boolean',    label: 'Zoomable image' },
        { path: 'ext',            type: 'boolean',    label: 'External link' },
    ],
    Hero2: [
        { path: 'title',       type: 'text',       label: 'Title' },
        { path: 'desc',        type: 'text-array', label: 'Description' },
        { path: 'button',      type: 'text',       label: 'Button text' },
        { path: 'image',       type: 'image',      label: 'Image' },
        { path: 'image_alt',   type: 'text',       label: 'Image alt text' },
        { path: 'bgimage',     type: 'image',      label: 'Background image' },
        { path: 'bgimage_alt', type: 'text',       label: 'Background image alt' },
        { path: 'tags',        type: 'text-array', label: 'Tags' },
        { path: 'link',        type: 'text',       label: 'Button link (URL)' },
        { path: 'fullscreen',  type: 'boolean',    label: 'Full screen' },
    ],
    Section: [
        { path: 'styleclass', type: 'text',    label: 'CSS class' },
        { path: 'alt',        type: 'boolean', label: 'Alternate layout' },
        { path: 'parallax',   type: 'boolean', label: 'Parallax' },
    ],
    TitleHero: [
        { path: 'title', type: 'text',       label: 'Title' },
        { path: 'desc',  type: 'text-array', label: 'Description' },
        { path: 'desc2', type: 'text-array', label: 'Description 2' },
        { path: 'image', type: 'image',      label: 'Image' },
        { path: 'h1',    type: 'boolean',    label: 'Render as H1' },
    ],
    TitleAlertBanner: [
        { path: 'title', type: 'text',       label: 'Title' },
        { path: 'desc',  type: 'text-array', label: 'Description' },
        { path: 'desc2', type: 'text-array', label: 'Description 2' },
    ],
    Carousel:      [],
    StatData:      [],
    SlidingGallery:[],
    Collection: [
        { path: 'title',             type: 'text',    label: 'Title' },
        { path: 'collection',        type: 'text',    label: 'Collection name' },
        { path: 'card',              type: 'text',    label: 'Card style (default / blog / info)' },
        { path: 'show_default_lang', type: 'boolean', label: 'Show default lang items' },
        { path: 'just_unique',       type: 'boolean', label: 'Unique (hide default-lang dupes)' },
        { path: 'filter_by_featured',type: 'boolean', label: 'Filter by featured' },
        { path: 'filter_featured',   type: 'boolean', label: 'Featured value to match' },
        { path: 'filter_filtertag',  type: 'text',    label: 'Filter tag (auto-enables tag filter)' },
    ],
    MdText: [
        { path: 'mdcollslug', type: 'text',    label: 'MDX collection slug (lang/slug)' },
        { path: 'title',      type: 'text',    label: 'Override title' },
        { path: 'image',      type: 'image',   label: 'Header image' },
        { path: 'fullscreen', type: 'boolean', label: 'Full screen height' },
    ],
    TeamGrid: [
        { path: 'filtertype', type: 'text', label: 'Filter type (e.g. "team", "advisor")' },
    ],
    NewsBanner: [
        { path: 'title',       type: 'text',       label: 'Title' },
        { path: 'desc',        type: 'text-array', label: 'Description' },
        { path: 'showbuttons', type: 'boolean',    label: 'Show buttons' },
    ],
    Presentation: [
        { path: 'title',    type: 'text',       label: 'Title' },
        { path: 'desc',     type: 'text-array', label: 'Description' },
        { path: 'datafile', type: 'text',       label: 'Slide file (.md)' },
    ],
    Process: [
        { path: 'title',    type: 'text',       label: 'Title' },
        { path: 'desc',     type: 'text-array', label: 'Description' },
        { path: 'datafile', type: 'text',       label: 'BPMN file (.bpmn)' },
    ],
    Video: [
        { path: 'title',       type: 'text',  label: 'Title' },
        { path: 'videoUrl',    type: 'text',  label: 'Video URL (.m3u8 or .mp4)' },
        { path: 'posterImage', type: 'image', label: 'Poster image' },
    ],
    CTA:      [],
    CTARemote: [
        { path: 'surveyid',        type: 'text', label: 'Survey ID' },
        { path: 'sourceid',        type: 'text', label: 'Source ID' },
        { path: 'title',           type: 'text', label: 'Form title' },
        { path: 'submitbuttontxt', type: 'text', label: 'Submit button text' },
        { path: 'thankyoumessage', type: 'text', label: 'Thank-you message' },
    ],
    LikeButton: [
        { path: 'sourceid', type: 'text', label: 'Source ID' },
    ],
    Hello: [
        { path: 'id', type: 'text', label: 'HTML id' },
    ],
};

var COMPLEX_NOTE_TYPES = ['Carousel', 'StatData', 'SlidingGallery', 'CTA'];

// ── Default templates ─────────────────────────────────────────────────────────
function makeTemplate(type) {
    var templates = {
        'Page-Metatags': { draft: false, element: 'Page-Metatags',
            content: { title: '', description: '', keywords: '', author: '' } },
        Hero: { draft: false, element: 'Hero',
            title: '', desc: [], button: '', image: '', link: '',
            fullscreen: false, image_zoomable: false, ext: false },
        Hero2: { draft: false, element: 'Hero2',
            title: '', desc: [], button: '', image: '', image_alt: '',
            bgimage: '', bgimage_alt: '', tags: [], link: '', fullscreen: false },
        Section: { draft: false, element: 'Section',
            styleclass: '', alt: false, parallax: false, elements: [] },
        TitleHero: { draft: false, element: 'TitleHero',
            title: '', desc: [], desc2: [], image: '', h1: true },
        TitleAlertBanner: { draft: false, element: 'TitleAlertBanner',
            title: '', desc: [], desc2: [] },
        Carousel: { draft: false, element: 'Carousel',
            data: [{ title: '', desc: '', image: '' }] },
        StatData: { draft: false, element: 'StatData',
            data: [{ title: '', value: '', desc: '' }] },
        Collection: { draft: false, element: 'Collection',
            title: '', collection: '', card: 'default',
            show_default_lang: false, just_unique: false,
            filter_by_featured: false, filter_featured: true,
            filter_filtertag: '' },
        MdText: { draft: false, element: 'MdText', mdcollslug: '', title: '', image: '', fullscreen: false },
        TeamGrid: { draft: false, element: 'TeamGrid', filtertype: '' },
        NewsBanner: { draft: false, element: 'NewsBanner',
            title: '', desc: [], showbuttons: true },
        SlidingGallery: { draft: false, element: 'SlidingGallery', dataid: '' },
        Presentation: { draft: false, element: 'Presentation',
            title: '', desc: [], datafile: '' },
        Process: { draft: false, element: 'Process',
            title: '', desc: [], datafile: '' },
        Video: { draft: false, element: 'Video',
            title: '', videoUrl: '', posterImage: '' },
        CTA: { draft: false, element: 'CTA', id: '', pages: [] },
        CTARemote: { draft: false, element: 'CTARemote',
            surveyid: '', sourceid: '', title: '',
            submitbuttontxt: 'Submit', thankyoumessage: 'Thank you!' },
        LikeButton: { draft: false, element: 'LikeButton', sourceid: '' },
        Hello: { draft: false, element: 'Hello', id: '' },
    };
    return JSON.parse(JSON.stringify(templates[type] || { draft: false, element: type }));
}

// ── Display label for an element ──────────────────────────────────────────────
function getElementLabel(el) {
    if (el.title)      return el.title;
    if (el.mdcollslug) return el.mdcollslug;
    if (el.collection) return el.collection;
    if (el.datafile)   return el.datafile;
    if (el.surveyid)   return el.surveyid;
    if (el.sourceid)   return el.sourceid;
    if (el.videoUrl)   return el.videoUrl;
    if (el.data && el.data.length) return el.data.length + ' items';
    if (el.content) {
        if (el.content.title)      return el.content.title;
        if (el.content.mdcollslug) return el.content.mdcollslug;
    }
    if (el.props) {
        if (el.props.collection)   return el.props.collection;
        if (el.props.datafile)     return el.props.datafile;
        if (el.props.surveyid)     return el.props.surveyid;
        if (el.props.sourceid)     return el.props.sourceid;
        if (el.props.videoUrl)     return el.props.videoUrl;
        if (el.props.data && el.props.data.length) return el.props.data.length + ' items';
    }
    return '';
}
