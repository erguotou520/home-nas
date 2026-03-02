import React, { useEffect, useState } from 'react'
import { StyleSheet, Image } from 'react-native'
import { YStack, XStack, Text, Button, Slider } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { Audio } from 'expo-av'
import { usePlayerStore } from '../stores'
import { useLyrics, getStreamUrl, getThumbnailUrl } from '../api'
import { LyricsDisplay } from '../components'

interface MusicPlayerScreenProps {
    route: any
    navigation: any
}

export function MusicPlayerScreen({ route, navigation }: MusicPlayerScreenProps) {
    const { path } = route.params
    const [sound, setSound] = useState<Audio.Sound | null>(null)

    const {
        isPlaying,
        positionMs,
        durationMs,
        setPlaying,
        setPosition,
        setDuration,
        setLyrics,
        updateLyricIndex,
        currentLyricIndex,
    } = usePlayerStore()

    const { data: lyricsData } = useLyrics(path)

    useEffect(() => {
        if (lyricsData) {
            setLyrics(lyricsData)
        }
    }, [lyricsData])

    async function playSound() {
        try {
            if (sound) {
                await sound.playAsync()
                setPlaying(true)
                return
            }

            const streamUrl = getStreamUrl(path)
            const { sound: newSound } = await Audio.Sound.createAsync(
                { uri: streamUrl },
                { shouldPlay: true },
                onPlaybackStatusUpdate
            )
            setSound(newSound)
            setPlaying(true)
        } catch (error) {
            console.error('Error playing sound', error)
        }
    }

    async function pauseSound() {
        if (sound) {
            await sound.pauseAsync()
            setPlaying(false)
        }
    }

    const onPlaybackStatusUpdate = (status: any) => {
        if (status.isLoaded) {
            setPosition(status.positionMillis)
            setDuration(status.durationMillis || 0)
            updateLyricIndex(status.positionMillis)
            if (status.didJustFinish) {
                setPlaying(false)
            }
        }
    }

    useEffect(() => {
        playSound()
        return () => {
            if (sound) {
                sound.unloadAsync()
            }
        }
    }, [])

    const formatTime = (ms: number) => {
        const totalSeconds = Math.floor(ms / 1000)
        const minutes = Math.floor(totalSeconds / 60)
        const seconds = totalSeconds % 60
        return `${minutes}:${seconds.toString().padStart(2, '0')}`
    }

    return (
        <YStack flex={1} backgroundColor="$background">
            {/* Header */}
            <XStack padding="$4" alignItems="center" justifyContent="space-between">
                <Button size="$3" chromeless onPress={() => navigation.goBack()}>
                    <Ionicons name="chevron-down" size={28} color="$text" />
                </Button>
                <YStack alignItems="center">
                    <Text fontSize="$2" color="$textMuted">正在播放</Text>
                    <Text fontSize="$4" fontWeight="bold" color="$text" numberOfLines={1}>
                        {path.split('/').pop()}
                    </Text>
                </YStack>
                <Button size="$3" chromeless>
                    <Ionicons name="ellipsis-horizontal" size={24} color="$text" />
                </Button>
            </XStack>

            {/* Album Art / Lyrics Toggle Container */}
            <YStack flex={1} padding="$4" justifyContent="center">
                {/* We can toggle between image and lyrics. For now, show lyrics if available, or image */}
                {lyricsData && lyricsData.length > 0 ? (
                    <LyricsDisplay lyrics={lyricsData} currentIndex={currentLyricIndex} />
                ) : (
                    <YStack alignItems="center" justifyContent="center">
                        <Image
                            source={{ uri: getThumbnailUrl(path) }}
                            style={{ width: 300, height: 300, borderRadius: 20 }}
                        />
                    </YStack>
                )}
            </YStack>

            {/* Controls */}
            <YStack padding="$6" gap="$4" backgroundColor="$surface">
                {/* Progress Bar */}
                <YStack gap="$2">
                    <Slider
                        size="$2"
                        width="100%"
                        value={[positionMs]}
                        max={durationMs || 100}
                        step={100}
                        onValueChange={(val) => {
                            if (sound) sound.setPositionAsync(val[0])
                        }}
                    >
                        <Slider.Track backgroundColor="$border">
                            <Slider.TrackActive backgroundColor="$primary" />
                        </Slider.Track>
                        <Slider.Thumb index={0} circular elevation="$2" />
                    </Slider>
                    <XStack justifyContent="space-between">
                        <Text color="$textMuted" fontSize="$2">{formatTime(positionMs)}</Text>
                        <Text color="$textMuted" fontSize="$2">{formatTime(durationMs)}</Text>
                    </XStack>
                </YStack>

                {/* Playback Controls */}
                <XStack justifyContent="center" alignItems="center" gap="$8">
                    <Button chromeless>
                        <Ionicons name="play-skip-back" size={32} color="$text" />
                    </Button>
                    <Button
                        size="$6"
                        circular
                        backgroundColor="$primary"
                        onPress={isPlaying ? pauseSound : playSound}
                    >
                        <Ionicons name={isPlaying ? 'pause' : 'play'} size={32} color="white" />
                    </Button>
                    <Button chromeless>
                        <Ionicons name="play-skip-forward" size={32} color="$text" />
                    </Button>
                </XStack>
            </YStack>
        </YStack>
    )
}
