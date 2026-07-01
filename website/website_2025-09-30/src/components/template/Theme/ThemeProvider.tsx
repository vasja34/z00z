'use client'
import { useState, useEffect } from 'react'
import ThemeContext from './ThemeContext'
import ConfigProvider from '@/components/ui/ConfigProvider'
import appConfig from '@/configs/app.config'
import applyTheme from '@/utils/applyThemeSchema'
import { setTheme as setThemeCookies } from '@/server/actions/theme'
import presetThemeSchemaConfig from '@/configs/preset-theme-schema.config'
import type { Theme } from '@/@types/theme'
import type { CommonProps } from '@/@types/common'
import { updateDocumentMode } from '@/utils/hooks/useTheme'

interface ThemeProviderProps extends CommonProps {
    theme: Theme
    locale?: string
}

const ThemeProvider = ({ children, theme, locale }: ThemeProviderProps) => {
    const [themeState, setThemeState] = useState<Theme>(theme)

    const handleSetTheme = async (payload: (param: Theme) => Theme | Theme) => {
        const setTheme = async (theme: Theme) => {
            setThemeState(theme)
            await setThemeCookies(JSON.stringify({ state: theme }))
        }

        if (typeof payload === 'function') {
            const nextTheme = payload(themeState)
            await setTheme(nextTheme)
        } else {
            await setTheme(payload)
        }
    }

    // 👇 inject highlight theme CSS when highlightTheme changes
    useEffect(() => {
        if (!themeState.highlightTheme || typeof document === 'undefined')
            return

        // remove existing highlight theme links
        document.head
            .querySelectorAll('link[data-highlight-theme]')
            .forEach((link) => link.remove())

        // add new one
        const link = document.createElement('link')
        link.rel = 'stylesheet'
        link.href = `/highlight-js-themes/${themeState.highlightTheme}.css`
        link.setAttribute('data-highlight-theme', themeState.highlightTheme)
        document.head.appendChild(link)
    }, [themeState.highlightTheme])

    return (
        <ThemeContext.Provider
            value={{
                theme: themeState,
                setTheme: handleSetTheme,
            }}
        >
            <ConfigProvider
                value={{
                    ...theme,
                    locale: locale || appConfig.locale,
                }}
            >
                {children}
            </ConfigProvider>
            <script
                suppressHydrationWarning
                dangerouslySetInnerHTML={{
                    __html: `(${applyTheme.toString()})(${JSON.stringify([
                        theme.themeSchema || 'default',
                        theme.mode,
                        presetThemeSchemaConfig,
                    ]).slice(1, -1)})`,
                }}
            />
        </ThemeContext.Provider>
    )
}

export default ThemeProvider
