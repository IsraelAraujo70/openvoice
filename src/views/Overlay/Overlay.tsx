import { Icon, IconBox, Text, RecordingDot, Button } from '@/components/atoms'
import type { IconName } from '@/components/atoms'
import { useTauri } from '@/hooks'
import type { AppState } from '@/types'
import styles from './Overlay.module.css'

const stateConfig: Record<AppState, { icon: IconName; title: string; hint: string }> = {
  idle: { icon: 'mic', title: 'Ready', hint: 'Click Start Recording' },
  recording: { icon: 'mic', title: 'Recording', hint: 'Click Stop Recording' },
  processing: { icon: 'loader', title: 'Processing...', hint: 'Please wait' },
  success: { icon: 'check', title: 'Copied to clipboard', hint: 'Ready for another recording' },
  error: { icon: 'x', title: 'Error', hint: 'Try again' },
}

export function Overlay() {
  const { state, preview, error, toggleRecording } = useTauri()
  const { icon, title, hint } = stateConfig[state]
  const isRecording = state === 'recording'
  const isProcessing = state === 'processing'

  return (
    <div className={`${styles.container} ${styles[state]}`}>
      <IconBox state={state}>
        <Icon name={icon} size={24} />
      </IconBox>

      {state === 'recording' && (
        <div className={styles.recordingIndicator}>
          <RecordingDot />
        </div>
      )}

      <Text variant="title">{title}</Text>
      
      {hint && <Text variant="caption" color="secondary">{hint}</Text>}
      {error && <Text variant="caption" color="error">{error}</Text>}

      <div className={styles.actions}>
        <Button onClick={toggleRecording} disabled={isProcessing}>
          {isRecording ? 'Stop Recording' : 'Start Recording'}
        </Button>
      </div>
      
      {preview && (
        <div className={styles.preview}>
          <Text variant="mono" color="secondary">{preview}</Text>
        </div>
      )}
    </div>
  )
}
