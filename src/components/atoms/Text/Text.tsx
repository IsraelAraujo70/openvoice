import React from 'react'
import styles from './Text.module.css'

type TextVariant = 'title' | 'body' | 'label' | 'caption' | 'mono'
type TextColor = 'primary' | 'secondary' | 'tertiary' | 'accent' | 'success' | 'error' | 'warning'

interface TextProps {
  variant?: TextVariant
  color?: TextColor
  children: React.ReactNode
  className?: string
  as?: 'span' | 'p' | 'h1' | 'h2' | 'h3' | 'label'
}

export function Text({ 
  variant = 'body', 
  color = 'primary', 
  children, 
  className = '',
  as: Component = 'span'
}: TextProps) {
  return (
    <Component className={`${styles.text} ${styles[variant]} ${styles[color]} ${className}`}>
      {children}
    </Component>
  )
}
