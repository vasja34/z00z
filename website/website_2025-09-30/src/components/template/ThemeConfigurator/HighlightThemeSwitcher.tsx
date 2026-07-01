'use client'

import React from 'react'
import { Select } from '@/components/ui'
import useTheme from '@/utils/hooks/useTheme'

type ThemeOption = { label: string; value: string }

const HighlightThemeSwitcher: React.FC = () => {
    const highlightTheme = useTheme((s) => s.highlightTheme)
    const setHighlightTheme = useTheme((s) => s.setHighlightTheme)

    return (
        <Select<ThemeOption>
            id="highlight-theme-select"
            className="w-full"
            size="md"
            menuPlacement="auto"
            isSearchable={false}
            options={allOptions}
            value={{
                label: THEME_NAME_LABEL_MAP[highlightTheme] || highlightTheme,
                value: highlightTheme,
            }}
            onChange={(option) => setHighlightTheme(option?.value || 'xcode')}
        />
    )
}

const THEME_NAME_LABEL_MAP: Record<string, string> = {
    'atom-one-dark-reasonable': 'Atom Dark',
    'monokai-sublime': 'Monokai Dark',
    'night-owl': 'Owl Dark',
    'atom-one-light': 'Atom Light',
    xcode: 'Xcode Light',
    github: 'Github Light',
}

const darkThemes: ThemeOption[] = [
    {
        value: 'atom-one-dark-reasonable',
        label: THEME_NAME_LABEL_MAP['atom-one-dark-reasonable'],
    },
    {
        value: 'monokai-sublime',
        label: THEME_NAME_LABEL_MAP['monokai-sublime'],
    },
    { value: 'night-owl', label: THEME_NAME_LABEL_MAP['night-owl'] },
]

const lightThemes: ThemeOption[] = [
    {
        value: 'atom-one-light',
        label: THEME_NAME_LABEL_MAP['atom-one-light'],
    },
    { value: 'xcode', label: THEME_NAME_LABEL_MAP['xcode'] },
    { value: 'github', label: THEME_NAME_LABEL_MAP['github'] },
]

const allOptions = [
    { label: 'Dark Themes', options: darkThemes },
    { label: 'Light Themes', options: lightThemes },
]

export default HighlightThemeSwitcher
