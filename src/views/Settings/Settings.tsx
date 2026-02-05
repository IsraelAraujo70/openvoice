import { useState, useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { emit } from '@tauri-apps/api/event'
import { Icon, Text, Button, Input, Select } from '@/components/atoms'
import { useTauri } from '@/hooks'
import styles from './Settings.module.css'

export function Settings() {
  const { config, devices, loadDevices, saveConfig } = useTauri()
  
  const [apiKey, setApiKey] = useState('')
  const [audioDevice, setAudioDevice] = useState('')
  const [status, setStatus] = useState<{ type: 'success' | 'error'; message: string } | null>(null)

  useEffect(() => {
    if (config) {
      setApiKey(config.api_key || '')
      setAudioDevice(config.audio_device || '')
    }
  }, [config])

  const showStatus = (type: 'success' | 'error', message: string) => {
    setStatus({ type, message })
    setTimeout(() => setStatus(null), 3000)
  }

  const handleSave = async () => {
    const newConfig = {
      api_key: apiKey.trim() || null,
      audio_device: audioDevice || null,
      model: null,
    }
    
    const success = await saveConfig(newConfig)
    if (success) {
      await emit('config-updated')
      showStatus('success', 'Settings saved!')
    } else {
      showStatus('error', 'Failed to save settings')
    }
  }

  const handleClose = async () => {
    const win = getCurrentWindow()
    await win.hide()
  }

  // Close on window close request
  useEffect(() => {
    const win = getCurrentWindow()
    const unlisten = win.onCloseRequested(async (e) => {
      e.preventDefault()
      await win.hide()
    })
    return () => { unlisten.then(fn => fn()) }
  }, [])

  const deviceOptions = devices.map(d => ({
    value: d.name,
    label: d.name + (d.is_default ? ' (default)' : '')
  }))

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <div className={styles.logo}>
          <Icon name="mic" size={24} />
        </div>
        <Text variant="title" as="h1">OpenVoice</Text>
        <Text variant="caption" color="secondary">Voice-to-clipboard</Text>
      </header>

      <div className={styles.card}>
        <div className={styles.field}>
          <Input
            label="API Key"
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="sk-or-v1-..."
            icon={<Icon name="key" size={16} />}
          />
        </div>

        <div className={styles.field}>
          <Select
            label="Audio Device"
            options={deviceOptions}
            value={audioDevice}
            onChange={setAudioDevice}
            placeholder="Default device"
            icon={<Icon name="mic" size={16} />}
            action={
              <Button variant="ghost" onClick={loadDevices}>
                <Icon name="refresh" size={14} />
                Refresh
              </Button>
            }
          />
        </div>

        <div className={styles.actions}>
          <Button onClick={handleSave}>Save Settings</Button>
          <Button variant="secondary" onClick={handleClose}>Close</Button>
        </div>

        {status && (
          <div className={`${styles.status} ${styles[status.type]}`}>
            <Icon name={status.type === 'success' ? 'check' : 'x'} size={16} />
            <Text variant="caption">{status.message}</Text>
          </div>
        )}
      </div>

      <div className={styles.info}>
        <div className={styles.infoHeader}>
          <Icon name="info" size={16} />
          <Text variant="caption" color="secondary">Quick Start</Text>
        </div>
        <ol className={styles.steps}>
          <li>
            <Text variant="caption" color="secondary">
              Get API key from{' '}
              <a href="https://openrouter.ai/keys" target="_blank" rel="noopener">
                openrouter.ai/keys
              </a>
            </Text>
          </li>
          <li>
            <Text variant="caption" color="secondary">
              Click Start Recording in the main window
            </Text>
          </li>
          <li>
            <Text variant="caption" color="secondary">Click Stop Recording to finish</Text>
          </li>
          <li>
            <Text variant="caption" color="secondary">Text is copied to clipboard</Text>
          </li>
        </ol>
      </div>
    </div>
  )
}
