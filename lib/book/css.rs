pub static CSS_TEMPLATE: &str = r#"
@namespace epub "http://www.idpf.org/2007/ops";

p {
  margin: 8px 0 0;
}

span.underline { text-decoration: underline; }

span.strikethrough { text-decoration: line-through; }

span.smallcaps { font-variant: small-caps; }

.boxedtext, .keycap {
  border-style: solid;
  border-width: 1px;
  border-color: #000;
  padding: 1px;
}

span.gray50 {
  color: #7F7F7F; /* 50 % gray */
}

.gray-background, .reverse-video {
  background: #2E2E2E;
  color: #FFF;
}

.light-gray-background {
  background: #A0A0A0;
}

h1 {
  font-size: 1.5em;
  font-weight: bold;
  margin: 20px !important;
}

h2 {
  font-size: 1.3em;
  font-weight: bold;
  color: #8e0012;
  margin: 15px 0 8px 0 !important;
}

h3 {
  font-size: 1.1em;
  font-weight: bold;
  margin: 10px 0 8px 0 !important;
}

h4, h5, h6 {
  font-size: bold;
  font-weight: 1em;
  color: #555;
  margin: 9px 0 !important;
}

/* Add styling for the <nav> structure in the Table of Contents */

nav ol {
  list-style-type: decimal;
  margin-top: 8px !important;
  margin-bottom: 8px !important;
  margin-left: 20px !important;
  padding-left: 25px !important;
}

nav ol ol {
  list-style-type: lower-alpha;
}

nav ol ol ol {
  list-style-type: lower-roman;
}

nav ul {
  list-style-type: square;
  margin-top: 8px !important;
  margin-bottom: 8px !important;
  margin-left: 5px !important;
  padding-left: 20px !important;
}

nav ul ul {
  list-style-type: none;
  padding-left: 0 !important;
  margin-left: 0 !important;
}

nav ul ul li p:before {
  content: "\2014 \0020";
}

nav ul ul ul li p:before {
  content: "";
}

nav ul ul ul {
  list-style-type: square;
  margin-left: 20px !important;
  padding-left: 30px !important;
}

/* Set the size for the cover image. */

img.cover-image {
    max-width: 100%;
    max-height: 100%;
}

/* fonts (keep at bottom); using Free Serif and Sans as a fallback for its rich set of glyphs */

strong, span.bold {
  font-weight: bold;
}

/* tables */

div.table, table {
  margin: 10px auto !important;
  max-width: 95%;
  border-collapse: collapse;
  border-spacing: 0;
}

div.table, div.informaltable {
  page-break-inside: avoid;
}

tr {
  border-bottom: 1px solid #c3c3c3;
}

tr th {
  border-bottom: #9d9d9d 2px solid !important;
  border-top: #9d9d9d 2px solid !important;
}

tr:nth-of-type(even) {
  background-color: #f1f6fc;
}

th {
  color: #000;
  font-weight: bold;
}

td, th {
  padding: 0.3em;
  text-align: left;
  vertical-align: baseline;
  font-size: 80%;
}

div.informaltable table {
  margin: 10px auto !important;
}

div.informaltable table tr {
  border-bottom: none;
}

div.informaltable table tr:nth-of-type(even) {
  background-color: transparent;
}

div.informaltable td, div.informaltable th {
  border: #9d9d9d 1px solid;
}

div.table p.title {
  font-weight: normal;
  font-style: italic;
  margin: 20px 0 0 0 !important;
  text-align: center;
  padding: 0;
}
"#;
