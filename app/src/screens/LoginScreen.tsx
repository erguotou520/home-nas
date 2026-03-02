import React, { useState } from 'react'
import { KeyboardAvoidingView, Platform, Alert } from 'react-native'
import { YStack, XStack, Text, Input, Button, Spinner } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import { useLogin } from '../api'
import { useAuthStore } from '../stores'

interface LoginScreenProps {
    navigation: any
}

interface FormErrors {
    serverUrl?: string
    username?: string
    password?: string
}

export function LoginScreen({ navigation }: LoginScreenProps) {
    const { serverUrl: savedServerUrl, setServerUrl: saveServerUrl } = useAuthStore()
    const [serverUrl, setServerUrl] = useState(savedServerUrl || '')
    const [username, setUsername] = useState('')
    const [password, setPassword] = useState('')
    const [showPassword, setShowPassword] = useState(false)
    const [errors, setErrors] = useState<FormErrors>({})
    const [touched, setTouched] = useState<{ serverUrl: boolean; username: boolean; password: boolean }>({
        serverUrl: false,
        username: false,
        password: false,
    })
    const login = useLogin()

    const validateField = (field: keyof FormErrors, value: string) => {
        if (!value.trim()) {
            return `${getFieldLabel(field)}不能为空`
        }
        return ''
    }

    const getFieldLabel = (field: keyof FormErrors): string => {
        switch (field) {
            case 'serverUrl':
                return '服务器地址'
            case 'username':
                return '用户名'
            case 'password':
                return '密码'
            default:
                return ''
        }
    }

    const handleFieldChange = (field: 'serverUrl' | 'username' | 'password', value: string) => {
        // Clear error when user starts typing
        if (errors[field]) {
            setErrors((prev) => ({ ...prev, [field]: undefined }))
        }

        // Update field value
        switch (field) {
            case 'serverUrl':
                setServerUrl(value)
                break
            case 'username':
                setUsername(value)
                break
            case 'password':
                setPassword(value)
                break
        }
    }

    const handleFieldBlur = (field: 'serverUrl' | 'username' | 'password', value: string) => {
        setTouched((prev) => ({ ...prev, [field]: true }))
        const error = validateField(field, value)
        if (error) {
            setErrors((prev) => ({ ...prev, [field]: error }))
        }
    }

    const handleLogin = async () => {
        // Mark all fields as touched
        setTouched({ serverUrl: true, username: true, password: true })

        // Validate all fields
        const newErrors: FormErrors = {}
        let hasError = false

        if (!serverUrl.trim()) {
            newErrors.serverUrl = '服务器地址不能为空'
            hasError = true
        }
        if (!username.trim()) {
            newErrors.username = '用户名不能为空'
            hasError = true
        }
        if (!password.trim()) {
            newErrors.password = '密码不能为空'
            hasError = true
        }

        if (hasError) {
            setErrors(newErrors)
            return
        }

        // Save server URL first
        saveServerUrl(serverUrl.trim())

        try {
            await login.mutateAsync({ username, password })
            // Navigation will be handled by auth state change
        } catch (error: any) {
            Alert.alert('登录失败', error.message || '请检查用户名和密码')
        }
    }

    return (
        <KeyboardAvoidingView
            behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
            style={{ flex: 1 }}
        >
            <YStack
                flex={1}
                backgroundColor="$background"
                padding="$6"
                justifyContent="center"
            >
                {/* Logo */}
                <YStack alignItems="center" marginBottom="$8">
                    <YStack
                        width={80}
                        height={80}
                        backgroundColor="$primary"
                        borderRadius="$6"
                        alignItems="center"
                        justifyContent="center"
                        marginBottom="$4"
                    >
                        <Ionicons name="server" size={40} color="#fff" />
                    </YStack>
                    <Text fontSize="$8" fontWeight="bold" color="$text">
                        Home NAS
                    </Text>
                    <Text fontSize="$3" color="$textMuted" marginTop="$2">
                        私有云存储管理
                    </Text>
                </YStack>

                {/* Form */}
                <YStack gap="$4">
                    {/* Server URL */}
                    <YStack>
                        <Text marginBottom="$2" color="$textMuted" fontSize="$3">
                            服务器地址
                        </Text>
                        <Input
                            placeholder="https://your-server.com"
                            value={serverUrl}
                            onChangeText={(value) => handleFieldChange('serverUrl', value)}
                            onBlur={() => handleFieldBlur('serverUrl', serverUrl)}
                            autoCapitalize="none"
                            autoCorrect={false}
                            keyboardType="url"
                            backgroundColor="$surface"
                            borderColor={touched.serverUrl && errors.serverUrl ? '$red10' : '$border'}
                            color="$text"
                            placeholderTextColor="$textMuted"
                        />
                        {touched.serverUrl && errors.serverUrl && (
                            <Text marginTop="$1" color="$red10" fontSize="$2">
                                {errors.serverUrl}
                            </Text>
                        )}
                    </YStack>

                    {/* Username */}
                    <YStack>
                        <Text marginBottom="$2" color="$textMuted" fontSize="$3">
                            用户名
                        </Text>
                        <Input
                            placeholder="请输入用户名"
                            value={username}
                            onChangeText={(value) => handleFieldChange('username', value)}
                            onBlur={() => handleFieldBlur('username', username)}
                            autoCapitalize="none"
                            autoCorrect={false}
                            backgroundColor="$surface"
                            borderColor={touched.username && errors.username ? '$red10' : '$border'}
                            color="$text"
                            placeholderTextColor="$textMuted"
                        />
                        {touched.username && errors.username && (
                            <Text marginTop="$1" color="$red10" fontSize="$2">
                                {errors.username}
                            </Text>
                        )}
                    </YStack>

                    {/* Password */}
                    <YStack>
                        <Text marginBottom="$2" color="$textMuted" fontSize="$3">
                            密码
                        </Text>
                        <XStack alignItems="center">
                            <Input
                                flex={1}
                                placeholder="请输入密码"
                                value={password}
                                onChangeText={(value) => handleFieldChange('password', value)}
                                onBlur={() => handleFieldBlur('password', password)}
                                secureTextEntry={!showPassword}
                                autoCapitalize="none"
                                autoCorrect={false}
                                backgroundColor="$surface"
                                borderColor={touched.password && errors.password ? '$red10' : '$border'}
                                color="$text"
                                placeholderTextColor="$textMuted"
                            />
                            <Button
                                position="absolute"
                                right="$2"
                                size="$2"
                                chromeless
                                onPress={() => setShowPassword(!showPassword)}
                            >
                                <Ionicons
                                    name={showPassword ? 'eye-off' : 'eye'}
                                    size={20}
                                    color="#94a3b8"
                                />
                            </Button>
                        </XStack>
                        {touched.password && errors.password && (
                            <Text marginTop="$1" color="$red10" fontSize="$2">
                                {errors.password}
                            </Text>
                        )}
                    </YStack>

                    {/* Login Button */}
                    <Button
                        marginTop="$4"
                        backgroundColor="$primary"
                        height={50}
                        onPress={handleLogin}
                        disabled={login.isPending}
                    >
                        {login.isPending ? (
                            <Spinner color="white" />
                        ) : (
                            <Text color="white" fontWeight="bold" fontSize="$4">
                                登录
                            </Text>
                        )}
                    </Button>
                </YStack>
            </YStack>
        </KeyboardAvoidingView>
    )
}
