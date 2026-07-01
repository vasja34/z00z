export type Direction = 'ltr' | 'rtl'
export type Mode = 'light' | 'dark'
export type ControlSize = 'lg' | 'md' | 'sm'
export type LayoutType =
    | 'blank'
    | 'collapsibleSide'
    | 'stackedSide'
    | 'topBarClassic'
    | 'framelessSide'
    | 'contentOverlay'

type fontSetting = {
    fontSize: number
    fontWeight: number
    fontFamily?: string
}
export type FontSettings = {
    h1: fontSetting
    h2: fontSetting
    h3: fontSetting
    h4: fontSetting
    h5: fontSetting
    h6: fontSetting
    p: fontSetting
}

export type Theme = {
    themeSchema: string
    direction: Direction
    mode: Mode
    selectedMode: string
    panelExpand: boolean
    controlSize: ControlSize
    layout: {
        type: LayoutType
        sideNavCollapse: boolean
        previousType?: LayoutType | ''
    }
    fontSettings: FontSettings
    highlightTheme: string
}
