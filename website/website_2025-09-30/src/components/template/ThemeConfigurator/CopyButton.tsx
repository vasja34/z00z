import Button from '@/components/ui/Button'
import { themeConfig } from '@/configs/theme.config'
import useTheme from '@/utils/hooks/useTheme'

const CopyButton = () => {
    const theme = useTheme((state) => state)

    const handleSave = () => {
        const config = {
            ...themeConfig,
            ...theme,
            layout: {
                type: theme.layout.type,
                sideNavCollapse: theme.layout.sideNavCollapse,
            },
            panelExpand: false,
        }

        const blob = new Blob([JSON.stringify(config, null, 2)], {
            type: 'application/json',
        })
        const url = URL.createObjectURL(blob)

        // Create a hidden link for download
        const a = document.createElement('a')
        a.href = url
        a.download = 'config.json'

        // Append, trigger click, then cleanup
        document.body.appendChild(a)
        a.click()
        document.body.removeChild(a)

        // Release the object URL
        URL.revokeObjectURL(url)
    }

    return (
        <Button block variant="solid" onClick={handleSave}>
            Save Config
        </Button>
    )
}

export default CopyButton
