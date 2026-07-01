'use client'

import Link from 'next/link'
import { Button } from '@/components/ui'
import useNavigation from '@/utils/hooks/useNavigation'
import { usePathname } from 'next/navigation'
import { BiChevronsLeft, BiChevronsRight } from 'react-icons/bi'
import { flattenNav } from './utils'

const NavigationButtons: React.FC = () => {
    const pathname = usePathname()
    const { navigationTree } = useNavigation()

    const flatNav = flattenNav(navigationTree)
    const currentIndex = flatNav.findIndex((nav) => nav.path === pathname)

    const prev = currentIndex > 0 ? flatNav[currentIndex - 1] : null
    const next =
        currentIndex < flatNav.length - 1 ? flatNav[currentIndex + 1] : null

    return (
        <nav className={`w-full flex flex-wrap items-center gap-4 ${!prev ? "justify-end" : "justify-start"}`}>
            {prev && (
                <Link href={prev.path} className="flex-1 xs:max-w-[50%]">
                    <Button
                        name="previous"
                        className="w-full !h-auto flex flex-col items-start gap-[2px] dark:bg-transparent dark:ring-transparent dark:hover:border-primary rounded-md py-2.5 lg:px-5 px-4"
                    >
                        <span className="font-semibold text-gray-500 dark:text-gray-400">
                            Previous
                        </span>
                        <span className="flex items-center font-semibold text-primary-mild gap-0.5">
                            <BiChevronsLeft className="mt-[1px]" /> {prev.title}
                        </span>
                    </Button>
                </Link>
            )}
            {next && (
                <Link href={next.path} className="flex-1 xs:max-w-[50%]">
                    <Button
                        name="next"
                        className="w-full !h-auto flex flex-col items-end gap-[2px] dark:bg-transparent dark:ring-transparent dark:hover:border-primary rounded-md py-2.5 lg:px-5 px-4"
                    >
                        <span className="font-semibold text-gray-500 dark:text-gray-400">
                            Next
                        </span>
                        <span className="flex items-center font-semibold text-primary-mild gap-0.5">
                            {next.title}{' '}
                            <BiChevronsRight className="mt-[1px]" />
                        </span>
                    </Button>
                </Link>
            )}
        </nav>
    )
}

export { NavigationButtons }
