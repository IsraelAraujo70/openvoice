import { useEffect, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { AppState, Config, AudioDevice } from '@/types'

export function useTauri() {
  const [state, setState] = useState<AppState>('idle')
  const [config, setConfig] = useState<Config | null>(null)
  const [devices, setDevices] = useState<AudioDevice[]>([])
  const [preview, setPreview] = useState('')
  const [error, setError] = useState('')

  const loadConfig = useCallback(async () => {
    try {
      const cfg = await invoke<Config>('load_config')
      setConfig(cfg)
      if (cfg.audio_device) {
        await invoke('set_audio_device', { deviceName: cfg.audio_device })
      }
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }, [])

  const loadDevices = useCallback(async () => {
    try {
      const devs = await invoke<AudioDevice[]>('get_audio_devices')
      setDevices(devs)
    } catch (e) {
      console.error('Failed to load devices:', e)
    }
  }, [])

  const saveConfig = useCallback(async (newConfig: Config) => {
    try {
      await invoke('save_config', { config: newConfig })
      await invoke('set_audio_device', { deviceName: newConfig.audio_device })
      setConfig(newConfig)
      return true
    } catch (e) {
      console.error('Failed to save config:', e)
      return false
    }
  }, [])

  const toggleRecording = useCallback(async () => {
    try {
      await invoke('toggle_recording')
    } catch (e) {
      console.error('Failed to toggle recording:', e)
    }
  }, [])

  useEffect(() => {
    loadConfig()
    loadDevices()

    const unlisteners: (() => void)[] = []

    const setupListeners = async () => {
      unlisteners.push(
        await listen('recording-started', () => {
          setState('recording')
          setPreview('')
          setError('')
        }),
        await listen('recording-stopped', () => {
          setState('processing')
        }),
        await listen('transcription-started', () => {
          setState('processing')
        }),
        await listen<string>('transcription-complete', (e) => {
          setState('success')
          const text = e.payload
          setPreview(text.length > 50 ? text.slice(0, 50) + '...' : text)
        }),
        await listen<string>('transcription-error', (e) => {
          setState('error')
          setError(e.payload.slice(0, 40))
        }),
        await listen('config-updated', () => {
          loadConfig()
        })
      )
    }

    setupListeners()

    return () => {
      unlisteners.forEach(fn => fn())
    }
  }, [loadConfig, loadDevices])

  return {
    state,
    config,
    devices,
    preview,
    error,
    loadConfig,
    loadDevices,
    saveConfig,
    toggleRecording,
  }
}
