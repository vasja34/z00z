import { THEME_ENUM } from '@/constants/theme.constant'
import type { Theme } from '@/@types/theme'

/**
 * Since some configurations need to be match with specific themes,
 * we recommend to use the configuration that generated from demo.
 */
export const themeConfig: Theme = {
    themeSchema: 'orange',
    direction: THEME_ENUM.DIR_LTR,
    mode: THEME_ENUM.MODE_LIGHT,
    selectedMode: THEME_ENUM.MODE_LIGHT,
    panelExpand: false,
    controlSize: 'md',
    layout: {
        type: THEME_ENUM.LAYOUT_STACKED_SIDE,
        sideNavCollapse: false,
    },
    fontSettings: {
        h1: {
            fontSize: 36,
            fontWeight: 700,
            fontFamily: 'Inter',
        },
        h2: {
            fontSize: 30,
            fontWeight: 700,
            fontFamily: 'Inter',
        },
        h3: {
            fontSize: 24,
            fontWeight: 700,
            fontFamily: 'Inter',
        },
        h4: {
            fontSize: 20,
            fontWeight: 700,
            fontFamily: 'Inter',
        },
        h5: {
            fontSize: 18,
            fontWeight: 700,
            fontFamily: 'Inter',
        },
        h6: {
            fontSize: 16,
            fontWeight: 600,
            fontFamily: 'Inter',
        },
        p: {
            fontSize: 16,
            fontWeight: 400,
            fontFamily: 'Inter',
        },
    },
    highlightTheme: 'xcode',
}
