import { create } from 'zustand'

interface LyricsLine {
    timeMs: number
    text: string
}

interface PlayerState {
    // Current playback
    currentTrack: string | null
    trackTitle: string
    trackArtist: string
    trackAlbum: string
    coverArt: string | null

    // Playback state
    isPlaying: boolean
    positionMs: number
    durationMs: number

    // Lyrics
    lyrics: LyricsLine[]
    currentLyricIndex: number

    // Video
    isVideo: boolean
    isFullscreen: boolean

    // Actions
    setTrack: (path: string, info: Partial<PlayerState>) => void
    setPlaying: (playing: boolean) => void
    setPosition: (ms: number) => void
    setDuration: (ms: number) => void
    setLyrics: (lyrics: LyricsLine[]) => void
    updateLyricIndex: (positionMs: number) => void
    setFullscreen: (fullscreen: boolean) => void
    clear: () => void
}

export const usePlayerStore = create<PlayerState>((set, get) => ({
    currentTrack: null,
    trackTitle: '',
    trackArtist: '',
    trackAlbum: '',
    coverArt: null,
    isPlaying: false,
    positionMs: 0,
    durationMs: 0,
    lyrics: [],
    currentLyricIndex: -1,
    isVideo: false,
    isFullscreen: false,

    setTrack: (path, info) => set({
        currentTrack: path,
        trackTitle: info.trackTitle || '',
        trackArtist: info.trackArtist || '',
        trackAlbum: info.trackAlbum || '',
        coverArt: info.coverArt || null,
        isVideo: info.isVideo || false,
        isPlaying: false,
        positionMs: 0,
        lyrics: [],
        currentLyricIndex: -1,
    }),

    setPlaying: (playing) => set({ isPlaying: playing }),

    setPosition: (ms) => set({ positionMs: ms }),

    setDuration: (ms) => set({ durationMs: ms }),

    setLyrics: (lyrics) => set({ lyrics }),

    updateLyricIndex: (positionMs) => {
        const { lyrics } = get()
        if (lyrics.length === 0) return

        let index = -1
        for (let i = 0; i < lyrics.length; i++) {
            if (lyrics[i].timeMs <= positionMs) {
                index = i
            } else {
                break
            }
        }

        set({ currentLyricIndex: index })
    },

    setFullscreen: (fullscreen) => set({ isFullscreen: fullscreen }),

    clear: () => set({
        currentTrack: null,
        trackTitle: '',
        trackArtist: '',
        trackAlbum: '',
        coverArt: null,
        isPlaying: false,
        positionMs: 0,
        durationMs: 0,
        lyrics: [],
        currentLyricIndex: -1,
        isVideo: false,
        isFullscreen: false,
    }),
}))
