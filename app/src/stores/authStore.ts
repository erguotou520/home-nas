import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'
import * as SecureStore from 'expo-secure-store'

interface User {
    id: number
    username: string
    role: string
}

interface AuthState {
    token: string | null
    user: User | null
    isLoading: boolean
    serverUrl: string

    setToken: (token: string | null) => void
    setUser: (user: User | null) => void
    setServerUrl: (url: string) => void
    logout: () => void
    isAuthenticated: () => boolean
    isAdmin: () => boolean
}

// Custom storage adapter for Expo SecureStore
const secureStorage = {
    getItem: async (name: string): Promise<string | null> => {
        try {
            return await SecureStore.getItemAsync(name)
        } catch {
            return null
        }
    },
    setItem: async (name: string, value: string): Promise<void> => {
        try {
            await SecureStore.setItemAsync(name, value)
        } catch {
            // Ignore storage errors
        }
    },
    removeItem: async (name: string): Promise<void> => {
        try {
            await SecureStore.deleteItemAsync(name)
        } catch {
            // Ignore storage errors
        }
    },
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            token: null,
            user: null,
            isLoading: true,
            serverUrl: '',

            setToken: (token) => set({ token }),

            setUser: (user) => set({ user }),

            setServerUrl: (url) => set({ serverUrl: url.replace(/\/$/, '') }),

            logout: () => set({ token: null, user: null }),

            isAuthenticated: () => !!get().token,

            isAdmin: () => get().user?.role === 'admin',
        }),
        {
            name: 'auth-storage',
            storage: createJSONStorage(() => secureStorage),
            onRehydrateStorage: () => (state) => {
                if (state) {
                    state.isLoading = false
                }
            },
        }
    )
)
