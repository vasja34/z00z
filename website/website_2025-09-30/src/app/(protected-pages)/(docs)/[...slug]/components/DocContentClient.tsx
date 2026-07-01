// app/docs/[...slug]/components/DocContentClient.tsx

'use client'

import React, { useEffect, useRef, useState, useLayoutEffect } from 'react' // Import useLayoutEffect
import Loading from '@/components/shared/Loading'
import * as cheerio from 'cheerio'
import hljs from 'highlight.js'

import { MODE_DARK } from '@/constants/theme.constant'
import useTheme from '@/utils/hooks/useTheme'
// import HighlightThemeSwitcher from './HighlightThemeSwitcher'

export interface DocContentClientProps {
    rawContent: string
    isHtmlFile: boolean
    frontMatterTitle?: string
    useShadowDOM: boolean
    // availableThemes: string[]
    setLoading?: React.Dispatch<React.SetStateAction<boolean>>
}

const DocContentClient: React.FC<DocContentClientProps> = ({
    rawContent,
    isHtmlFile,
    frontMatterTitle,
    useShadowDOM,
    setLoading,
}) => {
    const [processedHtml, setProcessedHtml] = useState('')
    const [isLoading, setIsLoading] = useState(true)
    const shadowHostRef = useRef<HTMLDivElement>(null)

    const mode = useTheme((state) => state.mode)
    const fontSettings = useTheme((state) => state.fontSettings)

    const isDarkMode = mode === MODE_DARK

    // Initialize themeTargetDocument to null, it will be set dynamically
    const [themeTargetDocument, setThemeTargetDocument] = useState<
        Document | ShadowRoot | null
    >(null)

    // This useEffect processes the raw content (HTML via Cheerio)
    useEffect(() => {
        console.log('DocContentClient: processContent Effect Running')
        const processContent = async () => {
            setIsLoading(true)
            setLoading?.(true)
            let currentHtmlContent = rawContent

            try {
                if (isHtmlFile) {
                    const $ = cheerio.load(rawContent)

                    // 1. --- Apply Code Highlighting (Existing Logic, for any <pre><code> found in HTML) ---
                    if ($('pre code').length > 0) {
                        $('pre code').each((_, el) => {
                            const code = $(el)
                            const text = code.text()
                            let lang = ''
                            const cls = code.attr('class')
                            if (cls) {
                                const match = cls.match(
                                    /(?:lang|language)-([a-zA-Z0-9]+)/,
                                )
                                if (match) lang = match[1]
                            }
                            const highlighted =
                                lang && hljs.getLanguage(lang)
                                    ? hljs.highlight(text, {
                                          language: lang,
                                          ignoreIllegals: true,
                                      }).value
                                    : hljs.highlightAuto(text).value
                            code.html(highlighted).addClass(
                                `hljs language-${lang}`,
                            )
                        })
                    }

                    // 2. --- CSS Selector Transformation & Dark Mode Filter Injection (Only if useShadowDOM) ---
                    if (useShadowDOM) {
                        $('style').each((_, el) => {
                            const styleContent = $(el).html()
                            if (styleContent) {
                                let transformedStyleContent = styleContent

                                transformedStyleContent =
                                    transformedStyleContent.replace(
                                        /:root\s*\{/g,
                                        ':host {',
                                    )
                                transformedStyleContent =
                                    transformedStyleContent.replace(
                                        /html\s*\{/g,
                                        ':host {',
                                    )
                                transformedStyleContent =
                                    transformedStyleContent.replace(
                                        /body\s*\{/g,
                                        ':host {',
                                    )

                                $(el).html(transformedStyleContent)
                            }
                        })

                        const darkModeFilterStyle = `
                            :host {
                                filter: ${isDarkMode ? 'invert(1) hue-rotate(180deg)' : 'none'};
                                transition: filter 0.3s ease;
                            }
                            :host img, :host video {
                                filter: ${isDarkMode ? 'invert(1) hue-rotate(180deg)' : 'none'};
                            }
                        `
                        const darkModeStyleTag = `<style class="dark-mode-filter-style">${darkModeFilterStyle}</style>`

                        // Ensure only one dark mode filter style tag is appended if it's already there
                        if ($('.dark-mode-filter-style').length) {
                            $('.dark-mode-filter-style').html(
                                darkModeFilterStyle,
                            )
                        } else if ($('head').length) {
                            $('head').append(darkModeStyleTag)
                        } else {
                            $.root().append(darkModeStyleTag)
                        }

                        currentHtmlContent = $.html()
                    } else {
                        currentHtmlContent = $.html()
                    }
                } else {
                    currentHtmlContent = rawContent
                }

                setProcessedHtml(currentHtmlContent)
            } catch (err) {
                console.error('DocContentClient: Client processing error:', err)
            } finally {
                setIsLoading(false)
                setLoading?.(false)
            }
        }

        processContent()
    }, [rawContent, isHtmlFile, useShadowDOM, mode])

    // NEW useEffect for Shadow DOM content injection AND setting themeTargetDocument
    // We use useLayoutEffect here to ensure the Shadow DOM is attached
    // and themeTargetDocument is set before the browser paints.
    useLayoutEffect(() => {
        if (!isLoading && useShadowDOM && isHtmlFile && shadowHostRef.current) {
            let shadowRoot: ShadowRoot
            if (shadowHostRef.current.shadowRoot) {
                shadowRoot = shadowHostRef.current.shadowRoot
                console.log('DocContentClient: Reusing existing ShadowRoot.')
            } else {
                shadowRoot = shadowHostRef.current.attachShadow({
                    mode: 'open',
                })
                console.log('DocContentClient: Attached new ShadowRoot.')
            }

            // Always set themeTargetDocument here as soon as ShadowRoot is available
            setThemeTargetDocument(shadowRoot)

            // Clear existing content before injecting new content
            shadowRoot.innerHTML = ''

            // Create a temporary div to parse the processed HTML
            const tempDiv = document.createElement('div')
            tempDiv.innerHTML = processedHtml

            // Append all children of the temporary div to the shadowRoot
            while (tempDiv.firstChild) {
                shadowRoot.appendChild(tempDiv.firstChild)
            }

            // Handle external scripts within the Shadow DOM (re-create to ensure execution)
            shadowRoot.querySelectorAll('script').forEach((oldScript) => {
                const newScript = document.createElement('script')
                Array.from(oldScript.attributes).forEach((attr) => {
                    newScript.setAttribute(attr.name, attr.value)
                })
                newScript.textContent = oldScript.textContent // Copy script content
                oldScript.parentNode?.replaceChild(newScript, oldScript)
            })
        } else if (!isLoading && !useShadowDOM) {
            // For Markdown or non-Shadow DOM HTML, target is always the main document.
            // Ensure this is only run on client-side (after hydration).
            if (typeof document !== 'undefined') {
                if (themeTargetDocument !== document) {
                    console.log(
                        'DocContentClient: Setting target to main document.',
                    )
                    setThemeTargetDocument(document)
                }
            }
            // Explicitly clear shadowRoot if we navigated from a shadow DOM page to a non-shadow DOM one.
            if (shadowHostRef.current?.shadowRoot) {
                console.log(
                    'DocContentClient: Clearing existing shadowRoot from previous state.',
                )
                shadowHostRef.current.shadowRoot.innerHTML = ''
            }
            // If not using Shadow DOM, ensure themeTargetDocument is not a ShadowRoot.
            if (
                themeTargetDocument &&
                (themeTargetDocument as ShadowRoot).host
            ) {
                // Check if it's a ShadowRoot
                console.log(
                    'DocContentClient: Clearing themeTargetDocument as it was ShadowRoot and not needed.',
                )
                setThemeTargetDocument(null) // Or set to document if it's the default
            }
        } else if (isLoading) {
            console.log(
                'DocContentClient: Still loading, themeTargetDocument not set yet.',
            )
            setThemeTargetDocument(null) // Ensure null while loading to prevent premature injection attempts
        }
        console.log(
            'DocContentClient: Final themeTargetDocument state value:',
            themeTargetDocument,
        )
    }, [
        isLoading,
        useShadowDOM,
        isHtmlFile,
        processedHtml,
        shadowHostRef.current,
    ])

    return (
        <div
            className={
                !useShadowDOM
                    ? 'prose dark:prose-invert max-w-none mx-auto px-8 py-6'
                    : ''
            }
        >
            {frontMatterTitle && <h1>{frontMatterTitle}</h1>}

            {/* Render the reusable HighlightThemeSwitcher component */}
            {/* <HighlightThemeSwitcher
                availableThemes={availableThemes}
                // targetDocument={themeTargetDocument} // Pass the dynamically determined target
            /> */}

            <div>
                <style>
                    {`
                    .dynamic-doc-content h1 {
                        font-size: ${fontSettings.h1.fontSize}px !important;
                        font-weight: ${fontSettings.h1.fontWeight} !important;
                        font-family: ${fontSettings.h1.fontFamily || 'Inter'}, sans-serif !important;
                    }
                    .dynamic-doc-content h2 {
                        font-size: ${fontSettings.h2.fontSize}px !important;
                        font-weight: ${fontSettings.h2.fontWeight} !important;
                        font-family: ${fontSettings.h2.fontFamily || 'Inter'}, sans-serif !important;
                    }
                    .dynamic-doc-content h3 {
                        font-size: ${fontSettings.h3.fontSize}px !important;
                        font-weight: ${fontSettings.h3.fontWeight} !important;
                        font-family: ${fontSettings.h3.fontFamily || 'Inter'}, sans-serif !important; 
                    }
                    .dynamic-doc-content h4 {
                        font-size: ${fontSettings.h4.fontSize}px !important;
                        font-weight: ${fontSettings.h4.fontWeight} !important;
                        font-family: ${fontSettings.h4.fontFamily || 'Inter'}, sans-serif !important;
                    }
                    .dynamic-doc-content h5 {
                        font-size: ${fontSettings.h5.fontSize}px !important;
                        font-weight: ${fontSettings.h5.fontWeight} !important;
                        font-family: ${fontSettings.h5.fontFamily || 'Inter'}, sans-serif !important;
                    }
                    .dynamic-doc-content h6 {
                        font-size: ${fontSettings.h6.fontSize}px !important;
                        font-weight: ${fontSettings.h6.fontWeight} !important;
                        font-family: ${fontSettings.h6.fontFamily || 'Inter'}, sans-serif !important;
                    }
                    .dynamic-doc-content p {
                        font-size: ${fontSettings.p.fontSize}px !important;
                        font-weight: ${fontSettings.p.fontWeight} !important;
                        font-family: ${fontSettings.p.fontFamily || 'Inter'}, sans-serif !important;
                    }
                `}
                </style>
                {isLoading ? (
                    <div className="min-h-screen flex justify-center">
                        <Loading type="cover" loading={isLoading} />
                    </div>
                ) : useShadowDOM && isHtmlFile ? (
                    <div
                        ref={shadowHostRef}
                        className="dynamic-doc-content shadow-dom-container"
                    />
                ) : (
                    <div
                        className="dynamic-doc-content"
                        dangerouslySetInnerHTML={{ __html: processedHtml }}
                    />
                )}
            </div>
        </div>
    )
}

export default DocContentClient
