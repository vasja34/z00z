'use client'

import StackedSideNav from '@/components/template/StackedSideNav'
import Header from '@/components/template/Header'
import MobileNav from '@/components/template/MobileNav'
import Search from '@/components/template/Search'
import LanguageSelector from '@/components/template/LanguageSelector'
import Notification from '@/components/template/Notification'
import SidePanel from '@/components//template/SidePanel'
import LayoutBase from '@/components//template/LayoutBase'
import { LAYOUT_STACKED_SIDE } from '@/constants/theme.constant'
import type { CommonProps } from '@/@types/common'
import Link from 'next/link'
import { FaGithub } from 'react-icons/fa'

const StackedSide = ({ children }: CommonProps) => {
    return (
        <LayoutBase
            type={LAYOUT_STACKED_SIDE}
            className="app-layout-stacked-side flex flex-auto flex-col"
        >
            <div className="flex flex-auto min-w-0">
                <StackedSideNav />
                <div className="flex flex-col flex-auto min-h-screen min-w-0 relative w-full">
                    <Header
                        className="shadow-sm dark:shadow-2xl"
                        headerStart={
                            <>
                                <MobileNav />
                            </>
                        }
                        headerEnd={
                            <>
                                {/* <LanguageSelector />
                                <Notification /> */}
                                <Search />
                                <Link
                                    href="https://github.com/vasja34/z00z"
                                    className="text-2xl"
                                    target="_blank"
                                    rel="noopener"
                                >
                                    <FaGithub />
                                </Link>
                                <SidePanel />
                            </>
                        }
                    />
                    <div className="h-full flex flex-auto flex-col">
                        {children}
                    </div>
                </div>
            </div>
        </LayoutBase>
    )
}

export default StackedSide
