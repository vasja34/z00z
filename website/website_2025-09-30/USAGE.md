# How to Set Up the Project (Next.js)

Before you can add pages or make changes to the website, you need to set up the project on your computer. Here’s how:

---

1. **Install Node.js**If not already installed, download and install it from:https://nodejs.org/
2. **Clone the Project**Open your terminal and run:

   ```bash
   git clone https://github.com/jukha/ecme-static-site-framewok.git
   cd ecme-static-site-framewok

   ```
3. **Install Dependencies**

```bash
    npm install
    # or
    yarn install
```

4. **Install Dependencies**

```bash
   npm run dev
   # or
   yarn dev
```

# How to Add New Pages to Our Website

This guide will walk you through the simple steps to add new content pages to our website and make them appear in the navigation menu. You don't need to be a developer to do this — just follow these instructions carefully!

---

## Step 1: Create Your Content Page (the `.md` file)

Our website uses special files called **Markdown** files (`.md` extension) for content pages. Think of them like simple Word documents, but without all the complex formatting buttons.

1. Go to the `content` folder in your project's main directory. This is where all your content pages live.
2. Create a **new file** inside this `content` folder.
3. Give it a **meaningful name**, for example:`my-new-article.md` or `about-us.md`
4. You can **organize** your files using subfolders, e.g.:`content/documents/about-us.md`
5. Write your content using basic formatting like:

```md
# Main Heading

## Subheading

**Bold Text**

_Italic Text_

- List Item
```

**Example: `about-us.md`**

```md
# About us

Welcome to about us page!

This is some plain text.

- Here is a list item
- Another list item
```

👉 **Remember to save your file!**

---

## Step 2: Add Your Page to the Navigation Menu

Now that you have your content file, you need to **add it to the navigation menu**.

1. Open the file: `config/navigation.config.yaml`

> ✅ **Important: The file name and folder structure must match the URL path exactly.**

For example:

- For the path `/about-us`, the file must be:`content/about-us.md`
- For `/docs/intro`, use:
  `content/docs/intro.md`

> ⚠️ **Important:** This file uses **YAML** format, which is very sensitive to **spaces**. Use **spaces**, not tabs!

### Understand the Menu Structure:

Each menu item is defined with these keys:

- `key`: Unique ID for the menu item (e.g., `aboutUsPage`)
- `path`: Web address of the page (e.g., `/about`)
- `title`: Text shown in the menu (e.g., "About Us")
- `translateKey`: For multilingual support (e.g., `nav.about`)
- `icon`: (Optional) Icon name
- `type`: `ITEM` for pages, `COLLAPSE` for dropdowns
- `authority`: Leave as `[]` (no restriction)
- `meta.description.label`: A short description
- `subMenu`: List of child menu items

---

### 🖼️ How to Add Icons to Menu Items

You can display an icon next to any menu item by setting the `icon` property in your YAML entry.

- The `icon` property should match the name of an icon exported from`src/configs/navigation-icon.config.tsx`.
- Browse or open that file to see all available icon names.
- Example icon names: `homeIcon`, `documentIcon`, `circleIcon`, etc.

**How to use:**

```yaml
- key: aboutPage
  path: /about
  title: About
  icon: homeIcon   # 👈 This will show the "home" icon
  type: ITEM
  authority: []
  meta:
      description:
          label: About our company
  subMenu: []
```

> ℹ️ **Tip:** If you want to add a new icon, add it to `src/configs/navigation-icon.config.tsx` and then reference its name in your YAML.

---

### 🎨 How to Change the Syntax Highlighting Theme

You can easily change the color theme used for code blocks in Markdown pages.

- The theme is set in the file`src/app/(protected-pages)/(docs)/[...slug]/page.tsx`
- Look for this line near the top of the file:

```tsx
import { dracula } from 'react-syntax-highlighter/dist/cjs/styles/prism'
```

