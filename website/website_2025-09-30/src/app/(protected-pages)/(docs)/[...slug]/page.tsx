import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import { notFound } from 'next/navigation'
import * as cheerio from 'cheerio'
import markdownit from 'markdown-it'
import hljs from 'highlight.js'

// Plugin styles
import '@mdit/plugin-alert/style'
import '@mdit/plugin-spoiler/style'

// Markdown-it Plugins
import { abbr } from '@mdit/plugin-abbr'
import { alert } from '@mdit/plugin-alert'
import { align } from '@mdit/plugin-align'
import { attrs } from '@mdit/plugin-attrs'
import { container } from '@mdit/plugin-container'
import { demo } from '@mdit/plugin-demo'
import { dl } from '@mdit/plugin-dl'
import { embed } from '@mdit/plugin-embed'
import { figure } from '@mdit/plugin-figure'
import { footnote } from '@mdit/plugin-footnote'
import { icon } from '@mdit/plugin-icon'
import { imgLazyload } from '@mdit/plugin-img-lazyload'
import { imgMark } from '@mdit/plugin-img-mark'
import { imgSize } from '@mdit/plugin-img-size'
import { include } from '@mdit/plugin-include'
import { ins } from '@mdit/plugin-ins'
import { katex } from '@mdit/plugin-katex'
import { mark } from '@mdit/plugin-mark'
import {
    createMathjaxInstance,
    MarkdownItMathjaxOptions,
    mathjax,
} from '@mdit/plugin-mathjax'
import { plantuml } from '@mdit/plugin-plantuml'
import { ruby } from '@mdit/plugin-ruby'
import { spoiler } from '@mdit/plugin-spoiler'
import { stylize } from '@mdit/plugin-stylize'
import { sub } from '@mdit/plugin-sub'
import { sup } from '@mdit/plugin-sup'
import { MarkdownItTabData, MarkdownItTabInfo, tab } from '@mdit/plugin-tab'
import { tasklist } from '@mdit/plugin-tasklist'
import { uml } from '@mdit/plugin-uml'
import { snippet } from '@mdit/plugin-snippet'
import MdItCollapsiblePlugin from 'markdown-it-collapsible'
// @ts-ignore
import markdownItKrokiPlugin from '@kazumatu981/markdown-it-kroki'

import anchor from 'markdown-it-anchor'
import toc from 'markdown-it-toc-done-right'
import { DocContent } from './components/DocContent'
import { buildTocFromRenderedHtml } from './components/utils'
import { escapeHtml } from 'markdown-it/lib/common/utils.mjs'

const contentRootDir = path.join(process.cwd(), 'content')

interface Params {
    slug: string[]
}
interface StaticDocPageProps {
    params: Promise<Params>
}
interface FrontMatter {
    title?: string
    [key: string]: unknown
}

const findFileCaseInsensitive = (
    directory: string,
    filenameWithoutExt: string,
    extensions: string[],
): string | null => {
    try {
        const entries = fs.readdirSync(directory, { withFileTypes: true })
        const lowerFilename = filenameWithoutExt.toLowerCase()

        for (const entry of entries) {
            if (entry.isFile()) {
                const entryNameLower = entry.name.toLowerCase()
                for (const ext of extensions) {
                    if (
                        entryNameLower ===
                        `${lowerFilename}${ext.toLowerCase()}`
                    ) {
                        return path.join(directory, entry.name)
                    }
                }
            }
        }
        return null
    } catch (error) {
        console.error(`Error reading directory ${directory}`, error)
        return null
    }
}

const readContentDir = (contentDir: string): string[] => {
    let filenames: string[] = []
    try {
        const entries = fs.readdirSync(contentDir, { withFileTypes: true })
        for (const entry of entries) {
            if (entry.isDirectory()) {
                filenames = filenames.concat(
                    readContentDir(path.join(contentDir, entry.name)),
                )
            } else {
                filenames.push(path.join(contentDir, entry.name))
            }
        }
        return filenames
    } catch (error) {
        console.error('Error reading content directory:', error)
        return []
    }
}

