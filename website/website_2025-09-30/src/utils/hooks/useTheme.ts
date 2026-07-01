'use client'

import { useContext } from 'react'
import ThemeContext from '@/components/template/Theme/ThemeContext'
import { MODE_DARK, MODE_LIGHT } from '@/constants/theme.constant'
import presetThemeSchemaConfig from '@/configs/preset-theme-schema.config'
import applyTheme from '@/utils/applyThemeSchema'
import type {
    Mode,
    Direction,
    LayoutType,
    Theme,
    FontSettings,
} from '@/@types/theme'
import { themeConfig } from '@/configs/theme.config'
import {
    DARK_MODE_OPTIONS,
    LIGHT_MODE_OPTIONS,
} from '@/constants/color-theme.constant'

type UseThemeReturnType = {
    setTheme: (theme: Theme) => void
    setSchema: (schema: string) => void
    setMode: (mode: Mode) => void
    setSelectedMode: (selectedMode: string) => void
    setSideNavCollapse: (sideNavCollapse: boolean) => void
    setDirection: (direction: Direction) => void
    setPanelExpand: (panelExpand: boolean) => void
    setLayout: (layout: LayoutType) => void
    setFontSettings: (settings: FontSettings) => void
    resetFontSettings: () => void
    setHighlightTheme: (highlightTheme: string, mode?: Theme['mode']) => void
} & Theme

const useTheme = <T>(selector: (state: UseThemeReturnType) => T): T => {
    const context = useContext(ThemeContext)

    if (context === undefined) {
        throw new Error('useTheme must be used under a ThemeProvider')
    }

    const getThemeState = () => ({
        ...context.theme,
        setTheme: (theme: Theme) => {
            const isDarkTheme = DARK_MODE_OPTIONS.some(
                (mode) => mode.value === theme.selectedMode,
            )
            context.setTheme((prevTheme) => ({
                ...prevTheme,
                ...theme,
                mode: isDarkTheme ? MODE_DARK : MODE_LIGHT,
            }))
            updateDocumentMode(theme.selectedMode)
        },
        setSchema: (themeSchema: string) => {
            context.setTheme((prevTheme) => ({ ...prevTheme, themeSchema }))
            applyTheme(themeSchema, context.theme.mode, presetThemeSchemaConfig)
        },
        setMode: (mode: Mode) => {
            context.setTheme((prevTheme) => ({ ...prevTheme, mode }))
            const root = window.document.documentElement
            const isEnabled = mode === MODE_DARK
            root.classList.remove(isEnabled ? MODE_LIGHT : MODE_DARK)
            root.classList.add(isEnabled ? MODE_DARK : MODE_LIGHT)
        },
        setSelectedMode: (selectedMode: string) => {
            const isDarkTheme = DARK_MODE_OPTIONS.some(
                (mode) => mode.value === selectedMode,
            )

            context.setTheme((prevTheme) => ({
                ...prevTheme,
                selectedMode,
                mode: isDarkTheme ? MODE_DARK : MODE_LIGHT,
            }))

            const root = window.document.documentElement
            root.classList.remove(isDarkTheme ? MODE_DARK : MODE_LIGHT)
            root.classList.add(isDarkTheme ? MODE_DARK : MODE_LIGHT)
        },
        setSideNavCollapse: (sideNavCollapse: boolean) => {
            context.setTheme((prevTheme) => ({
                ...prevTheme,
                layout: { ...prevTheme.layout, sideNavCollapse },
            }))
        },
        setDirection: (direction: Direction) => {
            context.setTheme((prevTheme) => ({ ...prevTheme, direction }))
            const root = window.document.documentElement
            root.setAttribute('dir', direction)
        },
        setPanelExpand: (panelExpand: boolean) => {
            context.setTheme((prevTheme) => ({ ...prevTheme, panelExpand }))
        },
        setLayout: (layout: LayoutType) => {
            context.setTheme((prevTheme) => ({
                ...prevTheme,
                layout: { ...prevTheme.layout, type: layout },
            }))
        },
        setFontSettings: (settings: FontSettings) => {
            context.setTheme((prevTheme) => ({
                ...prevTheme,
                fontSettings: settings,
            }))
        },
        resetFontSettings: () => {
            context.setTheme((prevTheme) => ({
                ...prevTheme,
                fontSettings: themeConfig.fontSettings,
            }))
        },
        setHighlightTheme: (highlightTheme: string, mode?: Theme['mode']) => {
            context.setTheme((prevTheme) => ({
                ...prevTheme,
                highlightTheme,
                mode: mode || prevTheme.mode,
            }))

            // Inject theme <link> into <head>
            if (typeof document !== 'undefined') {
                document.head
                    .querySelectorAll('link[data-highlight-theme]')
                    .forEach((link) => link.remove())

                const link = document.createElement('link')
                link.rel = 'stylesheet'
                link.href = `/highlight-js-themes/${highlightTheme}.css`
                link.setAttribute('data-highlight-theme', highlightTheme)
                document.head.appendChild(link)
            }
        },
    })

    const themeState = getThemeState()

    return selector(themeState)
}

const allModes = [...DARK_MODE_OPTIONS, ...LIGHT_MODE_OPTIONS].map(
    (mode) => mode.value,
)

// Also Automatically runs when theme changes in a useEffect placed in '/src/components/template/Theme/ThemeProvider.tsx'
export const updateDocumentMode = (selectedMode: string) => {
    if (typeof document === 'undefined') return

    const root = document.documentElement
    root.classList.remove(...allModes)
    root.classList.add(selectedMode)

    const isDarkTheme = DARK_MODE_OPTIONS.some(
        (mode) => mode.value === selectedMode,
    )

    if (isDarkTheme) {
        root.classList.add('dark')
    } else {
        root.classList.add('light')
    }
}

export default useTheme
