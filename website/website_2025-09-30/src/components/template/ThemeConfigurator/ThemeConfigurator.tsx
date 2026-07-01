import ModeSwitcher from './ModeSwitcher'
import LayoutSwitcher from './LayoutSwitcher'
import ThemeSwitcher from './ThemeSwitcher'
import DirectionSwitcher from './DirectionSwitcher'
import CopyButton from './CopyButton'
import FontSettingsSwitcher from './FontSettingsSwitcher'
import HighlightThemeSwitcher from './HighlightThemeSwitcher'
import LoadButton from './LoadButton'

const ThemeConfigurator = () => {
    return (
        <div className="flex flex-col h-full justify-between">
            <div className="flex-grow flex flex-col">
                <div className="flex items-center justify-between mb-10">
                    <div>
                        <h6>Direction</h6>
                        <span>Select a direction</span>
                    </div>
                    <DirectionSwitcher />
                </div>
                <div className="mb-10">
                    <h6 className="mb-3">Color Theme</h6>
                    <ModeSwitcher />
                </div>
                <div className="mb-10">
                    <h6 className="mb-3">UI Colors</h6>
                    <ThemeSwitcher />
                </div>
                <div className="mb-7">
                    <h6 className="mb-3">Code Theme</h6>
                    <HighlightThemeSwitcher />
                </div>
                <div className="mb-7">
                    <h6 className="mb-3">Typography</h6>
                    <FontSettingsSwitcher />
                </div>
                <div className="pb-10">
                    <h6 className="mb-3">Layout</h6>
                    <LayoutSwitcher />
                </div>
            </div>
            <div className="flex flex-col gap-4 pb-4 -mb-4">
                <CopyButton />
                <LoadButton />
            </div>
        </div>
    )
}

export default ThemeConfigurator
