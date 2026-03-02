import React, { useState, useCallback } from 'react'
import { Platform, Alert, Linking } from 'react-native'
import { YStack, XStack, Text, Button, Spinner } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { useFocusEffect } from '@react-navigation/native'
import * as FileSystem from 'expo-file-system'
import * as Sharing from 'expo-sharing'

import { FileGrid, FileList, ShareDialog } from '../components'
import { useFiles, useDeleteFile, getStreamUrl } from '../api'
import { useFileStore } from '../stores'

interface BrowserScreenProps {
    route: any
    navigation: any
}

export function BrowserScreen({ route, navigation }: BrowserScreenProps) {
    const { app } = route.params || { app: 'medias' }

    const {
        currentPath,
        selectedFiles,
        isGridView,
        setCurrentPath,
        setFiles,
        toggleSelection,
        clearSelection,
        setGridView,
        navigateUp,
    } = useFileStore()

    const [shareFile, setShareFile] = useState<{ path: string; name: string } | null>(null)

    const { data, isLoading, error, refetch } = useFiles(app, currentPath)
    const deleteFile = useDeleteFile()

    // Update files in store when data changes
    useFocusEffect(
        useCallback(() => {
            if (data?.entries) {
                setFiles(data.entries)
            }
        }, [data])
    )

    const handleFilePress = (file: any) => {
        if (selectedFiles.length > 0) {
            toggleSelection(file.path)
            return
        }

        if (file.isDir) {
            setCurrentPath(file.path)
            return
        }

        // Open file based on type
        const mime = file.mimeType || ''

        if (mime.startsWith('image/')) {
            navigation.navigate('ImageViewer', { path: file.path })
        } else if (mime.startsWith('video/')) {
            navigation.navigate('VideoPlayer', { path: file.path })
        } else if (mime.startsWith('audio/')) {
            navigation.navigate('MusicPlayer', { path: file.path })
        } else if (mime.includes('pdf')) {
            navigation.navigate('PdfViewer', { path: file.path })
        } else {
            // For other files, open with system handler on app, download on web
            handleOpenFile(file)
        }
    }

    const handleOpenFile = async (file: any) => {
        if (Platform.OS === 'web') {
            // Download on web
            const url = getStreamUrl(file.path)
            Linking.openURL(url)
        } else {
            // Open with system on native
            try {
                const url = getStreamUrl(file.path)
                const canOpen = await Linking.canOpenURL(url)
                if (canOpen) {
                    await Linking.openURL(url)
                } else {
                    Alert.alert('无法打开', '未找到可以打开此文件的应用')
                }
            } catch (error) {
                Alert.alert('错误', '打开文件失败')
            }
        }
    }

    const handleFileLongPress = (file: any) => {
        toggleSelection(file.path)
    }

    const handleDelete = async () => {
        if (selectedFiles.length === 0) return

        Alert.alert(
            '确认删除',
            `确定要删除选中的 ${selectedFiles.length} 个文件吗？`,
            [
                { text: '取消', style: 'cancel' },
                {
                    text: '删除',
                    style: 'destructive',
                    onPress: async () => {
                        try {
                            for (const path of selectedFiles) {
                                await deleteFile.mutateAsync({ app, path })
                            }
                            clearSelection()
                            refetch()
                        } catch (error: any) {
                            Alert.alert('删除失败', error.message)
                        }
                    },
                },
            ]
        )
    }

    const handleShare = () => {
        if (selectedFiles.length === 1) {
            const file = data?.entries.find(f => f.path === selectedFiles[0])
            if (file) {
                setShareFile({ path: file.path, name: file.name })
            }
        }
    }

    // Show grid for media/videos, list for documents
    const showGrid = isGridView && (app === 'medias' || app === 'videos')

    return (
        <YStack flex={1} backgroundColor="$background">
            {/* Header */}
            <XStack
                paddingHorizontal="$4"
                paddingVertical="$3"
                alignItems="center"
                justifyContent="space-between"
                borderBottomWidth={1}
                borderBottomColor="$borderColor"
            >
                <XStack alignItems="center" gap="$3">
                    {currentPath && (
                        <Button size="$3" chromeless onPress={navigateUp}>
                            <Ionicons name="arrow-back" size={24} color="#6366f1" />
                        </Button>
                    )}
                    <YStack>
                        <Text fontSize="$5" fontWeight="bold" color="$text">
                            {getAppName(app)}
                        </Text>
                        {currentPath && (
                            <Text fontSize="$2" color="$textMuted" numberOfLines={1}>
                                {currentPath}
                            </Text>
                        )}
                    </YStack>
                </XStack>

                <XStack gap="$2">
                    {/* View toggle for media */}
                    {(app === 'medias' || app === 'videos') && (
                        <Button
                            size="$3"
                            chromeless
                            onPress={() => setGridView(!isGridView)}
                        >
                            <Ionicons
                                name={isGridView ? 'list' : 'grid'}
                                size={20}
                                color="#94a3b8"
                            />
                        </Button>
                    )}
                </XStack>
            </XStack>

            {/* Selection Actions */}
            {selectedFiles.length > 0 && (
                <XStack
                    backgroundColor="$surface"
                    paddingHorizontal="$4"
                    paddingVertical="$3"
                    alignItems="center"
                    justifyContent="space-between"
                >
                    <XStack alignItems="center" gap="$3">
                        <Button size="$3" chromeless onPress={clearSelection}>
                            <Ionicons name="close" size={20} color="#94a3b8" />
                        </Button>
                        <Text color="$text">
                            已选择 {selectedFiles.length} 项
                        </Text>
                    </XStack>

                    <XStack gap="$2">
                        {selectedFiles.length === 1 && (
                            <Button size="$3" chromeless onPress={handleShare}>
                                <Ionicons name="share-outline" size={20} color="#6366f1" />
                            </Button>
                        )}
                        <Button size="$3" chromeless onPress={handleDelete}>
                            <Ionicons name="trash-outline" size={20} color="#ef4444" />
                        </Button>
                    </XStack>
                </XStack>
            )}

            {/* Content */}
            {isLoading ? (
                <YStack flex={1} alignItems="center" justifyContent="center">
                    <Spinner size="large" color="$primary" />
                </YStack>
            ) : error ? (
                <YStack flex={1} alignItems="center" justifyContent="center" padding="$4">
                    <Ionicons name="alert-circle" size={48} color="#ef4444" />
                    <Text color="$text" marginTop="$3">加载失败</Text>
                    <Button marginTop="$3" onPress={() => refetch()}>
                        重试
                    </Button>
                </YStack>
            ) : data?.entries.length === 0 ? (
                <YStack flex={1} alignItems="center" justifyContent="center">
                    <Ionicons name="folder-open" size={48} color="#94a3b8" />
                    <Text color="$textMuted" marginTop="$3">文件夹为空</Text>
                </YStack>
            ) : showGrid ? (
                <FileGrid
                    files={data?.entries || []}
                    onFilePress={handleFilePress}
                    onFileLongPress={handleFileLongPress}
                    selectedFiles={selectedFiles}
                />
            ) : (
                <FileList
                    files={data?.entries || []}
                    onFilePress={handleFilePress}
                    onFileLongPress={handleFileLongPress}
                    selectedFiles={selectedFiles}
                />
            )}

            {/* Share Dialog */}
            {shareFile && (
                <ShareDialog
                    visible={!!shareFile}
                    filePath={shareFile.path}
                    appType={app}
                    fileName={shareFile.name}
                    onClose={() => {
                        setShareFile(null)
                        clearSelection()
                    }}
                />
            )}
        </YStack>
    )
}

function getAppName(app: string): string {
    switch (app) {
        case 'medias': return '媒体库'
        case 'videos': return '视频库'
        case 'music': return '音乐库'
        case 'documents': return '文档库'
        default: return '文件'
    }
}
