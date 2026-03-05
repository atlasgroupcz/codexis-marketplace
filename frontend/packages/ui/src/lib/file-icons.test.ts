import { describe, expect, it } from 'vitest'
import {
  FileArchiveIcon,
  FileAudioIcon,
  FileCode2Icon,
  FileIcon,
  FileImageIcon,
  FileJson2Icon,
  FileSpreadsheetIcon,
  FileTextIcon,
  FileVideoIcon,
  FolderIcon,
} from 'lucide-react'
import { getEntryIcon, getFileIcon } from './file-icons'

describe('getFileIcon', () => {
  describe('extension-based detection', () => {
    it('returns code icon for TypeScript files', () => {
      expect(getFileIcon(undefined, 'ts')).toBe(FileCode2Icon)
      expect(getFileIcon(undefined, 'tsx')).toBe(FileCode2Icon)
    })

    it('returns code icon for JavaScript files', () => {
      expect(getFileIcon(undefined, 'js')).toBe(FileCode2Icon)
      expect(getFileIcon(undefined, 'jsx')).toBe(FileCode2Icon)
    })

    it('returns code icon for other programming languages', () => {
      expect(getFileIcon(undefined, 'py')).toBe(FileCode2Icon)
      expect(getFileIcon(undefined, 'go')).toBe(FileCode2Icon)
      expect(getFileIcon(undefined, 'rs')).toBe(FileCode2Icon)
      expect(getFileIcon(undefined, 'java')).toBe(FileCode2Icon)
    })

    it('returns JSON icon for JSON files', () => {
      expect(getFileIcon(undefined, 'json')).toBe(FileJson2Icon)
      expect(getFileIcon(undefined, 'jsonc')).toBe(FileJson2Icon)
      expect(getFileIcon(undefined, 'json5')).toBe(FileJson2Icon)
    })

    it('returns archive icon for compressed files', () => {
      expect(getFileIcon(undefined, 'zip')).toBe(FileArchiveIcon)
      expect(getFileIcon(undefined, 'tar')).toBe(FileArchiveIcon)
      expect(getFileIcon(undefined, 'gz')).toBe(FileArchiveIcon)
      expect(getFileIcon(undefined, '7z')).toBe(FileArchiveIcon)
    })

    it('returns image icon for image files', () => {
      expect(getFileIcon(undefined, 'png')).toBe(FileImageIcon)
      expect(getFileIcon(undefined, 'jpg')).toBe(FileImageIcon)
      expect(getFileIcon(undefined, 'jpeg')).toBe(FileImageIcon)
      expect(getFileIcon(undefined, 'gif')).toBe(FileImageIcon)
      expect(getFileIcon(undefined, 'svg')).toBe(FileImageIcon)
    })

    it('returns video icon for video files', () => {
      expect(getFileIcon(undefined, 'mp4')).toBe(FileVideoIcon)
      expect(getFileIcon(undefined, 'webm')).toBe(FileVideoIcon)
      expect(getFileIcon(undefined, 'mov')).toBe(FileVideoIcon)
    })

    it('returns audio icon for audio files', () => {
      expect(getFileIcon(undefined, 'mp3')).toBe(FileAudioIcon)
      expect(getFileIcon(undefined, 'wav')).toBe(FileAudioIcon)
      expect(getFileIcon(undefined, 'ogg')).toBe(FileAudioIcon)
    })

    it('returns text icon for document files', () => {
      expect(getFileIcon(undefined, 'txt')).toBe(FileTextIcon)
      expect(getFileIcon(undefined, 'md')).toBe(FileTextIcon)
      expect(getFileIcon(undefined, 'pdf')).toBe(FileTextIcon)
      expect(getFileIcon(undefined, 'doc')).toBe(FileTextIcon)
    })

    it('returns spreadsheet icon for spreadsheet files', () => {
      expect(getFileIcon(undefined, 'xlsx')).toBe(FileSpreadsheetIcon)
      expect(getFileIcon(undefined, 'csv')).toBe(FileSpreadsheetIcon)
    })

    it('returns generic file icon for unknown extensions', () => {
      expect(getFileIcon(undefined, 'unknown')).toBe(FileIcon)
      expect(getFileIcon(undefined, 'xyz')).toBe(FileIcon)
    })

    it('handles case insensitivity', () => {
      expect(getFileIcon(undefined, 'TS')).toBe(FileCode2Icon)
      expect(getFileIcon(undefined, 'JSON')).toBe(FileJson2Icon)
      expect(getFileIcon(undefined, 'PNG')).toBe(FileImageIcon)
    })
  })

  describe('MIME type detection', () => {
    it('returns image icon for image MIME types', () => {
      expect(getFileIcon('image/png')).toBe(FileImageIcon)
      expect(getFileIcon('image/jpeg')).toBe(FileImageIcon)
      expect(getFileIcon('image/gif')).toBe(FileImageIcon)
      expect(getFileIcon('image/svg+xml')).toBe(FileImageIcon)
    })

    it('returns video icon for video MIME types', () => {
      expect(getFileIcon('video/mp4')).toBe(FileVideoIcon)
      expect(getFileIcon('video/webm')).toBe(FileVideoIcon)
    })

    it('returns audio icon for audio MIME types', () => {
      expect(getFileIcon('audio/mpeg')).toBe(FileAudioIcon)
      expect(getFileIcon('audio/wav')).toBe(FileAudioIcon)
    })

    it('returns code icon for code text types', () => {
      expect(getFileIcon('text/javascript')).toBe(FileCode2Icon)
      expect(getFileIcon('text/css')).toBe(FileCode2Icon)
      expect(getFileIcon('text/html')).toBe(FileCode2Icon)
    })

    it('returns text icon for plain text', () => {
      expect(getFileIcon('text/plain')).toBe(FileTextIcon)
    })

    it('returns JSON icon for JSON MIME types', () => {
      expect(getFileIcon('application/json')).toBe(FileJson2Icon)
      expect(getFileIcon('application/ld+json')).toBe(FileJson2Icon)
    })

    it('returns archive icon for archive MIME types', () => {
      expect(getFileIcon('application/zip')).toBe(FileArchiveIcon)
      expect(getFileIcon('application/x-tar')).toBe(FileArchiveIcon)
      expect(getFileIcon('application/gzip')).toBe(FileArchiveIcon)
    })

    it('returns text icon for document MIME types', () => {
      expect(getFileIcon('application/pdf')).toBe(FileTextIcon)
      expect(getFileIcon('application/msword')).toBe(FileTextIcon)
    })

    it('returns spreadsheet icon for spreadsheet MIME types', () => {
      expect(getFileIcon('application/vnd.ms-excel')).toBe(FileSpreadsheetIcon)
      expect(
        getFileIcon(
          'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        ),
      ).toBe(FileSpreadsheetIcon)
    })
  })

  describe('fallback behavior', () => {
    it('returns generic icon with no arguments', () => {
      expect(getFileIcon()).toBe(FileIcon)
    })

    it('returns generic icon for unknown MIME type without extension', () => {
      expect(getFileIcon('application/octet-stream')).toBe(FileIcon)
    })

    it('falls back to extension when MIME type is unknown', () => {
      expect(getFileIcon('application/octet-stream', 'ts')).toBe(FileCode2Icon)
    })
  })
})

describe('getEntryIcon', () => {
  it('returns folder icon for directories', () => {
    expect(getEntryIcon(true)).toBe(FolderIcon)
    expect(getEntryIcon(true, 'image/png', 'png')).toBe(FolderIcon)
  })

  it('returns file icon for files based on MIME type', () => {
    expect(getEntryIcon(false, 'image/png')).toBe(FileImageIcon)
    expect(getEntryIcon(false, 'application/json')).toBe(FileJson2Icon)
  })

  it('returns file icon for files based on extension', () => {
    expect(getEntryIcon(false, undefined, 'ts')).toBe(FileCode2Icon)
    expect(getEntryIcon(false, undefined, 'json')).toBe(FileJson2Icon)
  })
})
