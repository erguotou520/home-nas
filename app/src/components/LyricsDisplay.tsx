import React, { useRef, useEffect } from 'react'
import { ScrollView } from 'react-native'
import { YStack, Text } from 'tamagui'
import Animated, {
    useAnimatedStyle,
    withTiming,
    useSharedValue,
} from 'react-native-reanimated'

interface LyricsLine {
    timeMs: number
    text: string
}

interface LyricsDisplayProps {
    lyrics: LyricsLine[]
    currentIndex: number
    onLinePress?: (index: number) => void
}

const LINE_HEIGHT = 36

export function LyricsDisplay({ lyrics, currentIndex, onLinePress }: LyricsDisplayProps) {
    const scrollRef = useRef<ScrollView>(null)

    // Auto-scroll to current lyric
    useEffect(() => {
        if (currentIndex >= 0 && scrollRef.current) {
            scrollRef.current.scrollTo({
                y: Math.max(0, currentIndex * LINE_HEIGHT - 100),
                animated: true,
            })
        }
    }, [currentIndex])

    if (lyrics.length === 0) {
        return (
            <YStack flex={1} alignItems="center" justifyContent="center" padding="$4">
                <Text color="$textMuted" fontSize="$4">
                    暂无歌词
                </Text>
            </YStack>
        )
    }

    return (
        <ScrollView
            ref={scrollRef}
            showsVerticalScrollIndicator={false}
            contentContainerStyle={{ paddingVertical: 100 }}
        >
            <YStack alignItems="center" gap="$2">
                {lyrics.map((line, index) => (
                    <LyricLine
                        key={`${index}-${line.timeMs}`}
                        text={line.text}
                        isActive={index === currentIndex}
                        isPassed={index < currentIndex}
                        onPress={() => onLinePress?.(index)}
                    />
                ))}
            </YStack>
        </ScrollView>
    )
}

interface LyricLineProps {
    text: string
    isActive: boolean
    isPassed: boolean
    onPress?: () => void
}

function LyricLine({ text, isActive, isPassed, onPress }: LyricLineProps) {
    const scale = useSharedValue(1)
    const opacity = useSharedValue(0.5)

    useEffect(() => {
        scale.value = withTiming(isActive ? 1.1 : 1, { duration: 200 })
        opacity.value = withTiming(isActive ? 1 : isPassed ? 0.4 : 0.6, { duration: 200 })
    }, [isActive, isPassed])

    const animatedStyle = useAnimatedStyle(() => ({
        transform: [{ scale: scale.value }],
        opacity: opacity.value,
    }))

    return (
        <Animated.View style={animatedStyle}>
            <Text
                fontSize={isActive ? '$5' : '$4'}
                fontWeight={isActive ? 'bold' : 'normal'}
                color={isActive ? '$primary' : '$text'}
                textAlign="center"
                paddingVertical="$2"
                paddingHorizontal="$4"
                onPress={onPress}
                height={LINE_HEIGHT}
            >
                {text || '♪'}
            </Text>
        </Animated.View>
    )
}
