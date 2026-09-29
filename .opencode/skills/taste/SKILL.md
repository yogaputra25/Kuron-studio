---
name: taste
description: Anti-slop rules for code, UI, and user-facing text in Kuron Studio. Use before writing or editing any Svelte component, CSS, Tailwind class, UI string, or identifier. Also use when a screen looks like a generated template, when reviewing a diff for generic feel, or when naming a new module, command, or variable.
---

# Taste

Kuron Studio is a work tool, not a landing page. The effect to aim for: dense,
quiet, and obviously made by someone who knows the work.

Read the section matching what you are writing.

---

## 1. Visual

### Skip

| Pattern | Why |
|---|---|
| Purple/blue gradient | Loudest tell of generated output, and it earns nothing in a work tool |
| Emoji as icons | Renders differently per OS, cannot be coloured, never aligns with text |
| Centred hero, generous whitespace | In a full-screen desktop app, whitespace is wasted pixels |
| Three feature cards in a row | A presentation pattern, not an interface pattern |
| `rounded-2xl` plus `shadow-lg` everywhere | When everything is shaped the same, shape means nothing |
| Entrance animations, stagger, hover on everything | Pulls attention away from the work |
| Icon inside a coloured circle | A framework habit, not a need |

### Do

- **Density, not air.** Tight rows, clear gaps between groups. Space separates
  groups; it does not fill the page.
- **One shape, one height** per group of controls, not four sizes used in turn.
- **Colour carries meaning.** Green succeeded, amber needs attention, red
  failed. Not for decoration, not to tell one box from another.
- **Snap to a grid.** Side by side elements share a baseline and a gap. Gaps
  that "look right" but are not equal read as wrong once a third element lands.

Colours, spacing, radii, and heights are already decided in `src/app.css` as
`--ks-*` tokens plus the Tailwind utilities that read them. Take them from there
rather than inventing values. `theme-discipline.test.ts` catches regressions.

## 2. Code

### Skip

| Pattern | Instead |
|---|---|
| `utils.ts`, `helpers.ts`, `manager.ts` | Name it after the domain: `bubble.ts`, `batch.ts`, `i18n.ts` |
| Abstraction with one implementation | Write it inline. Abstract at the second real implementation |
| A comment that restates the line below it | Delete it |
| Function wrapping a single line | The line |
| Boolean flag with no caller yet | Add it when a caller exists |
| Config value that has never changed | Hardcode it |
| "Refactor for readability" with no concrete problem behind it | Skip the refactor |

### Do

- **Names carry the domain, not the mechanism.** `PagePrep`, `effectiveGlossary`,
  `applyCached` say what happens. `processData`, `doWork`, `handle2` do not.
- **Code that survives deleting the identifier names.** If every name already
  explains itself, the names can go.
- **Comments explain why.** When the name explains the what, write nothing.
- **One file, one responsibility, named after both.**

## 3. User-facing text

Every UI string goes through `src/lib/i18n.ts` in all three languages (id, en,
zh), never inline in a component. `i18n.test.ts` fails when a key is missing in
any one of them.

### Skip

| Text | Problem |
|---|---|
| "Submit" | Submitting what? |
| "Something went wrong. Please try again." | Names no cause, and retry is often not possible |
| "Welcome! Let's get started" | Shouting, and there is no getting started to begin |
| "The AI is taking a break" | False friendliness |
| Tooltip repeating the button label | No added information |
| "This feature is coming soon" | A promise nobody can keep |

### Do

- **Buttons name action plus object.** "Simpan Project", "Terjemahkan 12
  Halaman", "Ekspor CBZ".
- **Errors name cause and way out.** "Kunci API ditolak. Periksa ulang kunci di
  Pengaturan Provider."
- **Empty states offer the action.** An empty grid gets a "Buat Project" button
  and one sentence of explanation, not an illustration.
- **Plain and short.** No over-politeness, no cheerleading, no vagueness.

## 4. Wider principles

- **A settled decision is not reopened every time a component is copied.**
  Colour, typeface, radius, and control height are already decided. Read them
  and use them.
- **Consistency outranks creativity.** Ten elements at mismatched heights damage
  the screen more than one boring element does.
- **Usability outranks looking impressive.** Tooltips, exact labels, focus
  order, and WCAG AA contrast are all already handled in the tokens and the
  tests. They are not the thing to trade away for a better-looking screen.
- **When something reads as generic, the problem is usually density, not
  palette.** Too much air, too little information per screen. Fix the layout
  before touching the colours.

### For a deeper audit

When the need is judgement against principles rather than a blocklist:

- `claude-mem:design-is` audits a design against Dieter Rams' ten principles.
- `opendesign:frontend-design` applies when there is no design system yet and a
  committed visual direction has to be chosen over framework defaults.
