'use server'

export interface GoogleFont {
    family: string
    category: string
    variants: string[]
    subsets?: string[]
}

export interface GoogleFontsResponse {
    items: GoogleFont[]
}

export async function getGoogleFontsAction(): Promise<GoogleFont[]> {
    const apiKey = process.env.GOOGLE_FONTS_API_KEY
    if (!apiKey) throw new Error('Missing GOOGLE_FONTS_API_KEY env var')

    const res = await fetch(
        `https://www.googleapis.com/webfonts/v1/webfonts?sort=popularity&key=${apiKey}`,
        { next: { tags: ['google-fonts'] } },
    )

    if (!res.ok) {
        throw new Error('Failed to fetch Google Fonts')
    }

    const data: GoogleFontsResponse = await res.json()
    return data.items
}
