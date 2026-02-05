import styles from './IconBox.module.css'
import type { AppState } from '@/types'

interface IconBoxProps {
  state: AppState
  children: React.ReactNode
}

export function IconBox({ state, children }: IconBoxProps) {
  return (
    <div className={`${styles.box} ${styles[state]}`}>
      {children}
    </div>
  )
}
