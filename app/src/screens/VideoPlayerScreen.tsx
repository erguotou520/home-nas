import React, { useState, useRef } from 'react'
import { StyleSheet, useWindowDimensions, Platform } from 'react-native'
import { YStack, XStack, Text, Button, Spinner } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { Video, ResizeMode, AVPlaybackStatus } from 'expo-av'
import { getStreamUrl } from '../api'

interface VideoPlayerScreenProps {
    route: any
    navigation: any
}

export function VideoPlayerScreen({ route, navigation }: VideoPlayerScreenProps) {
    const { path } = route.params
    const videoRef = useRef<Video>(null)
    const [status, setStatus] = useState<AVPlaybackStatus>({} as AVPlaybackStatus)
    const { width, height } = useWindowDimensions()

    const handleFullscreen = async () => {
        if (videoRef.current) {
            await videoRef.current.presentFullscreenPlayer()
        }
    }

    const streamUrl = getStreamUrl(path)

    return (
        <YStack flex={1} backgroundColor="black">
            {/* Header */}
            <XStack
                paddingHorizontal="$4"
                paddingVertical="$3"
                alignItems="center"
                justifyContent="space-between"
                position="absolute"
                top={0}
                left={0}
                right={0}
                zIndex={10}
                backgroundColor="rgba(0,0,0,0.5)"
            >
                <Button size="$3" chromeless onPress={() => navigation.goBack()}>
                    <Ionicons name="close" size={28} color="white" />
                </Button>
                <Text color="white" fontSize="$4" numberOfLines={1} flex={1} textAlign="center" marginHorizontal="$4">
                    {path.split('/').pop()}
                </Text>
                <YStack width={40} />
            </XStack>

            {/* Video Player */}
            <YStack flex={1} justifyContent="center" alignItems="center">
                <Video
                    ref={videoRef}
                    style={{ width: width, height: height * 0.7 }}
                    source={{ uri: streamUrl }}
                    useNativeControls
                    resizeMode={ResizeMode.CONTAIN}
                    isLooping={false}
                    onPlaybackStatusUpdate={status => setStatus(() => status)}
                    onError={(error) => console.error('Video Error:', error)}
                />
            </YStack>

            {/* Footer / Info */}
            <YStack padding="$4" backgroundColor="rgba(0,0,0,0.5)">
                <Text color="white" fontSize="$3">
                    视频地址: {streamUrl}
                </Text>
            </YStack>
        </YStack>
    )
}
