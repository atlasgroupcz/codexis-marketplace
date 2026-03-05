"use client";

import { useEffect, useState } from "react";
import * as XLSX from "xlsx";
import DOMPurify from "dompurify";
import { cn } from "@workspace/ui/lib/utils";

export type SpreadsheetProps = {
  url: string;
  className?: string;
};

type SheetData = {
  name: string;
  html: string;
};

export const Spreadsheet = ({ url, className }: SpreadsheetProps) => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sheets, setSheets] = useState<Array<SheetData>>([]);
  const [activeSheet, setActiveSheet] = useState(0);

  useEffect(() => {
    const loadSpreadsheet = async () => {
      setLoading(true);
      setError(null);

      try {
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to fetch: ${response.statusText}`);
        }

        const arrayBuffer = await response.arrayBuffer();
        const workbook = XLSX.read(arrayBuffer, { type: "array" });

        const sheetData: Array<SheetData> = workbook.SheetNames
          .filter((name) => workbook.Sheets[name] !== undefined)
          .map((name) => {
            const worksheet = workbook.Sheets[name]!;
            const rawHtml = XLSX.utils.sheet_to_html(worksheet, {
              id: `sheet-${name.replace(/\s+/g, "-")}`,
            });
            // Sanitize HTML to prevent XSS attacks
            const sanitizedHtml = DOMPurify.sanitize(rawHtml, {
              USE_PROFILES: { html: true },
              ALLOWED_TAGS: ["table", "thead", "tbody", "tfoot", "tr", "th", "td", "colgroup", "col"],
              ALLOWED_ATTR: ["id", "class", "colspan", "rowspan", "style"],
            });
            return { name, html: sanitizedHtml };
          });

        setSheets(sheetData);
        setActiveSheet(0);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to load spreadsheet"
        );
      } finally {
        setLoading(false);
      }
    };

    loadSpreadsheet();
  }, [url]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        Loading...
      </div>
    );
  }

  if (error) {
    return <div className="text-destructive p-4">{error}</div>;
  }

  if (sheets.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        No sheets found
      </div>
    );
  }

  const hasMultipleSheets = sheets.length > 1;

  return (
    <div className={cn("flex flex-col h-full w-full overflow-hidden", className)}>
      {/* Spreadsheet content - use relative/absolute for proper scroll containment */}
      <div className="flex-1 min-h-0 relative">
        <div className="absolute inset-0 overflow-auto bg-background">
          <div
            className={cn(
              "spreadsheet-preview",
              "[&_table]:border-collapse [&_table]:text-sm",
              "[&_td]:border [&_td]:border-border [&_td]:px-2 [&_td]:py-1 [&_td]:text-left [&_td]:align-top [&_td]:whitespace-nowrap",
              "[&_th]:border [&_th]:border-border [&_th]:px-2 [&_th]:py-1 [&_th]:text-left [&_th]:font-medium [&_th]:bg-muted [&_th]:whitespace-nowrap",
              "[&_tr:nth-child(even)]:bg-muted/30",
              "[&_tr:first-child_td]:font-medium [&_tr:first-child_td]:bg-muted"
            )}
            dangerouslySetInnerHTML={{ __html: sheets[activeSheet]?.html ?? "" }}
          />
        </div>
      </div>

      {/* Bottom tabs for multiple sheets (Excel-style) */}
      {hasMultipleSheets && (
        <div className="flex items-center gap-1 px-2 py-1.5 border-t bg-muted/50 overflow-x-auto shrink-0">
          {sheets.map((sheet, index) => (
            <button
              key={sheet.name}
              type="button"
              onClick={() => setActiveSheet(index)}
              className={cn(
                "px-3 py-1 text-xs font-medium rounded border transition-colors whitespace-nowrap",
                index === activeSheet
                  ? "bg-background border-border text-foreground shadow-sm"
                  : "bg-transparent border-transparent text-muted-foreground hover:bg-background/50 hover:text-foreground"
              )}
            >
              {sheet.name}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
