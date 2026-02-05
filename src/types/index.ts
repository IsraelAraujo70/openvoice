export type AppState = 'idle' | 'recording' | 'processing' | 'success' | 'error'

export interface AudioDevice {
  name: string
  is_default: boolean
}

export interface Config {
  api_key: string | null
  audio_device: string | null
  model: string | null
}
