import React, { useEffect, useRef } from 'react'
import { NavigationContainer } from '@react-navigation/native'
import { createNativeStackNavigator } from '@react-navigation/native-stack'
import { CommonActions } from '@react-navigation/native'

import { useAuthStore } from '../stores'
import {
    LoginScreen,
    HomeScreen,
    BrowserScreen,
    VideoPlayerScreen,
    MusicPlayerScreen,
    ImageViewerScreen
} from '../screens'

const Stack = createNativeStackNavigator()

export function AppNavigator() {
    const { token, isLoading } = useAuthStore()
    const navigationRef = useRef<any>(null)
    const routeNameRef = useRef<string | null>(null)

    // Redirect to /login when not authenticated
    useEffect(() => {
        if (!isLoading && !token && navigationRef.current) {
            // Small delay to ensure navigation is ready
            const timer = setTimeout(() => {
                const currentRoute = navigationRef.current?.getCurrentRoute()
                if (currentRoute?.name !== 'Login') {
                    navigationRef.current?.dispatch(
                        CommonActions.reset({
                            index: 0,
                            routes: [{ name: 'Login' }],
                        })
                    )
                }
            }, 100)
            return () => clearTimeout(timer)
        }
    }, [token, isLoading])

    if (isLoading) {
        return null // Or a splash screen
    }

    const linking = {
        prefixes: ['home-nas://', 'http://localhost:8081'],
        config: {
            screens: {
                Login: 'login',
                Home: '',
                Browser: 'files/:app',
                VideoPlayer: 'video',
                MusicPlayer: 'music',
                ImageViewer: 'image',
            },
        },
    }

    return (
        <NavigationContainer
            ref={navigationRef}
            linking={linking}
            onReady={() => {
                routeNameRef.current = navigationRef.current?.getCurrentRoute()?.name || null
            }}
            onStateChange={async () => {
                const previousRouteName = routeNameRef.current
                const currentRouteName = navigationRef.current?.getCurrentRoute()?.name

                if (previousRouteName !== currentRouteName) {
                    routeNameRef.current = currentRouteName
                }
            }}
        >
            <Stack.Navigator
                screenOptions={{
                    headerShown: false,
                    animation: 'slide_from_right',
                }}
            >
                {token ? (
                    // Authenticated screens
                    <>
                        <Stack.Screen name="Home" component={HomeScreen} />
                        <Stack.Screen name="Browser" component={BrowserScreen} />
                        <Stack.Screen name="VideoPlayer" component={VideoPlayerScreen} />
                        <Stack.Screen name="MusicPlayer" component={MusicPlayerScreen} />
                        <Stack.Screen name="ImageViewer" component={ImageViewerScreen} />
                    </>
                ) : (
                    // Unauthenticated screens
                    <Stack.Screen name="Login" component={LoginScreen} />
                )}
            </Stack.Navigator>
        </NavigationContainer>
    )
}
