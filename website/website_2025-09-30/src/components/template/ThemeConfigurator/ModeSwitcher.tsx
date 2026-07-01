'use client'

import useTheme from '@/utils/hooks/useTheme'
import { Select } from '@/components/ui'
import {
    DARK_MODE_OPTIONS,
    LIGHT_MODE_OPTIONS,
} from '@/constants/color-theme.constant'

type ModeOption = { label: string; value: string }

const ModeSwitcher = () => {
    const selectedMode = useTheme((state) => state.selectedMode)
    const setSelecteMode = useTheme((state) => state.setSelectedMode)

    const onModeChange = (option: string) => {
        setSelecteMode(option)
    }

    return (
        <div>
            <Select<ModeOption>
                className="w-full"
                size="md"
                menuPlacement="auto"
                isSearchable={false}
                options={MODE_OPTIONS}
                value={{
                    label:
                        ALL_MODE_OPTIONS.find((o) => o.value === selectedMode)
                            ?.label || 'Light',
                    value: selectedMode || 'light',
                }}
                onChange={(option) => onModeChange(option?.value || 'light')}
            />
        </div>
    )
}

const MODE_OPTIONS = [
    { label: 'Light Themes', options: LIGHT_MODE_OPTIONS },
    { label: 'Dark Themes', options: DARK_MODE_OPTIONS },
]

const ALL_MODE_OPTIONS = [...DARK_MODE_OPTIONS, ...LIGHT_MODE_OPTIONS]

export default ModeSwitcher
