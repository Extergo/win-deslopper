/* @ds-bundle: {"format":4,"namespace":"DeslopperDesignSystem_c828b3","components":[{"name":"Badge","sourcePath":"components/core/Badge.jsx"},{"name":"Button","sourcePath":"components/core/Button.jsx"},{"name":"Chip","sourcePath":"components/core/Chip.jsx"},{"name":"CountChip","sourcePath":"components/core/CountChip.jsx"},{"name":"Eyebrow","sourcePath":"components/core/Eyebrow.jsx"},{"name":"Glyph","sourcePath":"components/core/Glyph.jsx"},{"name":"Callout","sourcePath":"components/feedback/Callout.jsx"},{"name":"ProgressPanel","sourcePath":"components/feedback/ProgressPanel.jsx"},{"name":"SafetyNote","sourcePath":"components/feedback/SafetyNote.jsx"},{"name":"Seal","sourcePath":"components/feedback/Seal.jsx"},{"name":"StateChange","sourcePath":"components/feedback/StateChange.jsx"},{"name":"StatusText","sourcePath":"components/feedback/StatusText.jsx"},{"name":"Checkbox","sourcePath":"components/forms/Checkbox.jsx"},{"name":"Field","sourcePath":"components/forms/Field.jsx"},{"name":"SearchInput","sourcePath":"components/forms/SearchInput.jsx"},{"name":"Select","sourcePath":"components/forms/Select.jsx"},{"name":"EmptyState","sourcePath":"components/layout/EmptyState.jsx"},{"name":"FactGrid","sourcePath":"components/layout/FactGrid.jsx"},{"name":"Hero","sourcePath":"components/layout/Hero.jsx"},{"name":"MetricCard","sourcePath":"components/layout/MetricCard.jsx"},{"name":"PageHeader","sourcePath":"components/layout/PageHeader.jsx"},{"name":"SectionHeading","sourcePath":"components/layout/SectionHeading.jsx"},{"name":"Surface","sourcePath":"components/layout/Surface.jsx"},{"name":"Dialog","sourcePath":"components/navigation/Dialog.jsx"},{"name":"Disclosure","sourcePath":"components/navigation/Disclosure.jsx"},{"name":"ListRow","sourcePath":"components/navigation/ListRow.jsx"},{"name":"NavItem","sourcePath":"components/navigation/NavItem.jsx"}],"sourceHashes":{"components/core/Badge.jsx":"3ce10be41071","components/core/Button.jsx":"e531f0aa5047","components/core/Chip.jsx":"af8d1ac51016","components/core/CountChip.jsx":"45e2299ee323","components/core/Eyebrow.jsx":"5d940d8b69a7","components/core/Glyph.jsx":"e2819796ff90","components/feedback/Callout.jsx":"1931d5a01bea","components/feedback/ProgressPanel.jsx":"1866bfed97ad","components/feedback/SafetyNote.jsx":"6f1c330281cf","components/feedback/Seal.jsx":"af46ea3601ac","components/feedback/StateChange.jsx":"3f86e0e49250","components/feedback/StatusText.jsx":"49af2c3ed708","components/forms/Checkbox.jsx":"30b735eb120a","components/forms/Field.jsx":"de88faca266d","components/forms/SearchInput.jsx":"90c523fae5b2","components/forms/Select.jsx":"4b118f31266e","components/layout/EmptyState.jsx":"3191cfec20b9","components/layout/FactGrid.jsx":"1a76e63d1ac8","components/layout/Hero.jsx":"83bafb6f7de2","components/layout/MetricCard.jsx":"741e7ac71ea2","components/layout/PageHeader.jsx":"a368de319fc6","components/layout/SectionHeading.jsx":"b81270b2b4d6","components/layout/Surface.jsx":"53cd7eef3488","components/navigation/Dialog.jsx":"3b6744c0aaef","components/navigation/Disclosure.jsx":"ff20ef48a5b9","components/navigation/ListRow.jsx":"df9ad70b03c4","components/navigation/NavItem.jsx":"d78376c7a0c7","ui_kits/deslopper-app/ComponentsScreen.jsx":"ca3a5952bbe3","ui_kits/deslopper-app/Screens.jsx":"a004a0c18557","ui_kits/deslopper-app/Shell.jsx":"3826e3a5202d","ui_kits/deslopper-app/data.js":"dd8dc25b03b6"},"inlinedExternals":[],"unexposedExports":[]} */

(() => {

const __ds_ns = (window.DeslopperDesignSystem_c828b3 = window.DeslopperDesignSystem_c828b3 || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// components/core/Badge.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Badge({
  tone = 'accent',
  children,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("span", _extends({
    className: ['ds-badge', tone !== 'accent' && 'ds-badge--' + tone, className].filter(Boolean).join(' ')
  }, rest), children);
}
Object.assign(__ds_scope, { Badge });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Badge.jsx", error: String((e && e.message) || e) }); }

// components/core/Button.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Button({
  variant = 'secondary',
  size = 'md',
  block = false,
  children,
  className = '',
  ...rest
}) {
  const cls = ['ds-btn', 'ds-btn--' + variant, size !== 'md' && 'ds-btn--' + size, block && 'ds-btn--block', className].filter(Boolean).join(' ');
  return /*#__PURE__*/React.createElement("button", _extends({
    type: "button",
    className: cls
  }, rest), children);
}
Object.assign(__ds_scope, { Button });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Button.jsx", error: String((e && e.message) || e) }); }

// components/core/Chip.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Chip({
  active = false,
  children,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("button", _extends({
    type: "button",
    "aria-pressed": active,
    className: ['ds-chip', className].filter(Boolean).join(' ')
  }, rest), children);
}
Object.assign(__ds_scope, { Chip });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Chip.jsx", error: String((e && e.message) || e) }); }

// components/core/CountChip.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function CountChip({
  label,
  value,
  className = '',
  children,
  ...rest
}) {
  if (value === undefined) return /*#__PURE__*/React.createElement("span", _extends({
    className: ['ds-countchip', className].filter(Boolean).join(' ')
  }, rest), children ?? label);
  return /*#__PURE__*/React.createElement("span", _extends({
    className: ['ds-supportchip', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("strong", null, value), /*#__PURE__*/React.createElement("small", null, label));
}
Object.assign(__ds_scope, { CountChip });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/CountChip.jsx", error: String((e && e.message) || e) }); }

// components/core/Eyebrow.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Eyebrow({
  children,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("span", _extends({
    className: ['ds-eyebrow', className].filter(Boolean).join(' ')
  }, rest), children);
}
Object.assign(__ds_scope, { Eyebrow });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Eyebrow.jsx", error: String((e && e.message) || e) }); }

// components/core/Glyph.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Glyph({
  children,
  tone = 'tonal',
  size = 'md',
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("span", _extends({
    "aria-hidden": "true",
    className: ['ds-glyph', tone !== 'tonal' && 'ds-glyph--' + tone, size !== 'md' && 'ds-glyph--' + size, className].filter(Boolean).join(' ')
  }, rest), children);
}
Object.assign(__ds_scope, { Glyph });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Glyph.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Callout.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Callout({
  tone = 'info',
  title,
  children,
  note,
  trailing,
  banner = false,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ['ds-callout', tone !== 'info' && 'ds-callout--' + tone, banner && 'ds-callout--banner', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("div", null, title ? /*#__PURE__*/React.createElement("strong", null, title) : null, children ? /*#__PURE__*/React.createElement("p", null, children) : null, note ? /*#__PURE__*/React.createElement("small", null, note) : null), trailing);
}
Object.assign(__ds_scope, { Callout });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Callout.jsx", error: String((e && e.message) || e) }); }

// components/feedback/ProgressPanel.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function ProgressPanel({
  status,
  completed = 0,
  total = 1,
  detail,
  note,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("section", _extends({
    className: ['ds-progresspanel', className].filter(Boolean).join(' '),
    "aria-live": "polite"
  }, rest), /*#__PURE__*/React.createElement("div", {
    className: "ds-sectionheading"
  }, /*#__PURE__*/React.createElement("strong", null, status), /*#__PURE__*/React.createElement("span", null, completed, " of ", total)), /*#__PURE__*/React.createElement("progress", {
    className: "ds-progress",
    max: total || 1,
    value: completed
  }), detail ? /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      fontSize: 'var(--type-supporting)'
    }
  }, detail) : null, note ? /*#__PURE__*/React.createElement("div", {
    className: "ds-callout",
    style: {
      marginTop: 'var(--space-3)'
    }
  }, /*#__PURE__*/React.createElement("span", null, note)) : null);
}
Object.assign(__ds_scope, { ProgressPanel });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/ProgressPanel.jsx", error: String((e && e.message) || e) }); }

