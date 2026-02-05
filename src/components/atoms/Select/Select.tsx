import React from 'react'
import { Icon } from '../Icon'
import styles from './Select.module.css'

interface SelectOption {
  value: string
  label: string
}

interface SelectProps {
  label?: string
  options: SelectOption[]
  value: string
  onChange: (value: string) => void
  placeholder?: string
  icon?: React.ReactNode
  action?: React.ReactNode
}

export function Select({ 
  label, 
  options, 
  value, 
  onChange, 
  placeholder = 'Select...',
  icon,
  action
}: SelectProps) {
  return (
    <div className={styles.wrapper}>
      {label && (
        <div className={styles.labelRow}>
          <label className={styles.label}>{label}</label>
          {action}
        </div>
      )}
      <div className={styles.selectContainer}>
        {icon && <span className={styles.icon}>{icon}</span>}
        <select 
          className={`${styles.select} ${icon ? styles.withIcon : ''}`}
          value={value}
          onChange={(e) => onChange(e.target.value)}
        >
          <option value="">{placeholder}</option>
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        <Icon name="chevron-down" size={16} className={styles.chevron} />
      </div>
    </div>
  )
}
