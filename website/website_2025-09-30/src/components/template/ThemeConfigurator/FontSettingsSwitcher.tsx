'use client'

import React, { useEffect, useState } from 'react'
import useTheme from '@/utils/hooks/useTheme'
import { FormItem, Form } from '@/components/ui/Form'
import { useForm, Controller } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import type { ZodType } from 'zod'
import { FontSettings } from '@/@types/theme'
import { Button, Input, Select } from '@/components/ui'
import {
    FONT_TYPE_OPTIONS,
    FONT_WEIGHT_OPTIONS,
    FONTS_FAMILY_OPTIONS,
} from '@/constants/font-constant'
import { BiChevronDown, BiChevronUp } from 'react-icons/bi'
import { AnimatePresence, motion } from 'framer-motion'

const fontSettingsSchema = z.object({
    fontSize: z.number(),
    fontWeight: z.number(),
    fontFamily: z.string().optional(),
})

const validationSchema: ZodType<FontSettings> = z.object({
    h1: fontSettingsSchema,
    h2: fontSettingsSchema,
    h3: fontSettingsSchema,
    h4: fontSettingsSchema,
    h5: fontSettingsSchema,
    h6: fontSettingsSchema,
    p: fontSettingsSchema,
})

type FontOption = { label: string; value: string }

