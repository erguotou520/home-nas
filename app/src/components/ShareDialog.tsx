import React, { useState } from 'react'
import { Modal, Platform, Alert } from 'react-native'
import { YStack, XStack, Text, Button, Switch, Input, Sheet } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { useCreateShare } from '../api'
import * as Clipboard from 'expo-clipboard'
import * as Sharing from 'expo-sharing'

interface ShareDialogProps {
    visible: boolean
    filePath: string
    appType: string
    fileName: string
    onClose: () => void
}

const EXPIRY_OPTIONS = [
    { label: '1小时', hours: 1 },
    { label: '24小时', hours: 24 },
    { label: '7天', hours: 168 },
    { label: '永不过期', hours: null },
]

export function ShareDialog({
    visible,
    filePath,
    appType,
    fileName,
    onClose,
}: ShareDialogProps) {
    const [expiryIndex, setExpiryIndex] = useState(1) // Default 24h
    const [burnAfterRead, setBurnAfterRead] = useState(false)
    const [shareUrl, setShareUrl] = useState<string | null>(null)

    const createShare = useCreateShare()

    const handleCreateShare = async () => {
        try {
            const result = await createShare.mutateAsync({
                file_path: filePath,
                app_type: appType,
                expires_in_hours: EXPIRY_OPTIONS[expiryIndex].hours || undefined,
                burn_after_read: burnAfterRead,
            })
            setShareUrl(result.url)
        } catch (error: any) {
            Alert.alert('创建分享失败', error.message)
        }
    }

    const handleCopyLink = async () => {
        if (shareUrl) {
            await Clipboard.setStringAsync(shareUrl)
            Alert.alert('已复制', '链接已复制到剪贴板')
        }
    }

    const handleShareFile = async () => {
        if (Platform.OS === 'web') {
            Alert.alert('不支持', 'Web端不支持直接分享文件')
            return
        }

        if (await Sharing.isAvailableAsync()) {
            // For native apps, we would download then share
            // This is a placeholder - actual file sharing would require downloading first
            Alert.alert('提示', '请先下载文件后分享')
        }
    }

    const handleClose = () => {
        setShareUrl(null)
        onClose()
    }

    return (
        <Sheet
            modal
            open={visible}
            onOpenChange={(open: boolean) => !open && handleClose()}
            snapPoints={[50]}
            dismissOnSnapToBottom
        >
            <Sheet.Overlay />
            <Sheet.Frame padding="$4" backgroundColor="$background">
                <Sheet.Handle />

                <YStack gap="$4" marginTop="$4">
                    <Text fontSize="$6" fontWeight="bold" color="$text">
                        分享文件
                    </Text>

                    <Text color="$textMuted" numberOfLines={1}>
                        {fileName}
                    </Text>

                    {shareUrl ? (
                        // Show share link
                        <YStack gap="$3">
                            <YStack
                                backgroundColor="$surface"
                                padding="$3"
                                borderRadius="$2"
                            >
                                <Text fontSize="$3" color="$text" selectable>
                                    {shareUrl}
                                </Text>
                            </YStack>

                            <XStack gap="$3">
                                <Button
                                    flex={1}
                                    backgroundColor="$primary"
                                    onPress={handleCopyLink}
                                    icon={<Ionicons name="copy" size={18} color="#fff" />}
                                >
                                    复制链接
                                </Button>

                                {Platform.OS !== 'web' && (
                                    <Button
                                        flex={1}
                                        backgroundColor="$secondary"
                                        onPress={handleShareFile}
                                        icon={<Ionicons name="share" size={18} color="#fff" />}
                                    >
                                        分享文件
                                    </Button>
                                )}
                            </XStack>
                        </YStack>
                    ) : (
                        // Share options
                        <YStack gap="$4">
                            {/* Expiry options */}
                            <YStack gap="$2">
                                <Text color="$textMuted">链接有效期</Text>
                                <XStack gap="$2" flexWrap="wrap">
                                    {EXPIRY_OPTIONS.map((option, index) => (
                                        <Button
                                            key={option.label}
                                            size="$3"
                                            backgroundColor={expiryIndex === index ? '$primary' : '$surface'}
                                            onPress={() => setExpiryIndex(index)}
                                        >
                                            <Text color={expiryIndex === index ? '#fff' : '$text'}>
                                                {option.label}
                                            </Text>
                                        </Button>
                                    ))}
                                </XStack>
                            </YStack>

                            {/* Burn after read */}
                            <XStack alignItems="center" justifyContent="space-between">
                                <YStack>
                                    <Text color="$text">阅后即焚</Text>
                                    <Text fontSize="$2" color="$textMuted">
                                        查看一次后链接失效
                                    </Text>
                                </YStack>
                                <Switch
                                    checked={burnAfterRead}
                                    onCheckedChange={setBurnAfterRead}
                                    backgroundColor={burnAfterRead ? '$primary' : '$surface'}
                                >
                                    <Switch.Thumb animation="quick" />
                                </Switch>
                            </XStack>

                            {/* Create button */}
                            <Button
                                backgroundColor="$primary"
                                onPress={handleCreateShare}
                                disabled={createShare.isPending}
                            >
                                {createShare.isPending ? '创建中...' : '创建分享链接'}
                            </Button>
                        </YStack>
                    )}

                    <Button
                        variant="outlined"
                        borderColor="$border"
                        onPress={handleClose}
                    >
                        关闭
                    </Button>
                </YStack>
            </Sheet.Frame>
        </Sheet>
    )
}
