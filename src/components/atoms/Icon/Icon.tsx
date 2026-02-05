import React from 'react'
import styles from './Icon.module.css'

export type IconName = 
  | 'mic' 
  | 'loader' 
  | 'check' 
  | 'x' 
  | 'key' 
  | 'settings' 
  | 'refresh' 
  | 'keyboard'
  | 'info'
  | 'chevron-down'

interface IconProps {
  name: IconName
  size?: number
  className?: string
}

const icons: Record<IconName, React.ReactNode> = {
  mic: (
    <>
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/>
      <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
      <line x1="12" x2="12" y1="19" y2="22"/>
    </>
  ),
  loader: <path d="M21 12a9 9 0 1 1-6.219-8.56"/>,
  check: <polyline points="20 6 9 17 4 12"/>,
  x: (
    <>
      <line x1="18" y1="6" x2="6" y2="18"/>
      <line x1="6" y1="6" x2="18" y2="18"/>
    </>
  ),
  key: (
    <>
      <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
      <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
    </>
  ),
  settings: (
    <>
      <circle cx="12" cy="12" r="3"/>
      <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
    </>
  ),
  refresh: (
    <>
      <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/>
      <path d="M21 3v5h-5"/>
    </>
  ),
  keyboard: (
    <>
      <rect width="20" height="14" x="2" y="5" rx="2"/>
      <path d="M6 9h.01M10 9h.01M14 9h.01M18 9h.01M6 13h.01M10 13h4M18 13h.01"/>
    </>
  ),
  info: (
    <>
      <circle cx="12" cy="12" r="10"/>
      <path d="M12 16v-4M12 8h.01"/>
    </>
  ),
  'chevron-down': <path d="M6 9l6 6 6-6"/>,
}

export function Icon({ name, size = 20, className = '' }: IconProps) {
  const isSpinning = name === 'loader'
  
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={`${styles.icon} ${isSpinning ? styles.spinning : ''} ${className}`}
    >
      {icons[name]}
    </svg>
  )
}
