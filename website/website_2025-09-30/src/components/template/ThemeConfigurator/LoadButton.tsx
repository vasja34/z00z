'use client'

import Notification from '@/components/ui/Notification'
import Button from '@/components/ui/Button'
import toast from '@/components/ui/toast'
import { themeConfig } from '@/configs/theme.config'
import useTheme from '@/utils/hooks/useTheme'
import { useState } from 'react'
import { Theme } from '@/@types/theme'

const LoadButton = () => {
    const [isLoading, setIsLoading] = useState(false)
    const state = useTheme((state) => state)

    const handleFileSelect = async (event: Event) => {
        const input = event.target as HTMLInputElement
        const file = input.files?.[0]
        if (!file) return

        setIsLoading(true)
        try {
            const text = await file.text()

            let config
            try {
                config = JSON.parse(text)
            } catch {
                const cleaned = text
                    .replace(/export\s+default/, '')
                    .replace(/;$/, '')
                    .trim()
                config = eval(`(${cleaned})`)
            }

            if (config) {
                const newConfig: Theme = {
                    ...themeConfig,
                    ...config,
                }
                state.setSchema(newConfig.themeSchema)
                state.setMode(newConfig.mode)
                state.setSelectedMode(newConfig.selectedMode)
                state.setSideNavCollapse(newConfig.layout.sideNavCollapse)
                state.setDirection(newConfig.direction)
                state.setPanelExpand(newConfig.panelExpand)
                state.setLayout(newConfig.layout.type)
                state.setFontSettings(newConfig.fontSettings)
                state.setHighlightTheme(newConfig.highlightTheme, newConfig.mode)
                state.setTheme(newConfig)
                toast.push(
                    <Notification title="Load Success" type="success">
                        Configuration loaded successfully.
                    </Notification>,
                    { placement: 'top-center' },
                )
            }
        } catch (error) {
            toast.push(
                <Notification title="Load Failed" type="danger">
                    Error loading configuration. Please check the file and try again.
                </Notification>,
                { placement: 'top-center' },
            )
        } finally {
            setIsLoading(false)
            input.value = '' // safe reset
        }
    }

    const openFileDialog = () => {
        const input = document.createElement('input')
        input.type = 'file'
        // input.accept = '.json,.txt,.js,.ts'
        input.onchange = handleFileSelect
        input.click()
    }

    return (
        <Button
            block
            variant="default"
            loading={isLoading}
            onClick={openFileDialog}
        >
            Load Config
        </Button>
    )
}

export default LoadButton