export async function generateStaticParams(): Promise<Params[]> {
    const filenames = readContentDir(contentRootDir)
    return filenames
        .filter((f) => f.endsWith('.md') || f.endsWith('.html'))
        .map((filename) => {
            const relativePath = path.relative(contentRootDir, filename)
            const slug = relativePath
                .replace(/\.(md|html)$/, '')
                .split(path.sep)
                .map((s) => s.toLowerCase())
            return { slug }
        })
}

export default async function StaticDocPage({ params }: StaticDocPageProps) {
    const { slug } = await params
    const slugPath = slug.join(path.sep)
    const filenameWithoutExt = slug[slug.length - 1]
    const fileDirectory = path.join(contentRootDir, ...slug.slice(0, -1))
    const foundFilePath = findFileCaseInsensitive(
        fileDirectory,
        filenameWithoutExt,
        ['.md', '.html'],
    )

    if (!foundFilePath) return notFound()

    let rawContent = ''
    let tocHtml = ''
    let frontMatter: FrontMatter = {}
    let isHtmlFile = false
    let useShadowDOM = false

    const ext = path.extname(foundFilePath).toLowerCase()

    try {
        if (ext === '.md') {
            const fileContents = fs.readFileSync(foundFilePath, 'utf8')
            const { data, content } = matter(fileContents)
            frontMatter = data

            const options: MarkdownItMathjaxOptions = {
                output: 'svg',
                delimiters: 'all',
                allowInlineWithSpace: false,
                mathFence: true,
                a11y: true,
                svg: { fontCache: 'global' },
            }
            const mathjaxInstance = createMathjaxInstance(options)

            const md = markdownit({
                html: true,
                linkify: true,
                typographer: true,
                highlight: (str: string, lang: string): string => {
                    if (lang && hljs.getLanguage(lang)) {
                        try {
                            return (
                                `<pre class="hljs-pre"><code class="hljs language-${lang}">` +
                                hljs.highlight(str, {
                                    language: lang,
                                    ignoreIllegals: true,
                                }).value +
                                '</code></pre>'
                            )
                        } catch (_) {}
                    }
                    return (
                        `<pre class="hljs-pre"><code class="hljs">` +
                        md.utils.escapeHtml(str) +
                        '</code></pre>'
                    )
                },
            })
                .use(abbr)
                .use(alert, {
                    deep: true,
                })
                .use(align)
                .use(attrs)
                .use(container, {
                    name: 'warning',
                })
                .use(demo, {
                    showCodeFirst: false,
                    openRender: () =>
                        `<div class="plugin-demo-container overflow-hidden rounded-lg border border-markdown-demo-border text-black dark:text-white">`,
                    closeRender: () => `</div>`,
                    contentOpenRender: () =>
                        `<div class="demo-preview p-5 overflow-x-auto min-w-full">`,
                    contentCloseRender: () => `</div>
                            <div class="flex items-center justify-between border-t border-markdown-demo-border p-4">
                                <div class="font-medium">Demo</div>
                                <button class="toggle-btn hover:text-primary transition-all duration-300">
                                    <svg stroke="currentColor" fill="none" stroke-width="2" viewBox="0 0 24 24" stroke-linecap="round" stroke-linejoin="round" height="20px" width="20px" xmlns="http://www.w3.org/2000/svg" data-darkreader-inline-stroke="" style="--darkreader-inline-stroke: currentColor;"><path d="m9 7-5 5 5 5"></path><path d="m15 7 5 5-5 5"></path></svg>
                                </button>
                            </div>`,
                    codeRender: (tokens, idx, options, env, self) => {
                        const code = tokens[idx].content
                        return `<div class="code-section relative group rounded-b-lg">
                                    <div class="demo-code-container hidden text-base bg-markdown-demo-code-background border-t border-markdown-demo-border rounded-b-lg overflow-x-auto p-4">
                                        <pre data-code="${escapeHtml(code)}" class="!text-black dark:!text-white !bg-transparent p-2 !my-0">${escapeHtml(code)}</pre>
                                    </div>
                                    <button class="copy-button absolute right-4 top-4 opacity-0 group-hover:opacity-100 group-hover:bg-markdown-demo-code-copy-button-hover-background rounded-lg transition-all duration-300 p-2">
                                       <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-clipboard-icon lucide-clipboard"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>
                                    </button>
                                </div>`
                    },
                })
                .use(dl)
                .use(embed, {
                    config: [
                        {
                            name: 'youtube',
                            setup: (id: string) =>
                                `<div class="relative w-full lg:pb-[315px] pb-[56.25%] lg:max-w-[560px] lg:max-h-[315px]">
                                    <iframe class="absolute top-0 left-0 w-full h-full" src="https://www.youtube.com/embed/${id}" frameborder="0" allowfullscreen></iframe>
                                </div>`,
                        },
                    ],
                })
                .use(figure)
                .use(footnote)
                .use(icon)
                .use(imgLazyload)
                .use(imgMark)
                .use(imgSize)
                .use(include, {
                    currentPath: (env) => {
                        if (!env || !env.filePath) {
                            return contentRootDir
                        }
                        return path.dirname(env.filePath)
                    },
                    deep: true,
                })
                .use(snippet, {
                    currentPath: (env) => {
                        if (!env || !env.filePath) {
                            return contentRootDir
                        }
                        return path.dirname(env.filePath)
                    },
                })
                .use(ins)
                .use(katex)
                .use(mark)
                .use(mathjax, mathjaxInstance)
                .use(plantuml)
                .use(ruby)
                .use(spoiler)
                .use(stylize, {
                    config: [
                        // 🎨 Text color (supports Tailwind + hex)
                        {
                            matcher: /^(?!bg-)([#a-zA-Z0-9-]+):(.*)$/,
                            replacer: ({ tag, content }) => {
                                if (tag !== 'mark') return

                                const match = content.match(
                                    /^(?!bg-)([#a-zA-Z0-9-]+):(.*)$/,
                                )
                                if (!match) return
                                let [_, color, text] = match
                                if (!text) return

                                if (color.startsWith('#')) {
                                    // Custom hex → inline style
                                    return {
                                        tag: 'span',
                                        attrs: { style: `color: ${color};` },
                                        content: text.trim(),
                                    }
                                } else {
                                    // Add default shade if none provided
                                    if (!/-\d{3}$/.test(color)) {
                                        color = `${color}-600`
                                    }
                                    return {
                                        tag: 'span',
                                        attrs: { class: `text-${color}` },
                                        content: text.trim(),
                                    }
                                }
                            },
                        },

                        // 🎨 Background color (supports Tailwind + hex)
                        {
                            matcher: /^bg-([#a-zA-Z0-9-]+):(.*)$/,
                            replacer: ({ tag, content }) => {
                                if (tag !== 'mark') return

                                const match = content.match(
                                    /^bg-([#a-zA-Z0-9-]+):(.*)$/,
                                )
                                if (!match) return
                                let [_, bg, text] = match
                                if (!text) return

                                if (bg.startsWith('#')) {
                                    // Custom hex → inline style
                                    return {
                                        tag: 'span',
                                        attrs: {
                                            style: `background-color: ${bg}; padding: 0.15rem 0.3rem; border-radius: 0.25rem;`,
                                        },
                                        content: text.trim(),
                                    }
                                } else {
                                    // Add default shade if none provided
                                    if (!/-\d{3}$/.test(bg)) {
                                        bg = `${bg}-200`
                                    }
                                    return {
                                        tag: 'span',
                                        attrs: {
                                            class: `bg-${bg} dark:bg-${bg.replace(/\d+$/, '800')} text-black dark:text-black px-1 rounded-xs`,
                                            style: `color: ${bg.replace(/-\d+$/, '').replace(/-/g, '')};`,
                                        },
                                        content: text.trim(),
                                    }
                                }
                            },
                        },
                    ],
                })
                .use(sub)
                .use(sup)
                .use(tab, {
                    openRender: (info: MarkdownItTabInfo) => {
                        const nav = info.data
                            .map(
                                (tab, i) => `
                            <button
                                type="button"
                                class="tabs-nav-btn ${
                                    i === info.active
                                        ? 'tabs-nav-btn-active'
                                        : 'tabs-nav-btn-inactive'
                                }"
                                role="tab"
                                aria-controls="${tab.id}"
                                aria-selected="${i === info.active}"
                                data-tab="${tab.id}"
                            >
                                ${tab.title}
                            </button>`,
                            )
                            .join('')

                        return `
                        <div class="tabs">
                            <div class="tabs-nav" role="tablist">
                            ${nav}
                            </div>
                        `
                    },
                    closeRender: () => `</div>`,
                    tabOpenRender: (
                        data: MarkdownItTabData,
                        i: number,
                        info: MarkdownItTabInfo,
                    ) => {
                        // use info.active (the plugin already sets this when you use :active)
                        const isActive = i === info.active
                        return `<div class="tabs-panel ${
                            isActive ? 'tabs-panel-active' : 'tabs-panel-hidden'
                        }" id="${data.id}" role="tabpanel" aria-expanded="${isActive}">`
                    },

                    tabCloseRender: () => `</div>`,
                })
                .use(tasklist)
                .use(uml, {
                    name: 'mermaid',
                    open: 'mermaidstart',
                    close: 'mermaidend',
                    render: (tokens, index) => {
                        const token = tokens[index]
                        return `<div class="uml-block mermaid">${token.content}</div>`
                    },
                })
                .use(anchor, {
                    permalink: anchor.permalink.linkInsideHeader({
                        symbol: '',
                        placement: 'before',
                    }),
                    slugify: (s) =>
                        encodeURIComponent(
                            String(s).trim().toLowerCase().replace(/\s+/g, '-'),
                        ),
                })
                .use(toc, {
                    level: [1, 2, 3, 4, 5, 6],
                    containerClass: 'markdown-toc',
                    linkClass: 'dark:text-white no-underline hover:underline',
                    listType: 'ul',
                    callback: (html: string) => {
                        tocHtml =
                            html.trim() === '<nav class="markdown-toc"></nav>'
                                ? ''
                                : html
                    },
                })
                .use(MdItCollapsiblePlugin)
                .use(markdownItKrokiPlugin, {
                    // entrypoint: 'https://kroki.io',
                    entrypoint: process.env.NEXT_PUBLIC_KROKI_SERVER_URL || "https://kroki.io",
                    render: (encodedUrl: string, altText: string) => {
                        const correctedEncodedUrl = encodedUrl.replace(
                            'plantuml',
                            altText,
                        )
                        return `<embed class="markdown-it-kroki" title="${altText}" src="${correctedEncodedUrl}" />`
                    },
                })

            rawContent = md.render(content, { filePath: foundFilePath })
            isHtmlFile = false
            useShadowDOM = false
        } else if (ext === '.html') {
            const fileContents = fs.readFileSync(foundFilePath, 'utf8')
            rawContent = fileContents
            isHtmlFile = true

            const $ = cheerio.load(fileContents)
            useShadowDOM =
                $('style').length > 0 || $('link[rel="stylesheet"]').length > 0

            tocHtml = buildTocFromRenderedHtml(rawContent, 1, 6)
        } else {
            return notFound()
        }
    } catch (error) {
        console.error('Error rendering content:', error)
        return notFound()
    }

    return (
        <DocContent
            docContentClientProps={{
                rawContent: rawContent,
                isHtmlFile: isHtmlFile,
                frontMatterTitle: frontMatter?.title,
                useShadowDOM: useShadowDOM,
            }}
            tocHtml={tocHtml}
        />
    )
}

export async function generateMetadata({ params }: StaticDocPageProps) {
    const { slug } = await params
    const filenameWithoutExt = slug[slug.length - 1]
    const fileDirectory = path.join(contentRootDir, ...slug.slice(0, -1))
    const foundFilePath = findFileCaseInsensitive(
        fileDirectory,
        filenameWithoutExt,
        ['.md', '.html'],
    )

    let pageTitle = 'Documentation'
    try {
        if (foundFilePath) {
            const ext = path.extname(foundFilePath).toLowerCase()
            if (ext === '.md') {
                const fileContents = fs.readFileSync(foundFilePath, 'utf8')
                const { data } = matter(fileContents)
                pageTitle =
                    (data as FrontMatter).title ||
                    path.basename(foundFilePath, ext)
            } else {
                pageTitle = path.basename(foundFilePath, ext)
            }
        }
    } catch {
        pageTitle = 'Error Loading Doc'
    }

    return {
        title: pageTitle.replaceAll('-', ' '),
    }
}
