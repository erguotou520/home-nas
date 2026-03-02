import { useQuery } from '@tanstack/react-query'
import { apiClient } from './client'

interface LyricsLine {
    time_ms: number
    text: string
}

interface MediaInfo {
    title?: string
    artist?: string
    album?: string
    year?: number
    duration_secs?: number
    cover_art?: string
}

export function useLyrics(path: string) {
    return useQuery({
        queryKey: ['lyrics', path],
        queryFn: async () => {
            const response = await apiClient.get<{ lyrics: LyricsLine[] }>(
                `/api/media/lyrics/${encodeURIComponent(path)}`
            )
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data!.lyrics.map(l => ({
                timeMs: l.time_ms,
                text: l.text,
            }))
        },
        enabled: !!path,
    })
}

export function useMediaInfo(path: string) {
    return useQuery({
        queryKey: ['mediaInfo', path],
        queryFn: async () => {
            const response = await apiClient.get<MediaInfo>(
                `/api/media/info/${encodeURIComponent(path)}`
            )
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data!
        },
        enabled: !!path,
    })
}

export function getStreamUrl(path: string): string {
    return apiClient.getMediaUrl(path)
}

export function getThumbnailUrl(path: string): string {
    return apiClient.getThumbnailUrl(path)
}
