export function getSnippets(
    content: string,
    query: string,
    snippetLength = 200,
    maxMatches = 3,
): string {
    const lowerQuery = query.toLowerCase()

    // Split into lines
    const lines = content.split(/\r?\n/)

    // Collect all matching lines
    const matches: string[] = []

    for (const line of lines) {
        if (line.toLowerCase().includes(lowerQuery)) {
            let snippet = line.trim()

            // If line is too long, trim around the first match
            const index = snippet.toLowerCase().indexOf(lowerQuery)
            if (snippet.length > snippetLength) {
                const start = Math.max(0, index - Math.floor(snippetLength / 2))
                const end = Math.min(
                    snippet.length,
                    index + lowerQuery.length + Math.floor(snippetLength / 2),
                )
                snippet =
                    (start > 0 ? '...' : '') +
                    snippet.substring(start, end) +
                    (end < line.length ? '...' : '')
            }

            // Underline all occurrences
            const regex = new RegExp(`(${query})`, 'ig')
            snippet = snippet.replace(regex, '<u>$1</u>')

            matches.push(snippet)

            if (matches.length >= maxMatches) break
        }
    }

    // Fallback: no matches → return preview
    if (matches.length === 0) {
        const fallback = lines.find((line) => line.trim().length > 0) || content
        return [fallback.substring(0, snippetLength) + '...'].join('\n')
    }

    return matches.join('\n')
}
