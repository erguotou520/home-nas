import React from 'react'
import { FlatList, Pressable } from 'react-native'
import { XStack, YStack, Text } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'

interface FileEntry {
    name: string
    path: string
    isDir: boolean
    size?: number
    modified?: number
    mimeType?: string
}

interface FileListProps {
    files: FileEntry[]
    onFilePress: (file: FileEntry) => void
    onFileLongPress?: (file: FileEntry) => void
    selectedFiles?: string[]
}

export function FileList({
    files,
    onFilePress,
    onFileLongPress,
    selectedFiles = [],
}: FileListProps) {
    const renderItem = ({ item }: { item: FileEntry }) => {
        const isSelected = selectedFiles.includes(item.path)

        return (
            <Pressable
                onPress={() => onFilePress(item)}
                onLongPress={() => onFileLongPress?.(item)}
            >
                <XStack
                    paddingVertical="$3"
                    paddingHorizontal="$4"
                    alignItems="center"
                    gap="$3"
                    backgroundColor={isSelected ? '$primary' : 'transparent'}
                    borderBottomWidth={1}
                    borderBottomColor="$borderColor"
                >
                    {/* Icon */}
                    <YStack
                        width={44}
                        height={44}
                        alignItems="center"
                        justifyContent="center"
                        backgroundColor={isSelected ? '$primaryDark' : '$surface'}
                        borderRadius="$2"
                    >
                        <Ionicons
                            name={getFileIcon(item)}
                            size={24}
                            color={isSelected ? '#fff' : '#6366f1'}
                        />
                    </YStack>

                    {/* Info */}
                    <YStack flex={1}>
                        <Text
                            fontSize="$4"
                            color={isSelected ? '#fff' : '$text'}
                            numberOfLines={1}
                        >
                            {item.name}
                        </Text>
                        <XStack gap="$2">
                            {item.size && (
                                <Text fontSize="$2" color={isSelected ? 'rgba(255,255,255,0.7)' : '$textMuted'}>
                                    {formatFileSize(item.size)}
                                </Text>
                            )}
                            {item.modified && (
                                <Text fontSize="$2" color={isSelected ? 'rgba(255,255,255,0.7)' : '$textMuted'}>
                                    {formatDate(item.modified)}
                                </Text>
                            )}
                        </XStack>
                    </YStack>

                    {/* Arrow or checkbox */}
                    {isSelected ? (
                        <Ionicons name="checkmark-circle" size={24} color="#fff" />
                    ) : item.isDir ? (
                        <Ionicons name="chevron-forward" size={20} color="#94a3b8" />
                    ) : null}
                </XStack>
            </Pressable>
        )
    }

    return (
        <FlatList
            data={files}
            renderItem={renderItem}
            keyExtractor={(item) => item.path}
            showsVerticalScrollIndicator={false}
        />
    )
}

function getFileIcon(file: FileEntry): keyof typeof Ionicons.glyphMap {
    if (file.isDir) return 'folder'

    const mime = file.mimeType || ''
    const name = file.name.toLowerCase()

    if (mime.startsWith('image/')) return 'image'
    if (mime.startsWith('video/')) return 'videocam'
    if (mime.startsWith('audio/')) return 'musical-notes'
    if (mime.includes('pdf') || name.endsWith('.pdf')) return 'document-text'
    if (name.endsWith('.doc') || name.endsWith('.docx')) return 'document-text'
    if (name.endsWith('.xls') || name.endsWith('.xlsx')) return 'grid'
    if (name.endsWith('.ppt') || name.endsWith('.pptx')) return 'easel'
    if (name.endsWith('.md') || name.endsWith('.txt')) return 'document'
    if (mime.includes('zip') || mime.includes('rar') || name.endsWith('.zip')) return 'archive'

    return 'document'
}

function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
    return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`
}

function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000)
    return date.toLocaleDateString()
}
