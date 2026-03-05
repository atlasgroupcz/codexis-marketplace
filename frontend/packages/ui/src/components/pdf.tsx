import { useVirtualizer, useWindowVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useRef, useState } from "react";
import "react-pdf/dist/Page/TextLayer.css";
import "react-pdf/dist/Page/AnnotationLayer.css";
import { Document, Page, pdfjs } from "react-pdf";
import { cn } from "@workspace/ui/lib/utils";
import type { Virtualizer } from "@tanstack/react-virtual";
import type { MutableRefObject, ReactNode } from "react";
import type { DocumentProps, PageProps } from "react-pdf";

pdfjs.GlobalWorkerOptions.workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).toString();

export type PdfProps = {
  url: string;
  zoom?: number;
  virtualizerRef?: MutableRefObject<
    Virtualizer<HTMLElement, Element> | Virtualizer<Window, Element> | undefined
  >;
  className?: string;
  pageDecorators?: (args: {
    pageNumber: number;
    pageWidth: number;
    pageHeight: number;
    zoom: number;
  }) => { beside?: ReactNode; overlay?: ReactNode } | null;
  onInit?: () => void;
  /** Scroll to this page (1-indexed) after the PDF loads */
  initialPage?: number;
};

const DEFAULT_PAGE_HEIGHT = 1000;
const GAP_4 = 16;

export const Pdf = ({
  url,
  virtualizerRef,
  zoom = 1,
  className,
  pageDecorators,
  onInit,
  initialPage,
}: PdfProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollContainer =
    containerRef.current && isElementScrollable(containerRef.current)
      ? containerRef.current
      : null;
  const scrollableParent =
    !scrollContainer && containerRef.current
      ? getClosestScrollableParent(containerRef.current)
      : null;
  const scrollElement = scrollContainer || scrollableParent;
  const [pagesCount, setPagesCount] = useState(0);
  const pageHeight = DEFAULT_PAGE_HEIGHT * zoom;
  const [shouldMeasure, setShouldMeasure] = useState(true);
  const [pageAspectRatio, setPageAspectRatio] = useState(0.77);
  const initialScrollDone = useRef(false);

  const virtualWindow = useWindowVirtualizer({
    estimateSize: () => pageHeight,
    count: pagesCount,
    enabled: !scrollElement,
    gap: GAP_4,
  });

  const virtualContainer = useVirtualizer({
    getScrollElement: () => scrollElement,
    estimateSize: () => pageHeight,
    count: pagesCount,
    gap: GAP_4,
  });

  const virtual = scrollElement ? virtualContainer : virtualWindow;
  const virtualItems = virtual.getVirtualItems();
  const totalSize = virtual.getTotalSize();

  useEffect(() => {
    virtual.measure();
  }, [pageHeight, virtual]);

  useEffect(() => {
    if (virtualizerRef) {
      virtualizerRef.current = virtual;
    }
  }, [virtualizerRef, virtual]);

  const onLoadSuccess: DocumentProps["onLoadSuccess"] = async (pdf) => {
    setPagesCount(pdf.numPages);

    try {
      const firstPage = await pdf.getPage(1);
      const viewport = firstPage.getViewport({ scale: 1 });
      const ratio = viewport.width / viewport.height;
      setPageAspectRatio(ratio);
    } catch (error) {
      console.warn("Failed to get page dimensions, using default ratio", error);
    }
  };

  const measure: PageProps["onRenderSuccess"] = () => {
    if (!shouldMeasure) {
      return;
    }

    virtual.measure();
    setShouldMeasure(false);
  };

  const onGetAnnotationsSuccess: PageProps["onGetAnnotationsSuccess"] = (
    annotations
  ) => {
    annotations.forEach((annotation) => {
      if (annotation.subtype === "Link") {
        annotation.url = annotation.unsafeUrl;
      }
    });
  };

  const onRenderSuccess: PageProps["onRenderSuccess"] = (page) => {
    const index = page._pageIndex;
    if (index === 0 && shouldMeasure) {
      measure(page);
      onInit?.();

      // Scroll to initial page after first render and measurement
      if (initialPage && initialPage > 1 && !initialScrollDone.current) {
        initialScrollDone.current = true;
        // Use requestAnimationFrame to ensure virtualizer has settled after measure
        requestAnimationFrame(() => {
          virtual.scrollToIndex(initialPage - 1, { align: "start" });
        });
      }
    }
  };

  return (
    <Document
      key={url}
      externalLinkTarget="_blank"
      className={cn("size-full overflow-x-auto overflow-y-auto", className)}
      file={url}
      inputRef={containerRef}
      loading={<div />}
      onLoadSuccess={onLoadSuccess}
    >
      <div
        style={{ height: totalSize, paddingTop: virtualItems[0]?.start }}
        className={`flex flex-col gap-4 selection:bg-blue-400/25 selection:text-inherit`}
      >
        {virtualItems.map((item) => {
          const pageNumber = item.index + 1;
          const pageWidth = pageHeight * pageAspectRatio;
          const decorators = pageDecorators?.({
            pageNumber,
            pageWidth,
            pageHeight,
            zoom,
          });

          return (
            <div
              key={item.key}
              id={`pdf-page-${pageNumber}`}
              style={{ height: pageHeight }}
              data-type="page"
              data-page={pageNumber}
              className="flex gap-1 p-1"
            >
              <div className="relative mx-auto w-fit">
                <Page
                  loading={<div />}
                  onRenderSuccess={onRenderSuccess}
                  onGetAnnotationsSuccess={onGetAnnotationsSuccess}
                  className="overflow-clip rounded-md shadow-md [&_.highlightAnnotation]:hidden [&_.linkAnnotation]:bg-blue-500/20"
                  height={pageHeight}
                  pageIndex={item.index}
                />
                {decorators?.overlay}
              </div>
              {decorators?.beside}
            </div>
          );
        })}
      </div>
    </Document>
  );
};

const isElementScrollable = (element: HTMLElement): boolean => {
  const overflowY = window.getComputedStyle(element).overflowY;
  const isOverflowScrollable = overflowY === "auto" || overflowY === "scroll";
  const hasScrollableContent = element.scrollHeight > element.clientHeight;

  return isOverflowScrollable && hasScrollableContent;
};

const getClosestScrollableParent = (element: HTMLElement) => {
  let parent = element.parentElement;

  while (parent) {
    const overflowY = window.getComputedStyle(parent).overflowY;

    if (overflowY === "auto" || overflowY === "scroll") {
      return parent;
    }
    parent = parent.parentElement;
  }

  return null;
};
