---
uuid: 5c685f5d-ab83-464b-97df-45d5e9ca98e3
name: ocr
description: >-
  Convert binary documents to markdown using OCR / document conversion.
  Supported formats: PDF, DOC, DOCX, PPTX, XLS, XLSX, ODS, ODT, RTF, HTML,
  images (PNG, JPG, TIFF, BMP, GIF, WEBP), ZIP archives.
  Triggers on "ocr", "convert to markdown", "read PDF", "extract text from document",
  "convert document", "read this file" (for binary files).
allowed-tools: shell
i18n:
  cs:
    displayName: "OCR dokumentů"
    summary: "Převod dokumentů (PDF, DOCX, PPTX, obrázky, ZIP) do Markdownu pro další AI zpracování."
  en:
    displayName: "Document OCR"
    summary: "Convert documents (PDF, DOCX, PPTX, images, ZIP) to Markdown for downstream AI processing."
  sk:
    displayName: "OCR dokumentov"
    summary: "Prevod dokumentov (PDF, DOCX, PPTX, obrázky, ZIP) do Markdownu pre ďalšie AI spracovanie."
---

# OCR — Document to Markdown Conversion

Convert binary documents to Markdown text using the `ocr` command available in the shell.

## Usage

```bash
ocr <absolute-path-to-file>
```

## Supported Formats

- **Documents:** PDF, DOC, DOCX, PPTX, XLS, XLSX, ODS, ODT, RTF
- **Web:** HTML, HTM
- **Images:** PNG, JPG, TIFF, BMP, GIF, WEBP
- **Archives:** ZIP (all contained files are converted automatically — do NOT unzip first)

## When to Use

- User asks to read/extract text from a binary document (PDF, DOCX, image, etc.)
- User asks to convert a document to Markdown
- You need to read the contents of a binary file that `readFile` cannot handle

## When NOT to Use

- Plain text files (.txt, .csv, .json, .xml, .md, source code) — use `readFile` instead
- The file is already in a readable text format

## How It Works

1. Run `ocr /path/to/file.pdf`
2. The command returns JSON: `{"path": "/path/to/file.pdf-ocr.zip"}`
3. The result ZIP is saved next to the source file as `{filename}-ocr.zip`
4. Unzip the result and read the Markdown files inside

## Example

```bash
# Convert a PDF to markdown
ocr /home/codexis/report.pdf
# Returns: {"path": "/home/codexis/report.pdf-ocr.zip"}

# Unzip and read the result
unzip -o /home/codexis/report.pdf-ocr.zip -d /home/codexis/report-ocr
cat /home/codexis/report-ocr/*.md
```

## Error Handling

On error, the command returns JSON with an error message:
```json
{"error": "Permission denied or file not found: /home/codexis/missing.pdf"}
```
