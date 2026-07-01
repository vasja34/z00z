# Markdown-it Plugin Demos

This document contains demo content for various \`markdown-it\` plugins to help you verify their functionality.

---

## 1. \`@mdit/plugin-abbr\` (Abbreviations)

This plugin allows you to define abbreviations.

::: demo

*[HTML]: Hyper Text Markup Language
*[W3C]: World Wide Web Consortium

The HTML specification is maintained by the W3C.

:::

---

## 2. \`@mdit/plugin-alert\` (Alert Blocks)

This plugin provides special blockquotes for alerts/notes/warnings.

::: demo

> [!note]
> This is note text, This is ==Highlight text==

> [!important]
> This is important text, This is ==Highlight text==

> [!tip]
> This is tip text, This is ==Highlight text==

> [!warning]
> This is warning text, This is ==Highlight text==

> [!caution]
> This is caution text, This is ==Highlight text==

:::

---

## 3. \`@mdit/plugin-align\` (Text Alignment)

This plugin allows you to align text.

::: demo

:::: center

### Twinkle, Twinkle, Little Star

:::: right

——Jane Taylor

:::: center

Twinkle, twinkle, little star,

How I wonder what you are!

Up above the world so high,

Like a diamond in the sky.

When the blazing sun is gone,

When he nothing shines upon,

Then you show your little light,

Twinkle, twinkle, all the night.

Then the traveller in the dark,

Thanks you for your tiny spark,

He could not see which way to go,

If you did not twinkle so.

In the dark blue sky you keep,

And often thro' my curtains peep,

For you never shut your eye,

Till the sun is in the sky.

'Tis your bright and tiny spark,

Lights the trav’ller in the dark,

Tho' I know not what you are,

Twinkle, twinkle, little star.

::::

---

## 4. \`@mdit/plugin-attrs\` (Custom Attributes)

This plugin allows adding custom attributes to block and inline elements.

::: demo

Text with `inline code`{.inline-code} and ![favicon =50x50](/favicon.ico){.image}, also supporting _emphasis_{.inline-emphasis} and **bold**{.inline-bold}.

:::

---

::: demo

block content {.block}

:::

---

::: demo

```js {.fence}
const a = 1;
```

:::

---

::: demo

| A                        | B   | C   | D              |
| ------------------------ | --- | --- | -------------- |
| A1                       | B1  | C1  | D1 {rowspan=3} |
| A2 {colspan=2 rowspan=2} | B2  | C2  | D2             |
| A3                       | B3  | C3  | D3             |

{.table border=1}

:::

---

::: demo

- list item{.list-item}
  - nested list item
    {.nested}

{.list-wrapper}

:::

---

::: demo

--- {.horizontal}

:::

---

::: demo

A line with break  
{.break}

:::

---

## 5. \`@mdit/plugin-container\` (Custom Containers)

This plugin allows defining custom block containers.

::: demo

::: warning

Warning Text

:::

---

## 6. \`@mdit/plugin-demo\` (Demo Blocks)

This plugin typically renders code blocks with a live preview. The exact rendering depends on your setup.

::: demo

**Bold Text**

Text

:::

---

## 7. \`@mdit/plugin-dl\` (Definition Lists)

This plugin supports definition lists.

::: demo

Term 1

: Definition 1

Term 2 with _inline markup_

: Definition 2

        { some code, part of Definition 2 }

    Third paragraph of definition 2.

Term 3

: Definition
with lazy continuation.

    Second paragraph of the definition.

---

Term 1
: Definition 1

Term 2
: Definition 2a
: Definition 2b

:::

---

## 8. \`@mdit/plugin-embed\` (Embedding Content)

This plugin allows embedding content from various sources.

::: demo

{% youtube dQw4w9WgXcQ %}

:::

---

## 9. \`@mdit/plugin-figure\` (Figures with Captions)

This plugin creates figures and figcaption elements for images.

::: demo

![Logo](https://mdit-plugins.github.io/favicon.ico)

[![Logo](https://mdit-plugins.github.io/favicon.ico)](https://commonmark.org/)

![Logo](https://mdit-plugins.github.io/favicon.ico "Markdown")

[![Logo](https://mdit-plugins.github.io/favicon.ico "Markdown")](https://commonmark.org/)

:::

---

## 10. \`@mdit/plugin-footnote\` (Footnotes)

::: demo

This plugin enables footnotes.

Footnote 1 link[^first].

Footnote 2 link[^second].

Inline footnote^[Text of inline footnote] definition.

Duplicated footnote reference.

[^first]: Footnote **can have markup**

    and multiple paragraphs[^second].

[^second]: Footnote text.

:::

---

## 11. \`@mdit/plugin-icon\` (Icons)

This plugin allows embedding icons from icon libraries (e.g., Font Awesome, Material Design Icons).

::: demo

iPhone is made by ::apple::.

:::

---

## 12. \`@mdit/plugin-img-lazyload\` (Image Lazyload)

This plugin automatically adds \`loading="lazy"\` to images. No special Markdown syntax is needed; just a 
regular image.

::: demo

![Lazy Loaded Image](https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcTFYqoKTu_o3Zns2yExbst2Co84Gpc2Q1RJbA&s)

:::

---

## 13. \`@mdit/plugin-img-mark\` (Image Mark)

This plugin allows adding marks or highlights to images. The exact syntax might vary, but a common approach uses fragments.

::: demo

![GitHub Light](https://mdit-plugins.github.io/github-light.png#dark)
![GitHub Dark](https://mdit-plugins.github.io/github-dark.png#light)

:::

---

## 14. \`@mdit/plugin-img-size\` (Image Size)

This plugin allows specifying image dimensions directly in Markdown.

::: demo

![Logo =200x200](https://mdit-plugins.github.io/logo.svg 'Markdown')
![Logo =150x](https://mdit-plugins.github.io/logo.svg 'Markdown')
![Logo =x100](https://mdit-plugins.github.io/logo.svg 'Markdown')

:::

---

## 15. \`@mdit/plugin-include\` (Include Files)

This plugin allows including content from other Markdown files.
_Note: This demo only shows the syntax. For it to work, \`another-file.md\` must exist in your content directory at the correct path._

::: demo

<!-- @include: ./content/another-file.md -->

:::

---

## 16. \`@mdit/plugin-ins\` (Inserted Text)

This plugin renders text as "inserted" (typically underlined).

::: demo

VuePress Theme Hope is ++very++ powerful.

:::

---

## 17. \`@mdit/plugin-katex\` (KaTeX Math Rendering)

This plugin renders LaTeX math using KaTeX.

::: demo

Euler’s identity $e^{i\pi}+1=0$ is a beautiful formula in $\mathbb{R}^2$.

$$
\frac {\partial^r} {\partial \omega^r} \left(\frac {y^{\omega}} {\omega}\right)
= \left(\frac {y^{\omega}} {\omega}\right) \left\{(\log y)^r + \sum_{i=1}^r \frac {(-1)^ Ir \cdots (r-i+1) (\log y)^{ri}} {\omega^i} \right\}
$$

:::

---

## 18. \`@mdit/plugin-mark\` (Marked/Highlighted Text)

This plugin renders text as "marked" (typically highlighted).

::: demo

This is some ==highlighted text==.

:::

---

## 19. \`@mdit/plugin-mathjax\` (MathJax Math Rendering)

This plugin renders LaTeX math using MathJax.

::: demo

Inline math: $a^2 + b^2 = c^2$

Block math:
$$\\sum_{i=1}^n i = \\frac{n(n+1)}{2}$$

:::

---

## 20. \`@mdit/plugin-plantuml\` (PlantUML Diagrams)

This plugin renders PlantUML diagrams.

::: demo

@startuml
Alice -> Bob: Authentication Request
Bob --> Alice: Authentication Response
Alice -> Bob: Another request
Alice <-- Bob: Another response
@enduml

:::

---

## 21. \`@mdit/plugin-ruby\` (Ruby Annotations)

This plugin adds Ruby annotations (for East Asian languages).

::: demo

{漢字|かんじ} (Kanji)

:::

---

## 22. \`@mdit/plugin-snippet\` (Code Snippets)

This plugin allows including code snippets from files.
_Note: This demo only shows the syntax. For it to work, \`my-code.js\` must exist in your content directory at the correct path._

::: demo

<<< ./content/my-code.js#region

:::

---

## 23. \`@mdit/plugin-spoiler\` (Spoiler Text)

This Plugins is used to hide content.

::: demo

VuePress Theme Hope is !!powerful!!.

:::

---

## 24. \`@mdit/plugin-stylize\` (Stylize)

This plugin allows applying custom styles or classes using a simplified syntax.

::: demo

==blue:This is blue text==

==#ff0000:This is red text==

<span class="text-violet-500">This is highlight text</span>

==bg-amber:This is highlight text==

<span class="bg-emerald-200 text-emerald-900">This is highlight text</span>

:::

---

## 25. \`@mdit/plugin-sub\` (Subscript)

This plugin enables subscript text.

::: demo

Water is H~2~O.
The chemical formula for methane is CH~4~.

:::

---

## 26. \`@mdit/plugin-sup\` (Superscript)

This plugin enables superscript text.

::: demo

E=mc^2^
The first power is X^1^, the second is X^2^.

:::

---

## 27. \`@mdit/plugin-tab\` (Tabs)

This plugin creates tabbed content blocks.

::: demo

::: tabs

@tab:active Apple
Apple

@tab Banana
Banana

:::

---

## 28. \`@mdit/plugin-tasklist\` (Task List)

This plugin renders GitHub-style task lists.

::: demo

- [x] Completed task
- [ ] Uncompleted task
- [x] Another completed task
    - [ ] Nested task

:::

---

## 29. \`@mdit/plugin-tex\` (TeX Math Rendering)

This plugin renders TeX math (similar to KaTeX/MathJax).

::: demo

Inline TeX: $x^2 + y^2 = r^2$

Block TeX:
$$\\frac{d}{dx} \\left( \\int_{a}^{x} f(t) dt \\right) = f(x)$$

:::

---

## 30. \`@mdit/plugin-uml\` (UML Diagrams)

Plugin to support splitting contents from context

::: demo

@mermaidstart
flowchart TD
  A[Start] --> B{Decision}
  B -->|Option A| C[Process A]
  B -->|Option B| D[Process B]
  C --> E[End]
  D --> E
@mermaidend

:::

---

## 31. \`@mdit/markdown-it-collapsible\` (Collapsible / Details)

Plugin to support splitting contents from context

::: demo

+++ Click me!
Hidden text
+++

To start in open state, use ++> instead:

++> Click me!
Hidden text
++>

:::

## 32. \`@kazumatu981/markdown-it-kroki\`

Plugin to support kroki.

::: demo

```plantuml[d2]

D2 Parser: {
  shape: class

  # Default visibility is + so no need to specify.
  +reader: io.RuneReader
  readerPos: d2ast.Position

  # Private field.
  -lookahead: "[]rune"

  # Protected field.
  # We have to escape the # to prevent the line from being parsed as a comment.
  \#lookaheadPos: d2ast.Position

  +peek(): (r rune, eof bool)
  rewind()
  commit()

  \#peekn(n int): (s string, eof bool)
}

"github.com/terrastruct/d2parser.git" -> D2 Parser
```

:::