const FontSettingsSwitcher = () => {
    const [isOpen, setIsOpen] = useState(false)
    const [selectedHeaderType, setSelectedHeaderType] = useState('h1')

    const fontSettings = useTheme((state) => state.fontSettings)
    const setFontSettings = useTheme((state) => state.setFontSettings)
    // const resetFontSettings = useTheme((state) => state.resetFontSettings)

    const {
        formState: { errors, isSubmitting },
        control,
        handleSubmit,
    } = useForm<FontSettings>({
        resolver: zodResolver(validationSchema),
        values: fontSettings,
    })

    const onFormSubmit = (values: FontSettings) => {
        setFontSettings(values)
    }

    // Load selected font dynamically
    useEffect(() => {
        const font =
            fontSettings[selectedHeaderType as keyof FontSettings]?.fontFamily
        if (!font) return

        const id = `gf-${font}`
        if (document.getElementById(id)) return

        const link = document.createElement('link')
        link.id = id
        link.rel = 'stylesheet'
        link.href = `https://fonts.googleapis.com/css2?family=${font.replace(
            / /g,
            '+',
        )}:wght@300;400;600;700&display=swap`

        document.head.appendChild(link)
    }, [fontSettings, selectedHeaderType])

    return (
        <div>
            <button
                type="button"
                onClick={() => setIsOpen((s) => !s)}
                className="w-full font-semibold flex justify-between items-center bg-select-background rounded-[10px] border border-transparent dark:border-select-border px-3 py-2.5"
                aria-expanded={isOpen}
            >
                <h6 className="text-select-text">Font Settings</h6>
                {isOpen ? (
                    <BiChevronUp className="w-6.5 h-6.5" />
                ) : (
                    <BiChevronDown className="w-6.5 h-6.5" />
                )}
            </button>

            <AnimatePresence initial={false}>
                {isOpen && (
                    <motion.div
                        key="font-settings-panel"
                        className="!overflow-hidden pt-4 pb-2 px-[3px]"
                        initial={{ height: 0, overflow: 'hidden' }}
                        animate={{ height: 'auto', overflow: 'visible' }}
                        exit={{ height: 0, overflow: 'hidden' }}
                        transition={{ duration: 0.25, ease: 'easeOut' }}
                    >
                        <Form onSubmit={handleSubmit(onFormSubmit)}>
                            <div className="flex flex-col gap-5">
                                {/* Header Type */}
                                <div className="w-full grid grid-cols-3 items-center gap-4">
                                    <label className="col-span-1 text-[13px] leading-none font-semibold">
                                        Header Type
                                    </label>
                                    <Select<{ label: string; value: string }>
                                        className="col-span-2 !h-10 !max-h-10 cursor-pointer text-[13px] leading-none"
                                        size="sm"
                                        menuPlacement="auto"
                                        isSearchable={false}
                                        value={{
                                            label:
                                                FONT_TYPE_OPTIONS.find(
                                                    (option) =>
                                                        option.value ===
                                                        selectedHeaderType,
                                                )?.label || 'H1',
                                            value: selectedHeaderType,
                                        }}
                                        options={FONT_TYPE_OPTIONS}
                                        onChange={(option) =>
                                            setSelectedHeaderType(
                                                option?.value || 'h1',
                                            )
                                        }
                                    />
                                </div>

                                {/* Font Family */}
                                <FormItem
                                    className="grid grid-cols-3 gap-4 [&>div]:col-span-2 !mb-0"
                                    label="Font Family"
                                    labelClass="col-span-1 text-[13px] leading-none font-semibold !mb-0"
                                    errorMessage={
                                        (errors as any)[selectedHeaderType]
                                            ?.message
                                    }
                                >
                                    <Controller
                                        key={selectedHeaderType}
                                        name={
                                            selectedHeaderType as keyof FontSettings
                                        }
                                        control={control}
                                        render={({ field }) => (
                                            <Select<FontOption>
                                                className="col-span-2 cursor-pointer text-[13px] leading-none"
                                                size="sm"
                                                classNames={{
                                                    menuList: () =>
                                                        '!max-h-[180px]',
                                                }}
                                                menuPlacement="auto"
                                                isSearchable
                                                value={{
                                                    label:
                                                        field.value
                                                            .fontFamily ||
                                                        'Inter',
                                                    value:
                                                        field.value
                                                            .fontFamily ||
                                                        'Inter',
                                                }}
                                                options={FONTS_FAMILY_OPTIONS}
                                                onChange={(option) =>
                                                    field.onChange({
                                                        ...field.value,
                                                        fontFamily:
                                                            option?.value ||
                                                            'Inter',
                                                    })
                                                }
                                            />
                                        )}
                                    />
                                </FormItem>

                                {/* Font Size */}
                                <FormItem
                                    className="w-full !grid !grid-cols-3 items-center gap-4 [&>div]:col-span-2 !mb-0"
                                    label={'Font Size'}
                                    labelClass="col-span-1 text-[13px] leading-none font-semibold !mb-0"
                                    errorMessage={
                                        (errors as any)[selectedHeaderType]
                                            ?.message
                                    }
                                >
                                    <Controller
                                        key={selectedHeaderType}
                                        name={
                                            selectedHeaderType as keyof FontSettings
                                        }
                                        control={control}
                                        render={({ field }) => (
                                            <Input
                                                type="number"
                                                size="sm"
                                                min={0}
                                                className="w-full col-span-2 text-[13px] leading-none"
                                                placeholder={`Set ${String(
                                                    selectedHeaderType,
                                                ).toUpperCase()} font size`}
                                                invalid={Boolean(
                                                    (errors as any)[
                                                        selectedHeaderType
                                                    ],
                                                )}
                                                {...field}
                                                value={String(
                                                    field.value.fontSize ?? '',
                                                )}
                                                onChange={(e) =>
                                                    field.onChange({
                                                        ...field.value,
                                                        fontSize: Number(
                                                            e.target.value,
                                                        ),
                                                    })
                                                }
                                            />
                                        )}
                                    />
                                </FormItem>

                                {/* Font Weight */}
                                <FormItem
                                    className="grid grid-cols-3 gap-4 [&>div]:col-span-2 !mb-0"
                                    label="Font Style"
                                    labelClass="col-span-1 text-[13px] leading-none font-semibold !mb-0"
                                    errorMessage={
                                        (errors as any)[selectedHeaderType]
                                            ?.message
                                    }
                                >
                                    <Controller
                                        key={selectedHeaderType}
                                        name={
                                            selectedHeaderType as keyof FontSettings
                                        }
                                        control={control}
                                        render={({ field }) => (
                                            <Select<{
                                                label: string
                                                value: number
                                            }>
                                                className="col-span-2 cursor-pointer text-[13px] leading-none"
                                                size="sm"
                                                menuPlacement="top"
                                                isSearchable={false}
                                                value={{
                                                    label:
                                                        FONT_WEIGHT_OPTIONS.find(
                                                            (option) =>
                                                                option.value ===
                                                                field.value
                                                                    .fontWeight,
                                                        )?.label || '',
                                                    value: field.value
                                                        .fontWeight,
                                                }}
                                                options={FONT_WEIGHT_OPTIONS}
                                                onChange={(option) =>
                                                    field.onChange({
                                                        ...field.value,
                                                        fontWeight:
                                                            option?.value,
                                                    })
                                                }
                                            />
                                        )}
                                    />
                                </FormItem>
                            </div>

                            {/* Actions */}
                            <div className="flex flex-col gap-2 mt-5">
                                <Button
                                    block
                                    loading={isSubmitting}
                                    variant="solid"
                                    type="submit"
                                    size="sm"
                                >
                                    {isSubmitting ? 'Applying...' : 'Apply'}
                                </Button>
                                <Button
                                    block
                                    loading={isSubmitting}
                                    variant="default"
                                    type="button"
                                    onClick={() => {
                                        setIsOpen(false)
                                    }}
                                    size="sm"
                                >
                                    Close
                                </Button>
                            </div>
                        </Form>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    )
}

export default FontSettingsSwitcher
