'use client'

import React, { useEffect, useState } from 'react'
import DocContentClient, { DocContentClientProps } from './DocContentClient'
import { TableOfContent } from './TableOfContent'
import mermaid from 'mermaid'
import { NavigationButtons } from './NavigationButtons'
import { Breadcrumbs } from './Breadcrumbs'
import useTheme from '@/utils/hooks/useTheme'
import { DARK_MODE_OPTIONS } from '@/constants/color-theme.constant'

interface DocContentProps {
    docContentClientProps: DocContentClientProps
    tocHtml?: string
}

const DocContent: React.FC<DocContentProps> = ({
    docContentClientProps,
    tocHtml,
}) => {
    const [loading, setLoading] = useState(true)
    const themeState = useTheme((s) => s)

    useEffect(() => {
        if (!loading) {
            const timeout = setTimeout(() => {
                const isDarkTheme = DARK_MODE_OPTIONS.some(
                    (option) => option.value === themeState.selectedMode,
                )
                // Mermaid
                mermaid.initialize({
                    startOnLoad: true,
                    theme: isDarkTheme ? 'dark' : 'default',
                })
                mermaid.run()

                // Toggle demo code containers
                const toggles =
                    document.querySelectorAll<HTMLButtonElement>('.toggle-btn')
                toggles.forEach((btn) => {
                    btn.addEventListener('click', () => {
                        const container = btn.closest('.plugin-demo-container')
                        if (!container) return
                        const codeBlock = container.querySelector<HTMLElement>(
                            '.demo-code-container',
                        )
                        if (!codeBlock) return

                        codeBlock.classList.toggle('hidden')
                    })
                })

                // Copy code functionality
                const copyButtons =
                    document.querySelectorAll<HTMLButtonElement>('.copy-button')
                copyButtons.forEach((btn) => {
                    btn.addEventListener('click', () => {
                        const container = btn.closest('.code-section')
                        if (!container) return
                        const pre =
                            container.querySelector<HTMLElement>(
                                'pre[data-code]',
                            )
                        if (!pre) return

                        const code = pre.getAttribute('data-code') || ''
                        navigator.clipboard
                            .writeText(code)
                            .then(() => {
                                btn.innerHTML =
                                    '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-clipboard-check-icon lucide-clipboard-check"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><path d="m9 14 2 2 4-4"/></svg>'
                                setTimeout(() => {
                                    btn.innerHTML =
                                        '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-clipboard-icon lucide-clipboard"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>'
                                }, 1500)
                            })
                            .catch((err) => {
                                console.error('Failed to copy code:', err)
                            })
                    })
                })

                // Tabs
                const tabsContainers =
                    document.querySelectorAll<HTMLElement>('.tabs')

                tabsContainers.forEach((container) => {
                    const buttons =
                        container.querySelectorAll<HTMLButtonElement>(
                            '.tabs-nav-btn',
                        )
                    const panels =
                        container.querySelectorAll<HTMLElement>('.tabs-panel')

                    // 🔹 INITIALIZE active tab (from @tab:active or fallback to first)
                    let activeIndex = Array.from(buttons).findIndex(
                        (b) => b.getAttribute('aria-selected') === 'true',
                    )
                    if (activeIndex === -1) activeIndex = 0

                    buttons.forEach((b, i) => {
                        if (i === activeIndex) {
                            b.classList.add('tabs-nav-btn-active')
                            b.classList.remove('tabs-nav-btn-inactive')
                            b.setAttribute('aria-selected', 'true')
                            panels[i]?.classList.add('tabs-panel-active')
                            panels[i]?.classList.remove('tabs-panel-hidden')
                            panels[i]?.setAttribute('aria-expanded', 'true')
                        } else {
                            b.classList.remove('tabs-nav-btn-active')
                            b.classList.add('tabs-nav-btn-inactive')
                            b.setAttribute('aria-selected', 'false')
                            panels[i]?.classList.remove('tabs-panel-active')
                            panels[i]?.classList.add('tabs-panel-hidden')
                            panels[i]?.setAttribute('aria-expanded', 'false')
                        }
                    })

                    // 🔹 CLICK HANDLERS
                    buttons.forEach((btn, i) => {
                        btn.addEventListener('click', () => {
                            buttons.forEach((b) => {
                                b.classList.remove('tabs-nav-btn-active')
                                b.classList.add('tabs-nav-btn-inactive')
                                b.setAttribute('aria-selected', 'false')
                            })
                            panels.forEach((p) => {
                                p.classList.remove('tabs-panel-active')
                                p.classList.add('tabs-panel-hidden')
                                p.setAttribute('aria-expanded', 'false')
                            })

                            btn.classList.add('tabs-nav-btn-active')
                            btn.classList.remove('tabs-nav-btn-inactive')
                            btn.setAttribute('aria-selected', 'true')

                            const panel = panels[i]
                            if (panel) {
                                panel.classList.add('tabs-panel-active')
                                panel.classList.remove('tabs-panel-hidden')
                                panel.setAttribute('aria-expanded', 'true')
                            }
                        })
                    })
                })
            }, 0)

            return () => {
                clearTimeout(timeout)
                // cleanup listeners by replacing nodes with clones
                document
                    .querySelectorAll<HTMLButtonElement>('.toggle-btn')
                    .forEach((btn) => {
                        btn.replaceWith(btn.cloneNode(true))
                    })
                document
                    .querySelectorAll<HTMLButtonElement>('.copy-button')
                    .forEach((btn) => {
                        btn.replaceWith(btn.cloneNode(true))
                    })
                document
                    .querySelectorAll<HTMLButtonElement>('.tabs-nav-btn')
                    .forEach((btn) => {
                        btn.replaceWith(btn.cloneNode(true))
                    })
            }
        }
    }, [loading, themeState])

    return (
        <div className="flex lg:flex-row flex-col-reverse justify-center xl:gap-8 lg:gap-6">
            <div
                className={`w-full space-y-4 ${!loading ? 'max-w-[910px]' : ''} min-w-0`}
            >
                {/* Breadcrumbs */}
                {!loading && (
                    <div className="w-full px-8 pt-2 max-lg:hidden">
                        <Breadcrumbs />
                    </div>
                )}
                {/* Content */}
                <DocContentClient
                    {...docContentClientProps}
                    setLoading={docContentClientProps.setLoading || setLoading}
                />
                {!loading && <NavigationButtons />}
            </div>
            {/* Table of content */}
            {tocHtml && tocHtml.length > 0 && !loading && (
                <aside className="min-w-75 lg:max-w-84 h-fit max-h-[90vh] lg:sticky top-20 overflow-y-auto lg:px-3 px-8 max-lg:py-4">
                    <div className="lg:border-l border-toc-border lg:pl-4">
                        <TableOfContent
                            title="On this page"
                            tocHtml={tocHtml}
                        />
                    </div>
                </aside>
            )}
            {!loading && (
                <div className="w-full px-8 pt-2 pb-2 lg:hidden">
                    <Breadcrumbs />
                </div>
            )}
        </div>
    )
}

export { DocContent }