// components/feedback/SafetyNote.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function SafetyNote({
  title,
  children,
  internal = false,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("section", _extends({
    className: ['ds-safetynote', internal && 'ds-safetynote--internal', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("strong", null, /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true"
  }, "\u25CF"), " ", title), children ? /*#__PURE__*/React.createElement("p", null, children) : null);
}
Object.assign(__ds_scope, { SafetyNote });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/SafetyNote.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Seal.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Seal({
  mark = '✓',
  title,
  note,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ['ds-seal', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true"
  }, mark), /*#__PURE__*/React.createElement("strong", null, title), note ? /*#__PURE__*/React.createElement("small", null, note) : null);
}
Object.assign(__ds_scope, { Seal });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Seal.jsx", error: String((e && e.message) || e) }); }

// components/feedback/StateChange.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function StateChange({
  steps = [],
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ['ds-statechange', className].filter(Boolean).join(' ')
  }, rest), steps.map((s, i) => [/*#__PURE__*/React.createElement("span", {
    key: s.label
  }, /*#__PURE__*/React.createElement("small", null, s.label), /*#__PURE__*/React.createElement("strong", null, s.value)), i < steps.length - 1 ? /*#__PURE__*/React.createElement("span", {
    key: s.label + '-arrow',
    "aria-hidden": "true"
  }, "\u2192") : null]));
}
Object.assign(__ds_scope, { StateChange });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/StateChange.jsx", error: String((e && e.message) || e) }); }

// components/feedback/StatusText.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const TONES = {
  Observed: 'observed',
  'Externally managed': 'managed',
  'Permission limited': 'limited',
  'Failed inspection': 'failed',
  Unknown: 'unknown'
};
function StatusText({
  children,
  tone,
  className = '',
  ...rest
}) {
  const key = tone ?? TONES[String(children)] ?? '';
  return /*#__PURE__*/React.createElement("span", _extends({
    className: ['ds-status', key && 'ds-status--' + key, className].filter(Boolean).join(' ')
  }, rest), children);
}
Object.assign(__ds_scope, { StatusText });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/StatusText.jsx", error: String((e && e.message) || e) }); }

// components/forms/Checkbox.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Checkbox({
  label,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("label", {
    className: ['ds-check', className].filter(Boolean).join(' ')
  }, /*#__PURE__*/React.createElement("input", _extends({
    type: "checkbox"
  }, rest)), label);
}
Object.assign(__ds_scope, { Checkbox });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Checkbox.jsx", error: String((e && e.message) || e) }); }

// components/forms/Field.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Field({
  label,
  hint,
  multiline = false,
  className = '',
  ...rest
}) {
  const Control = multiline ? 'textarea' : 'input';
  return /*#__PURE__*/React.createElement("label", {
    className: ['ds-field', className].filter(Boolean).join(' ')
  }, /*#__PURE__*/React.createElement("span", null, label), /*#__PURE__*/React.createElement(Control, _extends({
    className: multiline ? 'ds-textarea' : 'ds-input'
  }, rest)), hint ? /*#__PURE__*/React.createElement("small", null, hint) : null);
}
Object.assign(__ds_scope, { Field });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Field.jsx", error: String((e && e.message) || e) }); }

// components/forms/SearchInput.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function SearchInput({
  label = 'Search',
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("label", {
    className: ['ds-search', className].filter(Boolean).join(' ')
  }, /*#__PURE__*/React.createElement("span", {
    className: "ds-sr"
  }, label), /*#__PURE__*/React.createElement("input", _extends({
    className: "ds-input",
    type: "search"
  }, rest)));
}
Object.assign(__ds_scope, { SearchInput });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/SearchInput.jsx", error: String((e && e.message) || e) }); }

// components/forms/Select.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Select({
  label,
  options = [],
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("label", {
    className: ['ds-field', className].filter(Boolean).join(' ')
  }, /*#__PURE__*/React.createElement("span", null, label), /*#__PURE__*/React.createElement("select", _extends({
    className: "ds-select"
  }, rest), options.map(o => /*#__PURE__*/React.createElement("option", {
    key: o.value,
    value: o.value
  }, o.label))));
}
Object.assign(__ds_scope, { Select });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Select.jsx", error: String((e && e.message) || e) }); }

// components/layout/EmptyState.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function EmptyState({
  title,
  children,
  action,
  large = false,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ['ds-empty', large && 'ds-empty--large', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("strong", null, title), children ? /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, children) : null, action ? /*#__PURE__*/React.createElement("div", {
    className: "ds-row",
    style: {
      justifyContent: 'center',
      marginTop: 'var(--space-4)'
    }
  }, action) : null);
}
Object.assign(__ds_scope, { EmptyState });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/EmptyState.jsx", error: String((e && e.message) || e) }); }

// components/layout/FactGrid.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function FactGrid({
  facts = [],
  mono = false,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("dl", _extends({
    className: ['ds-facts', mono && 'ds-facts--mono', className].filter(Boolean).join(' ')
  }, rest), facts.map(f => /*#__PURE__*/React.createElement("div", {
    key: f.term
  }, /*#__PURE__*/React.createElement("dt", null, f.term), /*#__PURE__*/React.createElement("dd", null, f.value))));
}
Object.assign(__ds_scope, { FactGrid });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/FactGrid.jsx", error: String((e && e.message) || e) }); }

// components/layout/Hero.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Hero({
  eyebrow,
  title,
  statement,
  actions,
  trailing,
  id,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("header", _extends({
    className: ['ds-hero', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("div", null, eyebrow ? /*#__PURE__*/React.createElement("span", {
    className: "ds-eyebrow"
  }, eyebrow) : null, /*#__PURE__*/React.createElement("h1", {
    id: id,
    className: "ds-h1"
  }, title), statement ? /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, statement) : null, actions ? /*#__PURE__*/React.createElement("div", {
    className: "ds-row",
    style: {
      marginTop: 'var(--space-4)'
    }
  }, actions) : null), trailing);
}
Object.assign(__ds_scope, { Hero });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/Hero.jsx", error: String((e && e.message) || e) }); }

// components/layout/MetricCard.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function MetricCard({
  label,
  value,
  note,
  accent = false,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("article", _extends({
    className: ['ds-metric', accent && 'ds-metric--accent', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("span", null, label), /*#__PURE__*/React.createElement("strong", null, value), note ? /*#__PURE__*/React.createElement("small", null, note) : null);
}
Object.assign(__ds_scope, { MetricCard });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/MetricCard.jsx", error: String((e && e.message) || e) }); }

// components/layout/PageHeader.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function PageHeader({
  eyebrow,
  title,
  description,
  trailing,
  id,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("header", _extends({
    className: ['ds-pageheader', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("div", null, eyebrow ? /*#__PURE__*/React.createElement("span", {
    className: "ds-eyebrow"
  }, eyebrow) : null, /*#__PURE__*/React.createElement("h1", {
    id: id,
    className: "ds-h1"
  }, title), description ? /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, description) : null), trailing);
}
Object.assign(__ds_scope, { PageHeader });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/PageHeader.jsx", error: String((e && e.message) || e) }); }

// components/layout/SectionHeading.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function SectionHeading({
  eyebrow,
  title,
  description,
  trailing,
  id,
  level = 2,
  className = '',
  ...rest
}) {
  const H = 'h' + level;
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ['ds-sectionheading', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("div", null, eyebrow ? /*#__PURE__*/React.createElement("span", {
    className: "ds-eyebrow"
  }, eyebrow) : null, /*#__PURE__*/React.createElement(H, {
    id: id,
    className: level === 2 ? 'ds-h2' : 'ds-h3'
  }, title), description ? /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, description) : null), trailing);
}
Object.assign(__ds_scope, { SectionHeading });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/SectionHeading.jsx", error: String((e && e.message) || e) }); }

// components/layout/Surface.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Surface({
  tone = 'default',
  flush = false,
  panel = false,
  as: Tag = 'section',
  children,
  className = '',
  ...rest
}) {
  const cls = [panel ? 'ds-panel' : 'ds-surface', !panel && tone !== 'default' && 'ds-surface--' + tone, !panel && flush && 'ds-surface--flush', className].filter(Boolean).join(' ');
  return /*#__PURE__*/React.createElement(Tag, _extends({
    className: cls
  }, rest), children);
}
Object.assign(__ds_scope, { Surface });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/Surface.jsx", error: String((e && e.message) || e) }); }

