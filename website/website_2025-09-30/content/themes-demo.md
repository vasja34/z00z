# Theme Variables and Color Classes Demo

This page showcases all the theme variables and CSS color classes used in the project, extracted from `/src/assets/styles/tailwind/index.css`.

## Gray Fonts from `tailwind/index.css`

<span class="text-gray-50 text-lg bg-black dark:bg-transparent">Gray 50 Font #fafafa</span>  
<span class="text-gray-100 text-lg bg-black dark:bg-transparent">Gray 100 Font #f5f5f5</span>  
<span class="text-gray-200 text-lg bg-black dark:bg-transparent">Gray 200 Font #e5e5e5</span>  
<span class="text-gray-300 text-lg bg-black dark:bg-transparent">Gray 300 Font #d4d4d4</span>  
<span class="text-gray-400 text-lg bg-black dark:bg-transparent">Gray 400 Font #a3a3a3</span>  
<span class="text-gray-500 text-lg dark:bg-white">Gray 500 Font #737373</span>  
<span class="text-gray-600 text-lg dark:bg-white">Gray 600 Font #525252</span>  
<span class="text-gray-700 text-lg dark:bg-white">Gray 700 Font #404040</span>  
<span class="text-gray-800 text-lg dark:bg-white">Gray 800 Font #404040</span>  
<span class="text-gray-900 text-lg dark:bg-white">Gray 900 Font #171717</span>  
<span class="text-gray-950 text-lg dark:bg-white">Gray 950 Font #0a0a0a</span>  

## Theme Variables from `tailwind/index.css`

These variables define the core color palette and various UI component styles across different themes.

| Variable Name | Description |
|---------------|-------------|
| `--heading-1`  | Heading 1 color. |
# Heading 1

| Variable Name | Description |
|---------------|-------------|
| `--heading-2` | Heading 2 color. |
## Heading 2

| Variable Name | Description |
|---------------|-------------|
| `--heading-3` | Heading 3 color. |
### Heading 3

| Variable Name | Description |
|---------------|-------------|
| `--heading-4` | Heading 4 color. |
#### Heading 4

| Variable Name | Description |
|---------------|-------------|
| `--heading-5` | Heading 5 color. |
##### Heading 5

| Variable Name | Description |
|---------------|-------------|
| `--heading-6` | Heading 6 color. |
###### Heading 6

