import { NavigationTree } from '@/@types/navigation'
import * as cheerio from 'cheerio'

const escapeHtml = (s: string) =>
    s.replace(
        /[&<>"']/g,
        (c) =>
            ({
                '&': '&amp;',
                '<': '&lt;',
                '>': '&gt;',
                '"': '&quot;',
                "'": '&#039;',
            })[c]!,
    )

export const buildTocFromRenderedHtml = (
    html: string,
    minLevel = 2,
    maxLevel = 6,
): string => {
    const $ = cheerio.load(html)
    const selector = Array.from(
        { length: maxLevel - minLevel + 1 },
        (_, i) => `h${i + minLevel}[id]`,
    ).join(',')
    const nodes = $(selector).toArray()
    if (nodes.length === 0) return ''

    const items = nodes.map((el) => {
        const level = Number((el as any).tagName[1])
        const id = $(el).attr('id') || ''
        const $el = $(el).clone()
        $el.find('a, .header-anchor').remove() // strip permalinks like ¶
        const text = $el.text().trim()
        return { level, id, text }
    })

    if (items.length === 0) return ''

    let base = Math.min(...items.map((i) => i.level))
    let cur = base - 1
    let out = ''

    for (const h of items) {
        while (cur < h.level) {
            out += '<ul>'
            cur++
        }
        while (cur > h.level) {
            out += '</ul>'
            cur--
        }
        out += `<li><a class="dark:text-white no-underline hover:underline" href="#${h.id}">${escapeHtml(h.text)}</a></li>`
    }
    while (cur >= base) {
        out += '</ul>'
        cur--
    }

    return out
}

// 🔍 Recursively search navigation tree to find breadcrumb path
export const findBreadcrumbs = (
    items: NavigationTree[],
    targetPath: string,
    parents: NavigationTree[] = [],
): NavigationTree[] | null => {
    for (const item of items) {
        const currentPath = item.path

        // Exact match
        if (currentPath && currentPath === targetPath) {
            return [...parents, item]
        }

        // Recurse into subMenu
        if (item.subMenu?.length) {
            const result = findBreadcrumbs(item.subMenu, targetPath, [
                ...parents,
                item,
            ])
            if (result) return result
        }
    }
    return null
}

interface Navigations {
    key: string
    path: string
    title: string
}

export const flattenNav = (tree: NavigationTree[]): Navigations[] => {
    const result: Navigations[] = []
    const walk = (items: NavigationTree[]) => {
        for (const item of items) {
            if (item.type === 'item' && item.path) {
                result.push({
                    key: item.key,
                    path: item.path,
                    title: item.title,
                })
            }
            if (item.subMenu && item.subMenu.length > 0) {
                walk(item.subMenu)
            }
        }
    }
    walk(tree)
    return result
}
