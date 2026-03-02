import React, { useMemo, useState } from 'react'
import { Alert } from 'react-native'
import { Button, Card, Input, Spinner, Text, XStack, YStack } from 'tamagui'
import { Ionicons } from '@expo/vector-icons'
import * as DocumentPicker from 'expo-document-picker'
import { useUploadFile } from '../api'

const APPS = [
  { id: 'medias', name: '媒体库' },
  { id: 'videos', name: '视频库' },
  { id: 'music', name: '音乐库' },
  { id: 'documents', name: '文档库' },
]

interface UploadScreenProps {
  navigation: any
}

export function UploadScreen({ navigation }: UploadScreenProps) {
  const [app, setApp] = useState('documents')
  const [targetPath, setTargetPath] = useState('')
  const [selectedFile, setSelectedFile] = useState<{ uri: string; name: string } | null>(null)
  const uploadFile = useUploadFile()

  const appLabel = useMemo(() => APPS.find((a) => a.id === app)?.name ?? app, [app])

  const handleSelectFile = async () => {
    const result = await DocumentPicker.getDocumentAsync({ multiple: false, copyToCacheDirectory: true })
    if (result.canceled) return
    const file = result.assets[0]
    setSelectedFile({ uri: file.uri, name: file.name })
  }

  const handleUpload = async () => {
    if (!selectedFile) {
      Alert.alert('提示', '请先选择文件')
      return
    }

    try {
      const response = await fetch(selectedFile.uri)
      const buffer = await response.arrayBuffer()
      const binary = new Uint8Array(buffer)

      await uploadFile.mutateAsync({
        app,
        path: targetPath,
        filename: selectedFile.name,
        bytes: binary,
        overwrite: false,
      })

      Alert.alert('上传成功', `已上传到 ${appLabel}/${targetPath || '.'}`)
      setSelectedFile(null)
    } catch (error: any) {
      Alert.alert('上传失败', error.message || '未知错误')
    }
  }

  return (
    <YStack flex={1} backgroundColor="$background" padding="$4" gap="$4">
      <XStack alignItems="center" gap="$3">
        <Button size="$3" chromeless onPress={() => navigation.goBack()}>
          <Ionicons name="arrow-back" size={22} color="#6366f1" />
        </Button>
        <Text fontSize="$6" fontWeight="bold" color="$text">上传文件</Text>
      </XStack>

      <Card backgroundColor="$surface" padding="$4" borderRadius="$4">
        <YStack gap="$3">
          <Text color="$textMuted">选择目标库</Text>
          <XStack gap="$2" flexWrap="wrap">
            {APPS.map((item) => (
              <Button
                key={item.id}
                size="$3"
                backgroundColor={app === item.id ? '$primary' : '$surfaceHover'}
                onPress={() => setApp(item.id)}
              >
                <Text color={app === item.id ? '#fff' : '$text'}>{item.name}</Text>
              </Button>
            ))}
          </XStack>

          <Text color="$textMuted">目标目录（可留空表示根目录）</Text>
          <Input
            value={targetPath}
            onChange={(e: any) => setTargetPath(e?.nativeEvent?.text ?? "")}
            placeholder="例如：albums/2024"
          />

          <Button onPress={handleSelectFile} icon={<Ionicons name="document-attach" size={18} color="#fff" />}>
            选择文件
          </Button>

          {selectedFile && (
            <Text color="$text" numberOfLines={2}>已选择：{selectedFile.name}</Text>
          )}

          <Button
            backgroundColor="$primary"
            onPress={handleUpload}
            disabled={!selectedFile || uploadFile.isPending}
            icon={uploadFile.isPending ? <Spinner size="small" color="#fff" /> : <Ionicons name="cloud-upload" size={18} color="#fff" />}
          >
            {uploadFile.isPending ? '上传中...' : '开始上传'}
          </Button>
        </YStack>
      </Card>
    </YStack>
  )
}
