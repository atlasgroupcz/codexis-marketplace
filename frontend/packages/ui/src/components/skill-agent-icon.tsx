import { useState } from 'react'
import { backendUrl } from '@workspace/utils/url'

interface SkillAgentIconProps {
  iconPath?: string | null
  alt?: string
  className?: string
}

export function SkillAgentIcon({
  iconPath,
  alt = '',
  className = 'size-4 rounded-sm object-contain',
}: SkillAgentIconProps) {
  const [failed, setFailed] = useState(false)

  if (!iconPath || failed) {
    return null
  }

  const src = backendUrl(
    `/rest/files/download?path=${encodeURIComponent(iconPath)}`,
  )

  return (
    <img
      src={src}
      alt={alt}
      className={className}
      onError={() => setFailed(true)}
    />
  )
}
