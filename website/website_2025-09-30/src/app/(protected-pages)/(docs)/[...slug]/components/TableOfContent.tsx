import React, { useEffect, useState } from 'react'
import { BiChevronDown, BiChevronUp } from 'react-icons/bi'
import { AnimatePresence, motion } from 'framer-motion'

interface TableOfContentProps {
    title?: string
    tocHtml: string
}

const TableOfContent: React.FC<TableOfContentProps> = ({
    tocHtml,
    title = 'On this page',
}) => {
    const [isOpen, setIsOpen] = useState(false)

    useEffect(() => {
        const handleClick = (e: Event) => {
            const target = e.target as HTMLElement | null
            if (!target) return

            const anchor = target.closest('a') as HTMLAnchorElement | null
            if (!anchor) return

            const href = anchor.getAttribute('href') || ''
            if (!href.startsWith('#')) return

            e.preventDefault()
            setIsOpen(false)

            const hash = href
            const id = hash.slice(1)

            const el = document.getElementById(id)
            if (el) {
                el.scrollIntoView({ behavior: 'instant', block: 'start' })
                history.replaceState(null, '', hash)
            } else {
                window.location.hash = hash
            }
        }

        document.addEventListener('click', handleClick)
        return () => document.removeEventListener('click', handleClick)
    }, [tocHtml])

    return (
        <div>
            {/* Desktop TOC */}
            <div className="hidden lg:block">
                <div className="text-sm font-semibold text-toc-title-text mb-2">
                    {title}
                </div>
                <div
                    className="prose prose-sm text-xs leading-[21px]"
                    dangerouslySetInnerHTML={{ __html: tocHtml }}
                />
            </div>

            {/* Mobile TOC */}
            <div className="block lg:hidden bg-toc-background rounded-xl px-4 py-2.5">
                <button
                    onClick={() => setIsOpen(!isOpen)}
                    className="flex w-full items-center justify-between text-base font-medium text-black dark:text-white"
                >
                    <span>{title}</span>
                    {isOpen ? (
                        <BiChevronUp className="w-5 h-5" />
                    ) : (
                        <BiChevronDown className="w-5 h-5" />
                    )}
                </button>

                <AnimatePresence initial={false}>
                    {isOpen && (
                        <motion.div
                            key="toc-content"
                            className="overflow-hidden"
                            initial={{ height: 0 }}
                            animate={{ height: 'auto' }}
                            transition={{ duration: 0.3, ease: 'easeOut' }}
                        >
                            <div
                                className="prose prose-sm py-2"
                                dangerouslySetInnerHTML={{ __html: tocHtml }}
                            />
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </div>
    )
}

export { TableOfContent }
