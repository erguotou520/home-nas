import React from 'react'
import { FlatList, Pressable, Image } from 'react-native'
import { XStack, YStack, Text, Card } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { getThumbnailUrl } from '../api'

interface FileEntry {
    name: string
    path: string
    isDir: boolean
    size?: number
    mimeType?: string
    thumbnail?: string
    metadata?: {
        title?: string
        poster?: string
        year?: string
    }
}

interface FileGridProps {
    files: FileEntry[]
    onFilePress: (file: FileEntry) => void
    onFileLongPress?: (file: FileEntry) => void
    selectedFiles?: string[]
    numColumns?: number
}

export function FileGrid({
    files,
    onFilePress,
    onFileLongPress,
    selectedFiles = [],
    numColumns = 3,
}: FileGridProps) {
    const renderItem = ({ item }: { item: FileEntry }) => {
        const isSelected = selectedFiles.includes(item.path)
        const thumbnailUrl = item.metadata?.poster || item.thumbnail

        return (
            <Pressable
                onPress={() => onFilePress(item)}
                onLongPress={() => onFileLongPress?.(item)}
                style={{ flex: 1 / numColumns, padding: 4 }}
            >
                <Card
                    flex={1}
                    aspectRatio={1}
                    backgroundColor={isSelected ? '$primary' : '$surface'}
                    borderRadius="$3"
                    overflow="hidden"
                    borderWidth={isSelected ? 2 : 0}
                    borderColor="$primary"
                >
                    {/* Thumbnail or Icon */}
                    {thumbnailUrl ? (
                        <Image
                            source={{ uri: getThumbnailUrl(thumbnailUrl) }}
                            style={{ width: '100%', height: '70%' }}
                            resizeMode="cover"
                        />
                    ) : (
                        <YStack flex={1} alignItems="center" justifyContent="center" height="70%">
                            <Ionicons
                                name={getFileIcon(item)}
                                size={40}
                                color={isSelected ? '#fff' : '#94a3b8'}
                            />
                        </YStack>
                    )}

                    {/* Name */}
                    <YStack padding="$2" flex={1} justifyContent="flex-end">
                        <Text
                            fontSize="$2"
                            color={isSelected ? '#fff' : '$text'}
                            numberOfLines={2}
                            textAlign="center"
                        >
                            {item.metadata?.title || item.name}
                        </Text>
                        {item.metadata?.year && (
                            <Text fontSize="$1" color="$textMuted" textAlign="center">
                                {item.metadata.year}
                            </Text>
                        )}
                    </YStack>

                    {/* Selection indicator */}
                    {isSelected && (
                        <YStack
                            position="absolute"
                            top="$2"
                            right="$2"
                            backgroundColor="$primary"
                            borderRadius="$10"
                            padding="$1"
                        >
                            <Ionicons name="checkmark" size={14} color="#fff" />
                        </YStack>
                    )}
                </Card>
            </Pressable>
        )
    }

    return (
        <FlatList
            data={files}
            renderItem={renderItem}
            keyExtractor={(item) => item.path}
            numColumns={numColumns}
            contentContainerStyle={{ padding: 8 }}
            showsVerticalScrollIndicator={false}
        />
    )
}

function getFileIcon(file: FileEntry): keyof typeof Ionicons.glyphMap {
    if (file.isDir) return 'folder'

    const mime = file.mimeType || ''
    if (mime.startsWith('image/')) return 'image'
    if (mime.startsWith('video/')) return 'videocam'
    if (mime.startsWith('audio/')) return 'musical-notes'
    if (mime.includes('pdf')) return 'document-text'
    if (mime.includes('zip') || mime.includes('rar')) return 'archive'

    return 'document'
}
