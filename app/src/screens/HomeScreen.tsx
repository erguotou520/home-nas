import React from 'react'
import { ScrollView, Pressable, useWindowDimensions } from 'react-native'
import { YStack, XStack, Text, Card } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { useAuthStore, useFileStore } from '../stores'

interface HomeScreenProps {
    navigation: any
}

const APPS = [
    {
        id: 'medias',
        name: '媒体库',
        icon: 'images' as const,
        color: '#ec4899',
        description: '照片和图片',
    },
    {
        id: 'videos',
        name: '视频库',
        icon: 'videocam' as const,
        color: '#f59e0b',
        description: '电影和剧集',
    },
    {
        id: 'music',
        name: '音乐库',
        icon: 'musical-notes' as const,
        color: '#8b5cf6',
        description: '音乐和播客',
    },
    {
        id: 'documents',
        name: '文档库',
        icon: 'document-text' as const,
        color: '#3b82f6',
        description: '文档和文件',
    },
]

export function HomeScreen({ navigation }: HomeScreenProps) {
    const { width } = useWindowDimensions()
    const { user, logout } = useAuthStore()
    const { setCurrentApp } = useFileStore()

    const isLargeScreen = width > 600
    const numColumns = isLargeScreen ? 4 : 2

    const handleAppPress = (appId: string) => {
        setCurrentApp(appId)
        navigation.navigate('Browser', { app: appId })
    }

    return (
        <YStack flex={1} backgroundColor="$background">
            {/* Header */}
            <XStack
                paddingHorizontal="$4"
                paddingVertical="$4"
                alignItems="center"
                justifyContent="space-between"
                borderBottomWidth={1}
                borderBottomColor="$borderColor"
            >
                <YStack>
                    <Text fontSize="$6" fontWeight="bold" color="$text">
                        欢迎回来
                    </Text>
                    <Text fontSize="$3" color="$textMuted">
                        {user?.username} · {user?.role === 'admin' ? '管理员' : '用户'}
                    </Text>
                </YStack>

                <XStack gap="$2">
                    {user?.role === 'admin' && (
                        <Pressable onPress={() => navigation.navigate('Admin')}>
                            <YStack
                                padding="$2"
                                backgroundColor="$surface"
                                borderRadius="$3"
                            >
                                <Ionicons name="settings" size={24} color="#6366f1" />
                            </YStack>
                        </Pressable>
                    )}

                    <Pressable onPress={logout}>
                        <YStack
                            padding="$2"
                            backgroundColor="$surface"
                            borderRadius="$3"
                        >
                            <Ionicons name="log-out" size={24} color="#94a3b8" />
                        </YStack>
                    </Pressable>
                </XStack>
            </XStack>

            {/* Apps Grid */}
            <ScrollView contentContainerStyle={{ padding: 16 }}>
                <YStack gap="$4">
                    <XStack flexWrap="wrap" marginHorizontal={-8}>
                        {APPS.map((app) => (
                            <Pressable
                                key={app.id}
                                onPress={() => handleAppPress(app.id)}
                                style={{
                                    width: `${100 / numColumns}%`,
                                    padding: 8,
                                }}
                            >
                                <Card
                                    backgroundColor="$surface"
                                    borderRadius="$4"
                                    padding="$5"
                                    hoverStyle={{ backgroundColor: '$surfaceHover' }}
                                    pressStyle={{ scale: 0.98 }}
                                    animation="quick"
                                >
                                    <YStack alignItems="center" gap="$3">
                                        <YStack
                                            width={60}
                                            height={60}
                                            backgroundColor={app.color}
                                            borderRadius="$4"
                                            alignItems="center"
                                            justifyContent="center"
                                        >
                                            <Ionicons name={app.icon} size={30} color="#fff" />
                                        </YStack>

                                        <YStack alignItems="center">
                                            <Text fontSize="$5" fontWeight="bold" color="$text">
                                                {app.name}
                                            </Text>
                                            <Text fontSize="$2" color="$textMuted">
                                                {app.description}
                                            </Text>
                                        </YStack>
                                    </YStack>
                                </Card>
                            </Pressable>
                        ))}
                    </XStack>

                    {/* Quick Links */}
                    <YStack marginTop="$4">
                        <Text fontSize="$4" fontWeight="bold" color="$text" marginBottom="$3">
                            快捷操作
                        </Text>

                        <XStack gap="$3">
                            <Pressable
                                style={{ flex: 1 }}
                                onPress={() => navigation.navigate('Shares')}
                            >
                                <Card
                                    backgroundColor="$surface"
                                    borderRadius="$3"
                                    padding="$4"
                                >
                                    <XStack alignItems="center" gap="$3">
                                        <Ionicons name="link" size={24} color="#22c55e" />
                                        <Text color="$text">我的分享</Text>
                                    </XStack>
                                </Card>
                            </Pressable>

                            <Pressable
                                style={{ flex: 1 }}
                                onPress={() => navigation.navigate('Upload')}
                            >
                                <Card
                                    backgroundColor="$surface"
                                    borderRadius="$3"
                                    padding="$4"
                                >
                                    <XStack alignItems="center" gap="$3">
                                        <Ionicons name="cloud-upload" size={24} color="#6366f1" />
                                        <Text color="$text">上传文件</Text>
                                    </XStack>
                                </Card>
                            </Pressable>
                        </XStack>
                    </YStack>
                </YStack>
            </ScrollView>
        </YStack>
    )
}