| Variable Name | Description | Example |
|---------------|-------------|---------|
| `--paragraph`  | Paragraph text color.     | This is a paragraph. |
| `--hr`         | Horizontal rule color.    | --- |
| `--small`      | Small text color.         | <small>Small text</small> |
| `--strong`     | Strong text color.        | **Strong text** |
| `--b`          | Bold text color.          | **Bold text** |
| `--em`         | Emphasized text color.    | *Emphasized text* |
| `--i`          | Italic text color.        | *Italic text* |
| `--u`          | Underlined text color.    | <u>Underlined text</u> |
| `--mark`       | Marked/highlighted text.  | <mark>Marked text</mark> |
| `--del`        | Deleted text color.       | ~~Deleted text~~ |
| `--ins`        | Inserted text color.      | <ins>Inserted text</ins> |
| `--sub`        | Subscript text color.     | <sub>Subscript text</sub> |
| `--sup`        | Superscript text color.   | <sup>Superscript text</sup> |
| `--background` | Main background color. |  |
| `--header-background` | Header background color. |  |
| `--header-hover-background` | Header hover background color. |  |
| `--header-hover-text` | Header hover text color. |  |
| `--side-nav-background` | Side navigation background color. |  |
| `--side-nav-border` | Side navigation border color. |  |
| `--stacked-side-nav-mini-background` | Stacked side navigation mini background color. |  |
| `--stacked-side-nav-mini-border` | Stacked side navigation mini border color. |  |
| `--stacked-side-nav-secondary-background` | Stacked side navigation secondary background color. |  |
| `--stacked-side-nav-secondary-border` | Stacked side navigation secondary border color. |  |
| `--menu-item-text` | Menu item text color. |  |
| `--menu-item-hover-text` | Menu item hover text color. |  |
| `--menu-item-hover-background` | Menu item hover background color. |  |
| `--menu-item-border` | Menu item border color. |  |
| `--menu-collapse-item-text` | Menu collapse item text color. |  |
| `--menu-collapse-item-hover-text` | Menu collapse item hover text color. |  |
| `--menu-collapse-item-hover-background` | Menu collapse item hover background color. |  |
| `--drawer-background` | Drawer background color. |  |
| `--drawer-border` | Drawer border color. |  |
| `--drawer-overlay` | Drawer overlay color. |  |
| `--search-dialog-border` | Search dialog border color. |  |
| `--search-dialog-input-text` | Search dialog input text color. |  |
| `--search-item-hover-background` | Search item hover background color. |  |
| `--search-item-text` | Search item text color. |  |
| `--toc-title-text` | Table of content title text color. |  |
| `--toc-background` | Table of content background color. |  |
| `--toc-border` | Table of content border color. |  |
| `--toc-title-text-dark` | Table of content title text color for dark mode (deprecated, use `--toc-title-text` in `.dark` block). |  |
| `--toc-background-dark` | Table of content background color for dark mode (deprecated, use `--toc-background` in `.dark` block). |  |
| `--toc-border-dark` | Table of content border color for dark mode (deprecated, use `--toc-border` in `.dark` block). |  |
| `--button-default-background` | Default button background color. |  |
| `--button-default-border` | Default button border color. |  |
| `--button-default-text` | Default button text color. |  |
| `--button-default-active-border` | Default button active border color. |  |
| `--avatar-background` | Avatar background color. |  |
| `--avatar-text` | Avatar text color. |  |
| `--card-background` | Card background color. |  |
| `--card-border` | Card border color. |  |
| `--dialog-background` | Dialog background color. |  |
| `--dialog-overlay` | Dialog overlay color. |  |
| `--dropdown-background` | Dropdown background color. |  |
| `--dropdown-border` | Dropdown border color. |  |
| `--input-background` | Input background color. |  |
| `--input-border` | Input border color. |  |
| `--input-text` | Input text color. |  |
| `--input-placeholder` | Input placeholder color. |  |
| `--input-addon-background` | Input addon background color. |  |
| `--input-addon-border` | Input addon border color. |  |
| `--notification-background` | Notification background color. |  |
| `--notification-border` | Notification border color. |  |
| `--notification-text` | Notification text color. |  |
| `--notification-item-background-hover` | Notification item hover background color. |  |
| `--notification-item-background-active` | Notification item active background color. |  |
| `--notification-item-background-border` | Notification item border background color. |  |
| `--notification-item-badge-background-readed` | Notification item badge background color for read items. |  |
| `--radio-border` | Radio button border color. |  |
| `--radio-ring` | Radio button ring color. |  |
| `--radio-disabled-background` | Radio button disabled background color. |  |
| `--radio-disabled-text` | Radio button disabled text color. |  |
| `--radio-disabled-ring` | Radio button disabled ring color. |  |
| `--radio-disabled-border` | Radio button disabled border color. |  |
| `--select-background` | Select background color. |  |
| `--select-border` | Select border color. |  |
| `--select-text` | Select text color. |  |
| `--select-single-value-text` | Select single value text color. |  |
| `--select-multi-value-background` | Select multi value background color. |  |
| `--select-multi-value-text` | Select multi value text color. |  |
| `--select-multi-value-border` | Select multi value border color. |  |
| `--select-menu-background` | Select menu background color. |  |
| `--select-menu-border` | Select menu border color. |  |
| `--select-menu-ring` | Select menu ring color. |  |
| `--select-option-hover-text` | Select option hover text color. |  |
| `--table-border` | Table border color. |  |
| `--table-header-background` | Table header background color. |  |
| `--table-header-text` | Table header text color. |  |
| `--table-header-border` | Table header border color. |  |
| `--table-body-border` | Table body border color. |  |
| `--table-footer-border` | Table footer border color. |  |
| `--table-row-background-hover` | Table row hover background color. |  |
| `--collapsible-background` | Collapsible component background color. |  |
| `--collapsible-text` | Collapsible component text color. |  |
| `--collapsible-border` | Collapsible component border color. |  |
| `--markdown-demo-code-background` | Markdown demo code background color. |  |
| `--markdown-demo-code-copy-button-hover-background` | Markdown demo code copy button hover background color. |  |
| `--markdown-demo-border` | Markdown demo border color. |  |
| `--markdown-tabs-border` | Markdown tabs border color. |  |
| `--markdown-tabs-text` | Markdown tabs text color. |  |
| `--markdown-note-bg` | Background color for markdown note alerts. |  |
| `--markdown-note-border` | Border color for markdown note alerts. |  |
| `--markdown-note-text` | Text color for markdown note alerts. |  |
| `--markdown-note-mark-bg` | Mark background color for markdown note alerts. |  |
| `--markdown-note-mark-text` | Mark text color for markdown note alerts. |  |
| `--markdown-important-bg` | Background color for markdown important alerts. |  |
| `--markdown-important-border` | Border color for markdown important alerts. |  |
| `--markdown-important-text` | Text color for markdown important alerts. |  |
| `--markdown-important-mark-bg` | Mark background color for markdown important alerts. |  |
| `--markdown-important-mark-text` | Mark text color for markdown important alerts. |  |
| `--markdown-tip-bg` | Background color for markdown tip alerts. |  |
| `--markdown-tip-border` | Border color for markdown tip alerts. |  |
| `--markdown-tip-text` | Text color for markdown tip alerts. |  |
| `--markdown-tip-mark-bg` | Mark background color for markdown tip alerts. |  |
| `--markdown-tip-mark-text` | Mark text color for markdown tip alerts. |  |
| `--markdown-warning-bg` | Background color for markdown warning alerts. |  |
| `--markdown-warning-border` | Border color for markdown warning alerts. |  |
| `--markdown-warning-text` | Text color for markdown warning alerts. |  |
| `--markdown-warning-mark-bg` | Mark background color for markdown warning alerts. |  |
| `--markdown-warning-mark-text` | Mark text color for markdown warning alerts. |  |
| `--markdown-caution-bg` | Background color for markdown caution alerts. |  |
| `--markdown-caution-border` | Border color for markdown caution alerts. |  |
| `--markdown-caution-text` | Text color for markdown caution alerts. |  |
| `--markdown-caution-mark-bg` | Mark background color for markdown caution alerts. |  |
| `--markdown-caution-mark-text` | Mark text color for markdown caution alerts. |  |
| `--markdown-task-border` | Markdown task border color. |