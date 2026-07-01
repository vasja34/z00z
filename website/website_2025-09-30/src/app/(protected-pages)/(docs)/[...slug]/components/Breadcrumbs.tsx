'use client'

import React from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { BiChevronRight } from 'react-icons/bi'
import { IoMdHome } from 'react-icons/io'
import useNavigation from '@/utils/hooks/useNavigation'
import { findBreadcrumbs } from './utils'

const Breadcrumbs: React.FC = () => {
    const pathname = usePathname()
    const { navigationTree } = useNavigation()

    const breadcrumbs = findBreadcrumbs(navigationTree, pathname) || []

    return (
        <div className="w-full flex flex-wrap items-center gap-1.5 text-sm">
            {/* Home link */}
            <Link href="/" className="flex items-center gap-1 -mt-[1px]">
                <IoMdHome
                    size={18}
                    className="fill-black dark:fill-white hover:fill-primary"
                />
            </Link>

            {breadcrumbs.map((crumb, idx) => {
                const isLast = idx === breadcrumbs.length - 1
                const hasPath = crumb.path && crumb.path.trim() !== ''

                return (
                    <React.Fragment key={crumb.key}>
                        <BiChevronRight size={16} className="mt-0.5" />
                        {isLast ? (
                            <span className="text-primary">{crumb.title}</span>
                        ) : hasPath ? (
                            <Link
                                href={crumb.path}
                                className="hover:text-primary"
                            >
                                {crumb.title}
                            </Link>
                        ) : (
                            <span>{crumb.title}</span>
                        )}
                    </React.Fragment>
                )
            })}
        </div>
    )
}

export { Breadcrumbs }