- To use a different theme, change `dracula` to any other available theme from [react-syntax-highlighter Prism themes](https://github.com/react-syntax-highlighter/react-syntax-highlighter/blob/master/AVAILABLE_STYLES_PRISM.MD).

**Example:**
To use the `okaidia` theme, update the import like this:

```tsx
import { okaidia } from 'react-syntax-highlighter/dist/cjs/styles/prism'
```

- Save the file and refresh your browser to see the new code highlighting style.

---

### ✅ Example A: Add a Simple About us Page

```yaml
- key: myArticlePage
  path: /about-us
  title: About us
  translateKey: nav.aboutUs
  icon: someIcon
  type: ITEM
  authority: []
  meta:
      description:
          translateKey: nav.aboutUsDesc
          label: A great read on new topics
  subMenu: []
```

---

### ✅ Example B: Add a Page Under a Dropdown Menu

Suppose your file is: `content/documentation/examples/first.md`

1. Add a **top-level dropdown** for Documentation.
2. Add a **submenu** for Examples.
3. Add your **actual page** in the final `subMenu`.

```yaml
- key: documentationParent
  path: ''
  title: Documentation
  translateKey: nav.documentation
  icon: documentIcon
  type: COLLAPSE
  authority: []
  meta:
      description:
          translateKey: nav.documentationDesc
          label: Guides and how-tos
  subMenu:
      - key: examplesParent
        path: ''
        title: Examples
        translateKey: nav.examples
        icon: exampleIcon
        type: COLLAPSE
        authority: []
        meta:
            description:
                translateKey: nav.examplesDesc
                label: Practical usage examples
        subMenu:
            - key: firstExamplePage
              path: /documentation/examples/first
              title: First Example
              translateKey: nav.firstExample
              icon: circleIcon
              type: ITEM
              authority: []
              meta:
                  description:
                      translateKey: nav.firstExampleDesc
                      label: A basic working example
              subMenu: []
```

---

## Step 3: See Your Changes!

Once you've saved both:

- Your `.md` file in the `content/` folder
- And updated `navigation.config.yaml`

Restart your local development server:

```bash
# Stop the current process
Ctrl + C

# Restart the server
npm run dev
# or
yarn dev
```

Open your browser and go to:
**http://localhost:3000**

🎉 Your new page and menu item should now be visible!

---

## 🧠 Quick Tips

- ✅ **Use spaces**, never tabs in YAML
- ✅ **Every key must be unique**
- ✅ `path` is the URL, `title` is what users see
- ✅ Use `ITEM` for direct pages, `COLLAPSE` for dropdowns

---

## How to Guide for Customizing Fonts

1\. To add or change the font, follow these steps:

- Open the file at `src/constants/font-constant.ts`.
- Locate the `FONT_TYPE_OPTIONS`, `FONT_WEIGHT_OPTIONS` and `FONT_FAMILY_OPTIONS` arrays.
- Modify the values of the arrays to your desired font types and weights.
- Save the file.
- Refresh your application to see the changes.

Note: If you want to add a new font, you can add a new object to the `FONT_FAMILY_OPTIONS` arrays. Make sure to provide a unique `id` and `value` for each new font.

Example:

```typescript
FONT_FAMILY_OPTIONS = [
  ...,
  {
    id: 7,
    label: 'Raleway',
    value: 'Raleway',
  },
];
```

---

## 🛠️ How to Guide: Customizing Theme Colors in Tailwind

To change the background and foreground colors in your project, you can modify the CSS variables defined in the `@layer theme` section of your Tailwind CSS file located at:

```
/src/assets/styles/tailwind/index.css
```

### ✅ Steps to Customize or Add a New Theme

---

### 1. **Open the Theme CSS File**

Navigate to the file:

```
/src/assets/styles/tailwind/index.css
```

This file contains your theme definitions using Tailwind's `@layer theme` directive.

---

### 2. **Locate the `:root` Theme Selector**

Inside the `@layer theme` block, find the `:root` selector. It contains global CSS variables such as:

```css
:root {
  --background: #ffffff;
  --heading-1: #000000;
  --paragraph: #333333;
  /* ... other variables */
}
```

---

### 3. **Identify Key Color Variables**

Common variables for background and text include:

* `--background`
* `--heading-1` through `--heading-6`
* `--paragraph`

You can update these with your desired color codes (e.g., HEX, RGB, or HSL).

---

### 4. **Create a New Theme**

To define a new theme:

1. Duplicate the `:root` block.
2. Rename the selector to match your new theme, e.g.:

```css
.daisy-dark-2 {
  --background: #1e1e1e;
  --heading-1: #ffffff;
  --paragraph: #cccccc;
  /* ...custom values */
}
```

> ✅ Tip: Make sure the class name (e.g., `.daisy-dark-2`) is unique.

---

### 5. **Register the Theme in the Constants File**

Open the theme constants file:

```
/src/constants/color-theme.constant.ts
```

Add your new theme name to the either `LIGHT_MODE_OPTIONS` or `DARK_MODE_OPTIONS` list so it can be recognized by the application, For Example:

```ts
export const DARK_MODE_OPTIONS = [
  ...,
    {
        id: 5,
        label: 'Daisy Dark 2',
        value: 'daisy-dark-2',
    },
]
```

---

### 6. **Apply and Test the New Theme**

* Save your changes to both files.
* Start or refresh your application.

---

### 💡 Example: Change Background Color to Red

Inside `.daisy-dark-2`:

```css
.daisy-dark-2 {
  --background: #ff0000;
}
```

This will change the global background to red when the `daisy-dark-2` theme is active.

---

### 📌 Notes

* Always use valid CSS color formats (HEX, RGB, HSL).
* Theme variables also include theming for components like buttons, borders, shadows, etc.
* Use descriptive names for your themes to avoid confusion.

---

## `@kazumatu981/markdown-it-kroki` Plugin — Usage Guide & Workaround

### 🚨 Known Issue

The `@kazumatu981/markdown-it-kroki` plugin currently has a bug where it uses the wrong URL to fetch diagrams. For example, when rendering a **D2** diagram, the plugin incorrectly sends a request to:

```
https://kroki.io/plantuml/svg
```

However, the correct URL should be:

```
https://kroki.io/d2/svg
```

---

### ✅ Workaround

To work around this issue, you can use the **alt text** in the fenced code block to specify the correct diagram type.

#### How It Works:

The plugin normally renders diagrams like this:

````markdown
```plantuml
Your diagram code...
```
````

To fix the incorrect diagram type, you can **override the default type (`plantuml`) using the alt text**, like so:

````markdown
```plantuml[d2]
Your D2 diagram code here...
```
````

In this case:

* The language is still set to `plantuml` (required due to how the plugin works).
* The **alt text** `[d2]` is used to specify the correct Kroki diagram type.

This causes the plugin to generate a request to the correct URL:

```
https://kroki.io/d2/svg
```

---

### 🔍 How to Find the Correct Diagram Type

To find the correct diagram type keyword (like `d2`, `mermaid`, `seqdiag`, etc.):

1. Go to [Kroki&#39;s Try page](https://kroki.io/#try)
2. Select the diagram type you want to use.
3. Observe the URL format:
   It will look like `https://kroki.io/<diagram-type>/svg`
   The `<diagram-type>` part is the **keyword** you need to use in the alt text.

---

### 🧪 Example

To render a **D2** diagram:

````markdown
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
````

---

## How to run the `kroki` Plugin Locally with Docker:

To run the `kroki` plugin locally with Docker, follow these steps:

- Download and install Docker on your local machine.
- Open a terminal and run the following command.
- `docker run -p 8000:8000 yuzutech/kroki`.
- This will start the `kroki` server locally on port 8000 (You can change the port if needed like `8001:8000`).
- Add the following line to your `.env` file with the correct server URL and port:
- `NEXT_PUBLIC_KROKI_SERVER_URL='http://localhost:8000'`
- If you want to use the `kroki` plugin in production you must instead add the following line to your `.env` file:
- `NEXT_PUBLIC_KROKI_SERVER_URL='https://kroki.io'`

## How to run the `kroki` Plugin Locally with Docker Compose:

To run the `kroki` plugin locally with Docker Compose, follow these steps:

- Download and install Docker on your local machine.
- Add the following line to your `.env` file with the correct server URL and port listed in the `docker-compose.yml` file:
- `NEXT_PUBLIC_KROKI_SERVER_URL='http://localhost:8000'`
- While in the project root directory where the `docker-compose.yml` file is located, run `docker compose up`.
- This will then run both your Next.js app and the `kroki` server in docker containers.
- To use extra diagram types uncomment the lines in the `docker-compose.yml` file.