// components/navigation/Dialog.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Dialog({
  eyebrow,
  title,
  children,
  actions,
  footnote,
  onDismiss,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "ds-scrim",
    onClick: onDismiss
  }, /*#__PURE__*/React.createElement("div", _extends({
    role: "dialog",
    "aria-modal": "true",
    className: ['ds-dialog', className].filter(Boolean).join(' '),
    onClick: e => e.stopPropagation()
  }, rest), eyebrow ? /*#__PURE__*/React.createElement("span", {
    className: "ds-eyebrow"
  }, eyebrow) : null, /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, title), children, actions ? /*#__PURE__*/React.createElement("div", {
    className: "ds-dialog__actions"
  }, actions) : null, footnote ? /*#__PURE__*/React.createElement("small", {
    style: {
      color: 'var(--color-text-secondary)'
    }
  }, footnote) : null));
}
Object.assign(__ds_scope, { Dialog });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/Dialog.jsx", error: String((e && e.message) || e) }); }

// components/navigation/Disclosure.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function Disclosure({
  summary,
  meta,
  trailing,
  inline = false,
  children,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("details", _extends({
    className: ['ds-disclosure', inline && 'ds-disclosure--inline', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement("summary", null, /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement("strong", null, summary), meta ? /*#__PURE__*/React.createElement("small", null, meta) : null), trailing ? /*#__PURE__*/React.createElement("span", {
    className: "ds-status"
  }, trailing) : null), /*#__PURE__*/React.createElement("div", {
    className: "ds-disclosure__body"
  }, children));
}
Object.assign(__ds_scope, { Disclosure });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/Disclosure.jsx", error: String((e && e.message) || e) }); }

// components/navigation/ListRow.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function ListRow({
  glyph,
  title,
  subtitle,
  status,
  selected = false,
  compact = false,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("button", _extends({
    type: "button",
    "aria-selected": selected,
    className: ['ds-listrow', compact && 'ds-listrow--compact', className].filter(Boolean).join(' ')
  }, rest), compact ? null : /*#__PURE__*/React.createElement(__ds_scope.Glyph, {
    tone: "accent"
  }, glyph), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement("strong", null, title), subtitle ? /*#__PURE__*/React.createElement("small", null, subtitle) : null), /*#__PURE__*/React.createElement("span", {
    className: "ds-status"
  }, status));
}
Object.assign(__ds_scope, { ListRow });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/ListRow.jsx", error: String((e && e.message) || e) }); }

// components/navigation/NavItem.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
function NavItem({
  glyph,
  label,
  description,
  active = false,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("button", _extends({
    type: "button",
    "aria-current": active ? 'page' : undefined,
    className: ['ds-navitem', className].filter(Boolean).join(' ')
  }, rest), /*#__PURE__*/React.createElement(__ds_scope.Glyph, {
    size: "sm"
  }, glyph), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement("strong", null, label), description ? /*#__PURE__*/React.createElement("small", null, description) : null));
}
Object.assign(__ds_scope, { NavItem });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/NavItem.jsx", error: String((e && e.message) || e) }); }

