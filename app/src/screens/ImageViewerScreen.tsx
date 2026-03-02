import React from 'react'
import { Image, useWindowDimensions } from 'react-native'
import { YStack, XStack, Button } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { getStreamUrl } from '../api'

interface ImageViewerScreenProps {
    route: any
    navigation: any
}

export function ImageViewerScreen({ route, navigation }: ImageViewerScreenProps) {
    const { path } = route.params
    const { width, height } = useWindowDimensions()
    const imageUrl = getStreamUrl(path)

    return (
        <YStack flex={1} backgroundColor="black" justifyContent="center" alignItems="center">
            {/* Header */}
            <XStack
                position="absolute"
                top={0}
                left={0}
                right={0}
                padding="$4"
                zIndex={10}
                backgroundColor="rgba(0,0,0,0.5)"
            >
                <Button size="$3" chromeless onPress={() => navigation.goBack()}>
                    <Ionicons name="close" size={28} color="white" />
                </Button>
            </XStack>

            <Image
                source={{ uri: imageUrl }}
                style={{ width: width, height: height }}
                resizeMode="contain"
            />
        </YStack>
    )
}
