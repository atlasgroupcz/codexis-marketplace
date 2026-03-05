"use client";

import { useEffect, useRef, useState } from "react";
import {  renderAsync } from "docx-preview";
import { cn } from "@workspace/ui/lib/utils";
import type {Options} from "docx-preview";

export type DocxProps = {
  url: string;
  className?: string;
  options?: Partial<Options>;
};

export const Docx = ({ url, className, options }: DocxProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadDocument = async () => {
      if (!containerRef.current) return;

      setLoading(true);
      setError(null);

      try {
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to fetch: ${response.statusText}`);
        }

        const arrayBuffer = await response.arrayBuffer();
        containerRef.current.innerHTML = "";

        await renderAsync(arrayBuffer, containerRef.current, undefined, {
          className: "docx-preview",
          breakPages: true,
          renderHeaders: true,
          renderFooters: true,
          ...options,
        });
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to load document"
        );
      } finally {
        setLoading(false);
      }
    };

    loadDocument();
  }, [url, options]);

  return (
    <div className={cn("size-full overflow-auto", className)}>
      {loading && (
        <div className="flex items-center justify-center h-full text-muted-foreground">
          Loading...
        </div>
      )}
      {error && <div className="text-destructive p-4">{error}</div>}
      <div ref={containerRef} className={cn("[&_.docx-preview]:p-8 [&_.docx-preview-wrapper]:!bg-background", loading && "hidden")} />
    </div>
  );
};