// ui_kits/deslopper-app/ComponentsScreen.jsx
try { (() => {
const {
  Surface,
  PageHeader,
  MetricCard,
  FactGrid,
  EmptyState,
  SectionHeading
} = window.DeslopperDesignSystem_c828b3;
const {
  Button,
  Badge,
  Chip,
  Glyph,
  Eyebrow,
  CountChip
} = window.DeslopperDesignSystem_c828b3;
const {
  Callout,
  StateChange,
  StatusText
} = window.DeslopperDesignSystem_c828b3;
const {
  ListRow,
  Disclosure
} = window.DeslopperDesignSystem_c828b3;
const {
  Field,
  Select,
  SearchInput
} = window.DeslopperDesignSystem_c828b3;
function matches(component, filter, query) {
  const q = query.trim().toLowerCase();
  const byFilter = filter === 'all' || filter === 'changed' && DRIFT.some(d => d.id === component.id && !d.resolved) || filter === 'managed' && component.status === 'Externally managed' || filter === 'permission_limited' && component.status === 'Permission limited' || filter === 'unknown' && component.status === 'Unknown' || filter === 'failed' && component.status === 'Failed inspection' || filter === 'desired_differs' && component.id === 'taskbar_widgets' || filter === 'package' && component.isPackage || filter === 'policy' && component.authority.includes('Policy') || filter === 'preference' && component.authority === 'UserPreference';
  const byQuery = !q || [component.name, component.category, component.purpose, component.id].join(' ').toLowerCase().includes(q);
  return byFilter && byQuery;
}
function OwnerControl({
  component,
  applied,
  busy,
  phase,
  onApply,
  onUndo
}) {
  const op = OWNER_OPERATIONS[component.id];
  const ready = op.status === 'ready';
  const verb = applied ? op.verbs[1] : op.verbs[0];
  return /*#__PURE__*/React.createElement(Surface, {
    tone: "accent",
    as: "section",
    style: {
      marginTop: 'var(--space-4)',
      display: 'grid',
      gap: 'var(--space-3)'
    }
  }, /*#__PURE__*/React.createElement(SectionHeading, {
    level: 3,
    eyebrow: "Owner control",
    title: `Control ${op.label}`,
    description: "Current-user scope. Deslopper checks the latest state, records the exact pre-state, applies one fixed setting, verifies it, and preserves Undo only when restoration is safe.",
    trailing: /*#__PURE__*/React.createElement(Badge, {
      tone: ready ? 'success' : 'warning'
    }, ready ? 'Undo supported' : 'Not actionable')
  }), /*#__PURE__*/React.createElement(Callout, {
    tone: ready ? 'info' : 'warning',
    title: op.status.replace(/_/g, ' '),
    note: `Scope: ${op.scope}`
  }, op.reason), busy ? /*#__PURE__*/React.createElement(Callout, {
    tone: "info",
    title: phase,
    note: "Keep Deslopper open while this local operation completes."
  }) : applied !== null && applied !== undefined && ready ? /*#__PURE__*/React.createElement(Callout, {
    tone: "success",
    title: "applied and verified",
    note: `Pre-state ${op.representation} captured; direct and detector verification agreed.`
  }, `${op.label} is now ${applied ? 'enabled' : 'disabled'} for this account.`) : null, /*#__PURE__*/React.createElement("div", {
    className: "ds-row"
  }, ready ? /*#__PURE__*/React.createElement(Button, {
    variant: "primary",
    disabled: busy,
    onClick: () => onApply(!applied)
  }, `${verb} ${op.label}`, op.verbs[0] === 'Hide' ? ' button' : '') : null, ready && applied !== null && applied !== undefined ? /*#__PURE__*/React.createElement(Button, {
    variant: "secondary",
    disabled: busy,
    onClick: onUndo
  }, `Undo last ${op.label} change`) : null), /*#__PURE__*/React.createElement(Callout, {
    title: "Latest Deslopper change",
    note: `${op.representation} → ${applied ? 'DWORD 1' : 'DWORD 0'} · Undo ${applied !== null && applied !== undefined ? 'available' : 'not available'}`
  }, applied !== null && applied !== undefined ? 'verified · 2026-08-16 09:52:10' : 'No change has been made from this account yet.'));
}
function PackageControl({
  component
}) {
  const pkg = PACKAGE_OPERATIONS[component.id];
  return /*#__PURE__*/React.createElement(Surface, {
    tone: "accent",
    as: "section",
    style: {
      marginTop: 'var(--space-4)',
      display: 'grid',
      gap: 'var(--space-3)'
    }
  }, /*#__PURE__*/React.createElement(SectionHeading, {
    level: 3,
    eyebrow: "Owner control",
    title: "Remove from this account",
    description: "One exact package identity, current user only. Removal captures complete target and dependency inventory first, then verifies absence directly and through the detector.",
    trailing: /*#__PURE__*/React.createElement(Badge, {
      tone: "neutral"
    }, pkg.restore)
  }), /*#__PURE__*/React.createElement(FactGrid, {
    mono: true,
    facts: [{
      term: 'Package identity',
      value: pkg.identity
    }, {
      term: 'Restore classification',
      value: pkg.restore
    }]
  }), /*#__PURE__*/React.createElement(Callout, {
    tone: "warning",
    title: "Package removal may remove local app data."
  }, "Deslopper never fabricates an Undo. Restore appears only when captured staged or provisioned state proves a deterministic local registration path."), /*#__PURE__*/React.createElement("div", {
    className: "ds-row"
  }, /*#__PURE__*/React.createElement(Button, {
    variant: "primary"
  }, "Remove for this account")));
}
function ComponentsScreen({
  state,
  actions
}) {
  const {
    filter,
    query,
    selectedId,
    applied,
    ownerBusy,
    ownerPhase,
    snapshot
  } = state;
  const shown = CATALOGUE.filter(c => matches(c, filter, query));
  const selected = CATALOGUE.find(c => c.id === selectedId);
  const drift = selected ? DRIFT.filter(d => d.id === selected.id && !d.resolved) : [];
  return /*#__PURE__*/React.createElement("section", {
    "aria-labelledby": "components-title"
  }, /*#__PURE__*/React.createElement(PageHeader, {
    id: "components-title",
    eyebrow: "21 registered components",
    title: "Browse what Deslopper observes",
    description: "Technical evidence stays secondary to the conclusion, its confidence, and its limits.",
    trailing: /*#__PURE__*/React.createElement(Badge, null, shown.length, " shown")
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gap: 'var(--space-3)',
      marginBottom: 'var(--space-4)'
    }
  }, /*#__PURE__*/React.createElement(SearchInput, {
    label: "Search components",
    placeholder: "Search name, purpose, or category",
    value: query,
    onChange: e => actions.setQuery(e.target.value)
  }), /*#__PURE__*/React.createElement("div", {
    className: "ds-chiprow",
    "aria-label": "Component filters"
  }, FILTERS.map(([id, label]) => /*#__PURE__*/React.createElement(Chip, {
    key: id,
    active: filter === id,
    onClick: () => actions.setFilter(id)
  }, label)))), /*#__PURE__*/React.createElement("div", {
    className: "app-split"
  }, /*#__PURE__*/React.createElement("div", {
    className: "app-list",
    "aria-live": "polite"
  }, shown.length ? shown.map(c => /*#__PURE__*/React.createElement(ListRow, {
    key: c.id,
    glyph: c.name.slice(0, 1),
    title: c.name,
    subtitle: `${c.category} · ${c.state}`,
    status: /*#__PURE__*/React.createElement(StatusText, null, c.status),
    selected: selectedId === c.id,
    onClick: () => actions.openComponent(c.id)
  })) : /*#__PURE__*/React.createElement(EmptyState, {
    title: "No components match"
  }, "Try a broader filter or clear the search.")), /*#__PURE__*/React.createElement(Surface, {
    panel: true,
    as: "section",
    "aria-live": "polite"
  }, selected ? /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("div", {
    className: "ds-sectionheading"
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement(Eyebrow, null, selected.category), /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, selected.name), /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, selected.purpose)), /*#__PURE__*/React.createElement(Badge, {
    tone: selected.status === 'Observed' ? 'success' : 'warning'
  }, selected.status)), /*#__PURE__*/React.createElement(FactGrid, {
    facts: [{
      term: 'Observed state',
      value: selected.state
    }, {
      term: 'Evidence freshness',
      value: snapshot ? selected.detectedAt : 'Not inspected'
    }, {
      term: 'Authority',
      value: `${selected.authority} · ${selected.confidence} confidence`
    }, {
      term: 'Applicability',
      value: selected.status === 'Not applicable' ? 'not supported on this edition' : 'supported'
    }, {
      term: 'Completeness',
      value: selected.isPackage ? 'complete for current user' : 'Not package-backed'
    }, {
      term: 'Desired state',
      value: selected.id === 'taskbar_widgets' ? 'Disabled' : 'Not configured'
    }, {
      term: 'Drift',
      value: drift.length ? `${drift.length} unresolved event(s)` : 'No unresolved drift'
    }, {
      term: 'Restart / sign-out',
      value: selected.restart
    }]
  }), /*#__PURE__*/React.createElement("section", {
    style: {
      marginTop: 'var(--space-4)',
      paddingTop: 'var(--space-4)',
      borderTop: 'var(--border-hairline)'
    }
  }, /*#__PURE__*/React.createElement("h3", {
    className: "ds-h3"
  }, "Why I reached this conclusion"), /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, selected.status === 'Permission limited' ? 'One or more package scopes could not be read from this account. Permission-limited does not mean absent.' : selected.status === 'Unknown' ? 'No supported representation was present. Absence of a value is a default, not a disabled state.' : 'A documented preference or policy value was read directly and matched the detector result.')), /*#__PURE__*/React.createElement("div", {
    className: "app-tradeoffs"
  }, /*#__PURE__*/React.createElement("article", null, /*#__PURE__*/React.createElement("h3", {
    className: "ds-h3"
  }, "What you gain"), /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, selected.benefit)), /*#__PURE__*/React.createElement("article", null, /*#__PURE__*/React.createElement("h3", {
    className: "ds-h3"
  }, "What to consider"), /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, selected.consider))), OWNER_OPERATIONS[selected.id] ? /*#__PURE__*/React.createElement(OwnerControl, {
    component: selected,
    applied: applied[selected.id],
    busy: ownerBusy,
    phase: ownerPhase,
    onApply: next => actions.applyOwner(selected.id, next),
    onUndo: () => actions.undoOwner(selected.id)
  }) : null, PACKAGE_OPERATIONS[selected.id] ? /*#__PURE__*/React.createElement(PackageControl, {
    component: selected
  }) : null, !OWNER_OPERATIONS[selected.id] && !PACKAGE_OPERATIONS[selected.id] ? /*#__PURE__*/React.createElement("section", {
    style: {
      marginTop: 'var(--space-4)',
      paddingTop: 'var(--space-4)',
      borderTop: 'var(--border-hairline)',
      display: 'grid',
      gap: 'var(--space-3)'
    }
  }, /*#__PURE__*/React.createElement(SectionHeading, {
    level: 3,
    eyebrow: "Inspection only",
    title: "Desired state and preview",
    description: "A desired state is a local comparison preference, not an instruction to Windows."
  }), /*#__PURE__*/React.createElement(Select, {
    label: "Desired state",
    options: [{
      value: 'disabled',
      label: 'Disabled (documented preference)'
    }, {
      value: 'enabled',
      label: 'Enabled (documented preference)'
    }]
  }), /*#__PURE__*/React.createElement(Field, {
    label: "Note",
    multiline: true,
    placeholder: "Why do you want this state? (stored locally)"
  }), /*#__PURE__*/React.createElement("div", {
    className: "ds-row"
  }, /*#__PURE__*/React.createElement(Button, {
    variant: "primary"
  }, "Save desired state"), /*#__PURE__*/React.createElement(Button, {
    variant: "secondary"
  }, "Generate preview")), /*#__PURE__*/React.createElement(Callout, {
    title: "Execution is unavailable in this build.",
    note: "This preview creates no approval nonce, transaction, or mutation plan."
  }, "The preview records current and desired values, authority, mechanism, restart implications, known risk, and rollback considerations.")) : null, /*#__PURE__*/React.createElement(Disclosure, {
    inline: true,
    summary: "Redacted technical evidence"
  }, /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, "Detector: ", selected.status.toLowerCase(), " \xB7 official support: supported"), /*#__PURE__*/React.createElement(FactGrid, {
    mono: true,
    facts: [{
      term: 'Source',
      value: selected.authority === 'Servicing' ? 'AppX shared scopes' : 'Current-user preference'
    }, {
      term: 'Confidence',
      value: `${selected.confidence} (evidence-weighted)`
    }]
  }), /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      fontSize: 'var(--type-supporting)'
    }
  }, "Install paths, filenames, and account identifiers are not stored. Reference: ", selected.documentation))) : /*#__PURE__*/React.createElement(EmptyState, {
    title: "Select a component",
    style: {
      marginTop: '18vh'
    }
  }, "Choose any of the 21 registered components to review its purpose, observed state, authority, applicability, desired state, and redacted evidence."))));
}
Object.assign(window, {
  ComponentsScreen
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/deslopper-app/ComponentsScreen.jsx", error: String((e && e.message) || e) }); }

// ui_kits/deslopper-app/Screens.jsx
try { (() => {
const {
  Surface,
  PageHeader,
  FactGrid,
  EmptyState,
  SectionHeading
} = window.DeslopperDesignSystem_c828b3;
const {
  Button,
  Badge,
  CountChip,
  Eyebrow
} = window.DeslopperDesignSystem_c828b3;
const {
  Callout,
  StateChange,
  StatusText
} = window.DeslopperDesignSystem_c828b3;
const {
  Disclosure
} = window.DeslopperDesignSystem_c828b3;
const {
  Select,
  Checkbox
} = window.DeslopperDesignSystem_c828b3;
function name(id) {
  return (CATALOGUE.find(c => c.id === id) || {}).name || id.replace(/_/g, ' ');
}
function DesiredScreen({
  actions
}) {
  return /*#__PURE__*/React.createElement("section", {
    "aria-labelledby": "desired-title"
  }, /*#__PURE__*/React.createElement(PageHeader, {
    id: "desired-title",
    eyebrow: "Local preferences",
    title: "Desired states and previews",
    description: "Compare what you want with what Deslopper observed. Nothing here can apply a Windows change.",
    trailing: /*#__PURE__*/React.createElement(Badge, null, "1 saved")
  }), /*#__PURE__*/React.createElement("div", {
    className: "app-two-col"
  }, /*#__PURE__*/React.createElement(Surface, {
    as: "article"
  }, /*#__PURE__*/React.createElement(Eyebrow, null, "validated"), /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, "Taskbar Widgets button"), /*#__PURE__*/React.createElement(FactGrid, {
    facts: [{
      term: 'Observed',
      value: 'Enabled'
    }, {
      term: 'Desired',
      value: 'Disabled'
    }, {
      term: 'Saved for',
      value: 'Windows 11 Pro build 26100'
    }, {
      term: 'Revision',
      value: '2'
    }]
  }), /*#__PURE__*/React.createElement(Callout, {
    title: "Not yet applied",
    note: "Automatic restoration is not available in this build."
  }, "This component has a registered Owner Mode operation; the preview remains a comparison record only."), /*#__PURE__*/React.createElement("div", {
    className: "ds-row",
    style: {
      marginTop: 'var(--space-3)'
    }
  }, /*#__PURE__*/React.createElement(Button, {
    variant: "secondary",
    onClick: () => actions.openComponent('taskbar_widgets')
  }, "Review or regenerate preview"))), /*#__PURE__*/React.createElement(Surface, null, /*#__PURE__*/React.createElement(SectionHeading, {
    level: 3,
    eyebrow: "Preview contract",
    title: "What a preview contains"
  }), /*#__PURE__*/React.createElement("ul", {
    className: "app-bullets"
  }, /*#__PURE__*/React.createElement("li", null, "Current and desired values, authority, and mechanism."), /*#__PURE__*/React.createElement("li", null, "Restart implications, known risk, and rollback considerations."), /*#__PURE__*/React.createElement("li", null, "Source inspection id, generation time, and schema version."), /*#__PURE__*/React.createElement("li", null, "An explicit declaration that execution is unavailable.")))));
}
function DriftScreen({
  state,
  actions
}) {
  const shown = DRIFT.filter(d => state.driftStatus === 'all' ? true : state.driftStatus === 'resolved' ? d.resolved : state.driftStatus === 'unreviewed' ? !d.reviewed : !d.resolved);
  return /*#__PURE__*/React.createElement("section", {
    "aria-labelledby": "drift-title"
  }, /*#__PURE__*/React.createElement(PageHeader, {
    id: "drift-title",
    eyebrow: "History-aware evidence",
    title: "Configuration drift",
    description: "Meaningful state, policy, applicability, and uncertainty changes. Normal package version servicing is not classified as drift.",
    trailing: /*#__PURE__*/React.createElement(Badge, null, shown.length, " shown")
  }), /*#__PURE__*/React.createElement("div", {
    className: "app-controls"
  }, /*#__PURE__*/React.createElement(Select, {
    label: "Status",
    value: state.driftStatus,
    onChange: e => actions.setDriftStatus(e.target.value),
    options: [{
      value: 'active',
      label: 'Active'
    }, {
      value: 'unreviewed',
      label: 'Unreviewed'
    }, {
      value: 'resolved',
      label: 'Resolved'
    }, {
      value: 'all',
      label: 'All'
    }]
  }), /*#__PURE__*/React.createElement(Select, {
    label: "Component",
    options: [{
      value: 'all',
      label: 'All components'
    }].concat(CATALOGUE.map(c => ({
      value: c.id,
      label: c.name
    })))
  })), /*#__PURE__*/React.createElement("div", {
    className: "ds-list"
  }, shown.length ? shown.map(event => /*#__PURE__*/React.createElement(Disclosure, {
    key: event.id,
    summary: name(event.id),
    meta: `${event.classification} · ${event.resolved ? 'Resolved' : event.reviewed ? 'Reviewed' : 'Needs review'}`,
    trailing: `${event.confidence} confidence`,
    open: event.id === 'taskbar_widgets'
  }, /*#__PURE__*/React.createElement(StateChange, {
    steps: [{
      label: 'Previous',
      value: event.previous
    }, {
      label: 'Current',
      value: event.current
    }, {
      label: 'Desired',
      value: event.desired
    }]
  }), /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      marginTop: 'var(--space-3)'
    }
  }, /*#__PURE__*/React.createElement("strong", {
    style: {
      color: 'var(--color-text)'
    }
  }, "Likely cause:"), " ", event.cause), event.facts.map(f => /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    key: f,
    style: {
      fontSize: 'var(--type-supporting)'
    }
  }, "Evidence: ", f)), /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      fontSize: 'var(--type-supporting)'
    }
  }, "Other plausible causes: ", event.alternatives.join(', ')), /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      fontSize: 'var(--type-supporting)'
    }
  }, "Related inspections: ", event.inspections), !event.reviewed ? /*#__PURE__*/React.createElement(Button, {
    size: "sm"
  }, "Mark reviewed locally") : null)) : /*#__PURE__*/React.createElement(EmptyState, {
    large: true,
    title: "No drift events match"
  }, "Drift appears only after comparable saved inspections. Failed and cancelled detector results are not treated as proof of change.")));
}
function HistoryScreen() {
  return /*#__PURE__*/React.createElement("section", {
    "aria-labelledby": "history-title"
  }, /*#__PURE__*/React.createElement(PageHeader, {
    id: "history-title",
    eyebrow: "Persisted locally",
    title: "History",
    description: "Review Owner Mode setting changes, package removals, and saved inspections.",
    trailing: /*#__PURE__*/React.createElement(Badge, null, HISTORY.length, " saved")
  }), /*#__PURE__*/React.createElement(Surface, {
    style: {
      marginBottom: 'var(--space-4)'
    }
  }, /*#__PURE__*/React.createElement(SectionHeading, {
    eyebrow: "Owner Mode",
    title: "Changes on this account",
    description: "Preference transactions and current-user package deployment share this history.",
    trailing: /*#__PURE__*/React.createElement(Badge, {
      tone: "neutral"
    }, OWNER_HISTORY.length, " changes")
  }), /*#__PURE__*/React.createElement("div", {
    className: "app-history",
    style: {
      marginTop: 'var(--space-4)'
    }
  }, OWNER_HISTORY.map(t => /*#__PURE__*/React.createElement("article", {
    key: t.id
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("strong", null, t.subject), /*#__PURE__*/React.createElement(StatusText, null, t.status.replace(/_/g, ' '))), /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, t.kind, " \xB7 ", t.time), /*#__PURE__*/React.createElement("small", {
    className: "app-mono"
  }, t.from, " \u2192 ", t.to, " \xB7 Undo ", t.undo ? 'available' : 'not available'))))), /*#__PURE__*/React.createElement(Surface, {
    style: {
      marginBottom: 'var(--space-4)'
    }
  }, /*#__PURE__*/React.createElement(SectionHeading, {
    title: "Compare snapshots",
    description: "Choose two inspections from this machine identity.",
    trailing: /*#__PURE__*/React.createElement(Button, {
      variant: "primary"
    }, "Compare")
  }), /*#__PURE__*/React.createElement("div", {
    className: "app-controls",
    style: {
      marginTop: 'var(--space-4)'
    }
  }, /*#__PURE__*/React.createElement(Select, {
    label: "Previous",
    options: HISTORY.map(h => ({
      value: h.id,
      label: `${h.time} · build ${h.build}`
    })),
    defaultValue: HISTORY[1].id
  }), /*#__PURE__*/React.createElement(Select, {
    label: "Current",
    options: HISTORY.map(h => ({
      value: h.id,
      label: `${h.time} · build ${h.build}`
    })),
    defaultValue: HISTORY[0].id
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gap: 'var(--space-2)',
      paddingTop: 'var(--space-4)',
      marginTop: 'var(--space-4)',
      borderTop: 'var(--border-hairline)'
    }
  }, /*#__PURE__*/React.createElement("strong", null, "2 component-level change(s)"), DRIFT.map(d => /*#__PURE__*/React.createElement("div", {
    className: "app-compare",
    key: d.id
  }, /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement("strong", null, name(d.id)), /*#__PURE__*/React.createElement("small", null, "Observed \u2192 ", d.current === 'Disabled by policy' ? 'Externally managed' : 'Observed')), /*#__PURE__*/React.createElement("span", null, d.previous, " \u2192 ", d.current), /*#__PURE__*/React.createElement("p", {
    className: "ds-p"
  }, d.cause))))), /*#__PURE__*/React.createElement("div", {
    className: "ds-list"
  }, HISTORY.map(item => /*#__PURE__*/React.createElement(Surface, {
    as: "article",
    key: item.id
  }, /*#__PURE__*/React.createElement("div", {
    className: "ds-sectionheading"
  }, /*#__PURE__*/React.createElement("strong", null, item.time), /*#__PURE__*/React.createElement(StatusText, null, item.status.replace(/_/g, ' '))), /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      marginTop: 'var(--space-2)'
    }
  }, item.edition, " \xB7 build ", item.build, " \xB7 ", item.ms, " ms"), /*#__PURE__*/React.createElement("div", {
    className: "ds-chiprow",
    style: {
      margin: 'var(--space-3) 0'
    }
  }, /*#__PURE__*/React.createElement(CountChip, null, item.successful, " successful"), /*#__PURE__*/React.createElement(CountChip, null, item.unknown, " unknown"), /*#__PURE__*/React.createElement(CountChip, null, item.failed, " failed"), /*#__PURE__*/React.createElement(CountChip, null, item.cancelled, " cancelled"), /*#__PURE__*/React.createElement(CountChip, null, item.drift, " drift")), /*#__PURE__*/React.createElement("small", {
    style: {
      color: 'var(--color-text-secondary)'
    }
  }, "Trigger: user-requested local inspection \xB7 ", item.management)))));
}
function SettingsScreen({
  actions
}) {
  return /*#__PURE__*/React.createElement("section", {
    "aria-labelledby": "settings-title"
  }, /*#__PURE__*/React.createElement(PageHeader, {
    id: "settings-title",
    eyebrow: "Local product controls",
    title: "Settings & About",
    description: "Manage Deslopper's own history and diagnostics. These actions do not alter Windows configuration.",
    trailing: /*#__PURE__*/React.createElement(Badge, {
      tone: "neutral"
    }, "v", PRODUCT.version)
  }), /*#__PURE__*/React.createElement("div", {
    className: "app-two-col"
  }, /*#__PURE__*/React.createElement(Surface, null, /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, "About Deslopper"), /*#__PURE__*/React.createElement(FactGrid, {
    facts: [{
      term: 'Product',
      value: PRODUCT.productName
    }, {
      term: 'Release',
      value: PRODUCT.releaseLabel
    }, {
      term: 'Version',
      value: PRODUCT.version
    }, {
      term: 'Build mode',
      value: PRODUCT.buildMode
    }, {
      term: 'Mutation availability',
      value: PRODUCT.mutationAvailability
    }, {
      term: 'Database schema',
      value: `v${PRODUCT.schema}`
    }, {
      term: 'Database location',
      value: PRODUCT.databaseLocation
    }, {
      term: 'Windows support',
      value: PRODUCT.supportedWindows
    }]
  }), /*#__PURE__*/React.createElement(Callout, {
    note: "Documentation is included with the repository. No network access is required to understand this build."
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gap: 'var(--space-4)',
      alignContent: 'start'
    }
  }, /*#__PURE__*/React.createElement(Surface, null, /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, "Inspection history"), /*#__PURE__*/React.createElement(Select, {
    label: "Retention",
    defaultValue: "180",
    options: [{
      value: '30',
      label: '30 days'
    }, {
      value: '90',
      label: '90 days'
    }, {
      value: '180',
      label: '180 days'
    }, {
      value: '365',
      label: '365 days'
    }, {
      value: '0',
      label: 'Keep until I clear it'
    }]
  }), /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      margin: 'var(--space-3) 0'
    }
  }, "Retention applies only to Deslopper's local read-only history. Pruning always preserves the latest snapshot."), /*#__PURE__*/React.createElement(Button, {
    variant: "danger"
  }, "Clear local history")), /*#__PURE__*/React.createElement(Surface, null, /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, "Privacy-safe diagnostics"), /*#__PURE__*/React.createElement("p", {
    className: "ds-p",
    style: {
      marginBottom: 'var(--space-3)'
    }
  }, "Create a user-controlled JSON file for a bug report. Deslopper never uploads it or chooses a destination."), /*#__PURE__*/React.createElement(Button, {
    variant: "primary",
    onClick: actions.openDiagnostics
  }, "Review export contents")), /*#__PURE__*/React.createElement(Surface, null, /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, "Privacy model"), /*#__PURE__*/React.createElement("ul", {
    className: "app-bullets"
  }, /*#__PURE__*/React.createElement("li", null, "Inspection data stays local by default."), /*#__PURE__*/React.createElement("li", null, "Profile paths, account identities, sync-root names, and raw command output are excluded or redacted."), /*#__PURE__*/React.createElement("li", null, "Machine identity and the development-host denylist never enter diagnostics."), /*#__PURE__*/React.createElement("li", null, "Owner Mode has no filesystem, shell, network, or updater permission."))))));
}
Object.assign(window, {
  DesiredScreen,
  DriftScreen,
  HistoryScreen,
  SettingsScreen
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/deslopper-app/Screens.jsx", error: String((e && e.message) || e) }); }

// ui_kits/deslopper-app/Shell.jsx
try { (() => {
const {
  Surface,
  PageHeader,
  Hero,
  MetricCard,
  FactGrid,
  EmptyState,
  SectionHeading
} = window.DeslopperDesignSystem_c828b3;
const {
  Button,
  Badge,
  Chip,
  Glyph,
  Eyebrow,
  CountChip
} = window.DeslopperDesignSystem_c828b3;
const {
  Callout,
  ProgressPanel,
  SafetyNote,
  StateChange,
  StatusText,
  Seal
} = window.DeslopperDesignSystem_c828b3;
const {
  NavItem,
  ListRow,
  Disclosure,
  Dialog
} = window.DeslopperDesignSystem_c828b3;
const {
  Field,
  Select,
  Checkbox,
  SearchInput
} = window.DeslopperDesignSystem_c828b3;
const SECTIONS = [['overview', 'O', 'Overview', 'Current system summary'], ['components', 'C', 'Components', '21 observed surfaces'], ['desired', 'P', 'Desired states', 'Preview-only preferences'], ['drift', 'D', 'Drift', 'Changes over time'], ['history', 'H', 'History', 'Saved inspections'], ['settings', 'S', 'Settings & About', 'Privacy and local data']];
function Sidebar({
  section,
  onSection,
  theme,
  onTheme
}) {
  return /*#__PURE__*/React.createElement("aside", {
    className: "app-sidebar",
    "aria-label": "Primary navigation"
  }, /*#__PURE__*/React.createElement("div", {
    className: "app-brand"
  }, /*#__PURE__*/React.createElement(Glyph, {
    tone: "brand"
  }, "D"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("strong", null, "Deslopper"), /*#__PURE__*/React.createElement("span", null, PRODUCT.releaseLabel))), /*#__PURE__*/React.createElement("nav", {
    className: "ds-nav"
  }, SECTIONS.map(([id, glyph, label, description]) => /*#__PURE__*/React.createElement(NavItem, {
    key: id,
    glyph: glyph,
    label: label,
    description: description,
    active: section === id,
    onClick: () => onSection(id)
  }))), /*#__PURE__*/React.createElement("div", {
    style: {
      flex: 1
    }
  }), /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "ds-btn ds-btn--secondary ds-btn--sm ds-btn--block",
    onClick: onTheme
  }, theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'), /*#__PURE__*/React.createElement(SafetyNote, {
    title: "Owner mode"
  }, "Current-user changes are explicit, verified, recorded locally, and undoable when safe."));
}
function OverviewScreen({
  state,
  actions
}) {
  const {
    snapshot,
    progress,
    running
  } = state;
  const drift = DRIFT.filter(d => !d.resolved).length;
  return /*#__PURE__*/React.createElement("section", {
    "aria-labelledby": "overview-title"
  }, /*#__PURE__*/React.createElement(Hero, {
    id: "overview-title",
    eyebrow: "Local Windows configuration",
    title: "A clear view of what Windows is doing.",
    statement: snapshot ? `I found ${drift} settings that changed since an earlier inspection.` : 'I have not inspected this PC yet.',
    actions: /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Button, {
      variant: "primary",
      size: "lg",
      disabled: running,
      onClick: actions.runInspection
    }, snapshot ? 'Run a fresh inspection' : 'Start first inspection'), running ? /*#__PURE__*/React.createElement(Button, {
      variant: "secondary",
      size: "lg",
      onClick: actions.cancelInspection
    }, "Cancel safely") : null),
    trailing: /*#__PURE__*/React.createElement(Seal, {
      title: "Owner Mode M4",
      note: "Verified current-user settings and exact AppX package removal. No UAC."
    })
  }), progress ? /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: 'var(--space-4)'
    }
  }, /*#__PURE__*/React.createElement(ProgressPanel, {
    status: progress.status,
    completed: progress.completed,
    total: progress.total,
    detail: `${progress.detail} · ${progress.warnings} warnings · ${progress.errors} failures`,
    note: progress.note
  })) : null, /*#__PURE__*/React.createElement("div", {
    className: "app-metrics"
  }, /*#__PURE__*/React.createElement(MetricCard, {
    label: "Last inspection",
    value: snapshot ? '09:41' : 'Not yet',
    note: snapshot ? 'Fresh' : 'No inspection yet'
  }), /*#__PURE__*/React.createElement(MetricCard, {
    label: "Detected drift",
    value: snapshot ? drift : 0,
    note: "Unresolved configuration changes",
    accent: Boolean(snapshot && drift)
  }), /*#__PURE__*/React.createElement(MetricCard, {
    label: "Managed externally",
    value: snapshot ? 1 : 0,
    note: "Policy remains authoritative"
  }), /*#__PURE__*/React.createElement(MetricCard, {
    label: "Unknown or limited",
    value: snapshot ? 8 : 0,
    note: "Not interpreted as absence"
  }), /*#__PURE__*/React.createElement(MetricCard, {
    label: "Failed detectors",
    value: 0,
    note: "Retryable, scoped failures"
  })), /*#__PURE__*/React.createElement("div", {
    className: "app-two-col"
  }, /*#__PURE__*/React.createElement(Surface, null, /*#__PURE__*/React.createElement(SectionHeading, {
    eyebrow: "Current evidence",
    title: "Configuration at a glance",
    trailing: /*#__PURE__*/React.createElement(Button, {
      variant: "text",
      onClick: () => actions.goto('components')
    }, "Browse all 21")
  }), snapshot ? /*#__PURE__*/React.createElement("div", {
    className: "ds-list",
    style: {
      marginTop: 'var(--space-4)'
    }
  }, CATALOGUE.slice(0, 6).map(c => /*#__PURE__*/React.createElement(ListRow, {
    key: c.id,
    compact: true,
    title: c.name,
    subtitle: c.category,
    status: /*#__PURE__*/React.createElement(StatusText, null, c.status),
    onClick: () => actions.openComponent(c.id)
  }))) : /*#__PURE__*/React.createElement(EmptyState, {
    title: "No inspection yet"
  }, "Start a read-only inspection to populate all 21 component views. You can cancel safely while supported.")), /*#__PURE__*/React.createElement(Surface, null, /*#__PURE__*/React.createElement(Eyebrow, null, "What I can do"), /*#__PURE__*/React.createElement("h2", {
    className: "ds-h2"
  }, "Observe first, explain clearly"), /*#__PURE__*/React.createElement("ul", {
    className: "app-bullets"
  }, /*#__PURE__*/React.createElement("li", null, "Distinguish your choices from Windows servicing and administrator policy."), /*#__PURE__*/React.createElement("li", null, "Keep snapshots and meaningful drift history locally."), /*#__PURE__*/React.createElement("li", null, "Apply and exactly undo six registered current-user settings."), /*#__PURE__*/React.createElement("li", null, "Export a privacy-reviewed diagnostic file only when you ask.")), /*#__PURE__*/React.createElement(Callout, {
    title: "Restore is offered only when captured package state proves it safe.",
    note: "Preference previews remain non-executable; package removal requires one explicit click."
  }))));
}
Object.assign(window, {
  Sidebar,
  OverviewScreen,
  SECTIONS
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/deslopper-app/Shell.jsx", error: String((e && e.message) || e) }); }

// ui_kits/deslopper-app/data.js
try { (() => {
// Deslopper catalogue + synthetic observation data.
// Component definitions are the real v1 catalogue from src/platform.rs (21 components).
// Observations/drift/history are fabricated but shaped like the Rust model — no real machine data.

const CATALOGUE = [['onedrive', 'OneDrive', 'Cloud integration', 'Syncs files and integrates with Explorer.', 'Installed', 'Permission limited', 'UserPreference', 'inferred'], ['microsoft_365_copilot', 'Microsoft 365 Copilot app', 'Application', 'Provides the Microsoft 365 Copilot application.', 'Current user: present; provisioned: unknown', 'Permission limited', 'Servicing', 'moderate'], ['phone_link', 'Phone Link', 'Application', 'Connects a phone to Windows.', 'Current user: present; provisioned: present', 'Observed', 'Servicing', 'moderate'], ['consumer_copilot', 'Microsoft Copilot app', 'Assistant', 'Provides the consumer Copilot application.', 'Current user: present; provisioned: present', 'Observed', 'Servicing', 'moderate'], ['widgets_platform', 'Widgets platform', 'Shell experience', 'Provides the Windows Widgets surface.', 'Not configured', 'Unknown', 'Policy', 'inferred'], ['consumer_experiences', 'Microsoft consumer experiences', 'Recommendations', 'Controls documented consumer suggestions.', 'Unsupported', 'Not applicable', 'Policy', 'inferred'], ['welcome_experience', 'Welcome / finish setting up', 'Recommendations', 'Controls setup suggestions after sign-in.', 'Enabled', 'Observed', 'UserPreference', 'user confirmed'], ['tips_suggestions', 'Tips and suggestions', 'Recommendations', 'Controls Windows tips and suggestions.', 'Disabled', 'Observed', 'UserPreference', 'user confirmed'], ['lock_screen_suggestions', 'Lock-screen suggestions', 'Personalization', 'Controls Spotlight suggestions on the lock screen.', 'Unknown', 'Unknown', 'UserPreference', 'inferred'], ['start_recommendations', 'Start recommendations', 'Shell experience', 'Controls recommendation surfaces in Start.', 'Not configured', 'Unknown', 'Policy', 'inferred'], ['notification_suggestions', 'Notification suggestions', 'Recommendations', 'Controls suggested notifications.', 'Disabled', 'Observed', 'UserPreference', 'user confirmed'], ['settings_suggested_content', 'Suggested content in Settings', 'Recommendations', 'Controls suggested content in Settings.', 'Enabled', 'Observed', 'UserPreference', 'user confirmed'], ['search_web_results', 'Web/Bing results in Search', 'Search', 'Controls web results in Windows Search.', 'Disabled by policy', 'Externally managed', 'LocalPolicy', 'inferred'], ['search_highlights', 'Search highlights', 'Search', 'Controls Search highlights.', 'Enabled', 'Observed', 'UserPreference', 'user confirmed'], ['taskbar_widgets', 'Taskbar Widgets button', 'Taskbar', 'Controls the Widgets taskbar entry point.', 'Enabled', 'Observed', 'UserPreference', 'user confirmed'], ['taskbar_task_view', 'Taskbar Task View button', 'Taskbar', 'Controls the Task View taskbar entry point.', 'Disabled', 'Observed', 'UserPreference', 'user confirmed'], ['taskbar_search', 'Taskbar Search button/box', 'Taskbar', 'Controls taskbar Search presentation.', 'Enabled', 'Observed', 'UserPreference', 'user confirmed'], ['personal_teams_chat', 'Personal Teams / legacy Chat', 'Application', 'Controls the personal Teams/Chat entry point.', 'Current user: absent; provisioned: unknown', 'Permission limited', 'Servicing', 'moderate'], ['clipchamp', 'Clipchamp', 'Application', 'Provides the Clipchamp application.', 'Current user: present; provisioned: present', 'Observed', 'Servicing', 'moderate'], ['news_weather', 'News and Weather apps', 'Application', 'Provides news and weather applications.', 'Current user: present; provisioned: permission limited', 'Permission limited', 'Servicing', 'moderate'], ['solitaire', 'Microsoft Solitaire Collection', 'Application', 'Provides the Solitaire application.', 'Current user: present; provisioned: present', 'Observed', 'Servicing', 'moderate']].map(([id, name, category, purpose, state, status, authority, confidence]) => ({
  id,
  name,
  category,
  purpose,
  state,
  status,
  authority,
  confidence,
  isPackage: ['microsoft_365_copilot', 'phone_link', 'consumer_copilot', 'personal_teams_chat', 'clipchamp', 'news_weather', 'solitaire'].includes(id),
  detectedAt: '2026-08-16 09:41:22',
  restart: 'none',
  benefit: 'User-reviewed control over a documented Windows surface.',
  consider: 'Review gaming and shell dependencies before changing. Do not change when domain or MDM authority is detected.',
  documentation: 'https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience'
}));
const OWNER_OPERATIONS = {
  taskbar_widgets: {
    label: 'Widgets',
    verbs: ['Hide', 'Show'],
    status: 'ready',
    scope: 'current user',
    reason: 'Fixed current-user DWORD is readable and writable in this account.',
    representation: 'DWORD 1'
  },
  taskbar_task_view: {
    label: 'Task View',
    verbs: ['Hide', 'Show'],
    status: 'ready',
    scope: 'current user',
    reason: 'Physically validated PASS on Windows 11 x64 without UAC.',
    representation: 'DWORD 0'
  },
  welcome_experience: {
    label: 'Windows welcome experience',
    verbs: ['Turn off', 'Turn on'],
    status: 'ready',
    scope: 'current user',
    reason: 'ContentDeliveryManager value is present and binary.',
    representation: 'DWORD 1'
  },
  tips_suggestions: {
    label: 'Tips and suggestions',
    verbs: ['Turn off', 'Turn on'],
    status: 'ready',
    scope: 'current user',
    reason: 'SoftLandingEnabled is present and binary.',
    representation: 'DWORD 0'
  },
  notification_suggestions: {
    label: 'Notification suggestions',
    verbs: ['Turn off', 'Turn on'],
    status: 'ready',
    scope: 'current user',
    reason: 'SubscribedContent-338389Enabled is present and binary.',
    representation: 'DWORD 0'
  },
  settings_suggested_content: {
    label: 'Suggested content in Settings',
    verbs: ['Turn off', 'Turn on'],
    status: 'managed',
    scope: 'current user',
    reason: 'Managed by Windows policy. Deslopper will not override it.',
    representation: 'DWORD 1'
  }
};
const PACKAGE_OPERATIONS = {
  consumer_copilot: {
    identity: 'Microsoft.Copilot',
    restore: 'restore available'
  },
  phone_link: {
    identity: 'Microsoft.YourPhone',
    restore: 'reinstall required'
  },
  clipchamp: {
    identity: 'Clipchamp.Clipchamp',
    restore: 'restore available'
  },
  solitaire: {
    identity: 'Microsoft.MicrosoftSolitaireCollection',
    restore: 'reinstall required'
  }
};
const DRIFT = [{
  id: 'taskbar_widgets',
  classification: 'Preference',
  confidence: 'moderate',
  reviewed: false,
  resolved: false,
  previous: 'Disabled',
  current: 'Enabled',
  desired: 'Disabled',
  cause: 'A user or shell action re-enabled the taskbar entry point.',
  facts: ['Current-user Explorer preference TaskbarDa changed from DWORD 0 to DWORD 1.', 'No policy value was present in either inspection.'],
  alternatives: ['Feature update reset', 'Profile roaming'],
  inspections: 'insp-0219 → insp-0221'
}, {
  id: 'search_web_results',
  classification: 'ManagementAuthority',
  confidence: 'strong',
  reviewed: true,
  resolved: false,
  previous: 'Enabled',
  current: 'Disabled by policy',
  desired: 'Not set',
  cause: 'A local policy source became authoritative for Windows Search web results.',
  facts: ['Effective policy evidence appeared without a matching per-setting resultant-policy record.'],
  alternatives: ['MDM assignment'],
  inspections: 'insp-0219 → insp-0221'
}];
const HISTORY = [{
  id: 'insp-0221',
  time: '2026-08-16 09:41:22',
  status: 'completed_with_partial_failures',
  edition: 'Windows 11 Pro',
  build: '26100.4770',
  ms: 8420,
  successful: 20,
  unknown: 4,
  failed: 0,
  cancelled: 1,
  drift: 2,
  management: 'No domain or MDM authority detected'
}, {
  id: 'insp-0219',
  time: '2026-08-14 21:07:03',
  status: 'completed',
  edition: 'Windows 11 Pro',
  build: '26100.4770',
  ms: 7810,
  successful: 21,
  unknown: 3,
  failed: 0,
  cancelled: 0,
  drift: 0,
  management: 'No domain or MDM authority detected'
}, {
  id: 'insp-0214',
  time: '2026-08-09 08:15:44',
  status: 'cancelled',
  edition: 'Windows 11 Pro',
  build: '26100.4652',
  ms: 2110,
  successful: 9,
  unknown: 1,
  failed: 0,
  cancelled: 11,
  drift: 0,
  management: 'No domain or MDM authority detected'
}];
const OWNER_HISTORY = [{
  id: 'txn-7731',
  subject: 'Taskbar Task View button',
  kind: 'Current-user setting',
  status: 'rollback_available',
  time: '2026-08-16 09:52:10',
  from: 'DWORD 0',
  to: 'DWORD 1',
  undo: true
}, {
  id: 'txn-7702',
  subject: 'Tips and suggestions',
  kind: 'Current-user setting',
  status: 'verified',
  time: '2026-08-15 18:22:41',
  from: 'DWORD 1',
  to: 'DWORD 0',
  undo: true
}, {
  id: 'txn-7688',
  subject: 'Taskbar Widgets button',
  kind: 'Current-user setting',
  status: 'rejected_unchanged',
  time: '2026-08-15 18:19:02',
  from: 'DWORD 1',
  to: 'DWORD 1',
  undo: false
}, {
  id: 'pkg-0412',
  subject: 'Clipchamp',
  kind: 'Removed from this account',
  status: 'removed',
  time: '2026-08-15 18:31:55',
  from: 'Clipchamp.Clipchamp 3.1.20',
  to: 'absent for current user',
  undo: false,
  restore: 'restore available'
}];
const PRINCIPLES = ['Deslopper inspects before it proposes anything.', 'Owner Mode changes only registered current-user settings after an explicit click and never silently alters Windows.', 'Unknown does not mean broken, and permission-limited does not mean absent.', 'Inspection history stays local by default and sensitive evidence is redacted.'];
const FILTERS = [['all', 'All'], ['changed', 'Changed'], ['desired_differs', 'Desired differs'], ['managed', 'Managed externally'], ['permission_limited', 'Permission limited'], ['unknown', 'Unknown'], ['failed', 'Failed'], ['package', 'Package-backed'], ['policy', 'Policy-backed'], ['preference', 'User preference']];
const PRODUCT = {
  productName: 'Deslopper',
  releaseLabel: 'Owner Mode M4',
  version: '0.4.1',
  buildMode: 'normal owner mode',
  mutationAvailability: 'six fixed current-user settings, four exact package identities',
  schema: '9',
  databaseLocation: '%LOCALAPPDATA%\\Deslopper\\deslopper.db (redacted)',
  supportedWindows: 'Windows 10 2004+ and Windows 11 (x64)'
};
Object.assign(window, {
  CATALOGUE,
  OWNER_OPERATIONS,
  PACKAGE_OPERATIONS,
  DRIFT,
  HISTORY,
  OWNER_HISTORY,
  PRINCIPLES,
  FILTERS,
  PRODUCT
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/deslopper-app/data.js", error: String((e && e.message) || e) }); }

__ds_ns.Badge = __ds_scope.Badge;

__ds_ns.Button = __ds_scope.Button;

__ds_ns.Chip = __ds_scope.Chip;

__ds_ns.CountChip = __ds_scope.CountChip;

__ds_ns.Eyebrow = __ds_scope.Eyebrow;

__ds_ns.Glyph = __ds_scope.Glyph;

__ds_ns.Callout = __ds_scope.Callout;

__ds_ns.ProgressPanel = __ds_scope.ProgressPanel;

__ds_ns.SafetyNote = __ds_scope.SafetyNote;

__ds_ns.Seal = __ds_scope.Seal;

__ds_ns.StateChange = __ds_scope.StateChange;

__ds_ns.StatusText = __ds_scope.StatusText;

__ds_ns.Checkbox = __ds_scope.Checkbox;

__ds_ns.Field = __ds_scope.Field;

__ds_ns.SearchInput = __ds_scope.SearchInput;

__ds_ns.Select = __ds_scope.Select;

__ds_ns.EmptyState = __ds_scope.EmptyState;

__ds_ns.FactGrid = __ds_scope.FactGrid;

__ds_ns.Hero = __ds_scope.Hero;

__ds_ns.MetricCard = __ds_scope.MetricCard;

__ds_ns.PageHeader = __ds_scope.PageHeader;

__ds_ns.SectionHeading = __ds_scope.SectionHeading;

__ds_ns.Surface = __ds_scope.Surface;

__ds_ns.Dialog = __ds_scope.Dialog;

__ds_ns.Disclosure = __ds_scope.Disclosure;

__ds_ns.ListRow = __ds_scope.ListRow;

__ds_ns.NavItem = __ds_scope.NavItem;

})();
