"use client";

import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import {  cva } from "class-variance-authority";
import { PanelLeft } from "lucide-react";

import { Button } from "@workspace/ui/components/button";
import { Input } from "@workspace/ui/components/input";
import { Separator } from "@workspace/ui/components/separator";
import { Sheet, SheetContent } from "@workspace/ui/components/sheet";
import { Skeleton } from "@workspace/ui/components/skeleton";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@workspace/ui/components/tooltip";
import { useIsMobile } from "@workspace/ui/hooks/use-mobile";
import { cn } from "@workspace/ui/lib/utils";
import type {VariantProps} from "class-variance-authority";

export interface UseSidebarResizeProps {
  /**
   * Direction of the resize handle
   * - 'left': Handle is on left side (for right-positioned panels)
   * - 'right': Handle is on right side (for left-positioned panels)
   */
  direction?: "left" | "right";

  /**
   * Current width of the panel
   */
  currentWidth: string;

  /**
   * Callback to update width when resizing
   */
  onResize: (width: string) => void;

  /**
   * Callback to toggle panel visibility
   */
  onToggle?: () => void;

  /**
   * Whether the panel is currently collapsed
   */
  isCollapsed?: boolean;

  /**
   * Minimum resize width
   */
  minResizeWidth?: string;

  /**
   * Maximum resize width
   */
  maxResizeWidth?: string;

  /**
   * Whether to enable auto-collapse when dragged below threshold
   */
  enableAutoCollapse?: boolean;

  /**
   * Auto-collapse threshold as percentage of minResizeWidth
   * A value of 1.0 means the panel will collapse when dragged to minResizeWidth
   * A value of 0.5 means the panel will collapse when dragged to 50% of minResizeWidth
   * A value of 1.5 means the panel will collapse when dragged to 50% beyond minResizeWidth
   * Can be any positive number, not limited to the range 0.0-1.0
   */
  autoCollapseThreshold?: number;

  /**
   * Threshold to expand when dragging in opposite direction (0.0-1.0)
   * Percentage of distance needed to drag back to expand
   */
  expandThreshold?: number;

  /**
   * Whether to enable drag functionality
   */
  enableDrag?: boolean;

  /**
   * Callback to update dragging rail state
   */
  setIsDraggingRail?: (isDragging: boolean) => void;

  /**
   * Cookie name for persisting width
   */
  widthCookieName?: string;

  /**
   * Cookie max age in seconds
   */
  widthCookieMaxAge?: number;

  /**
   * Whether this is a nested sidebar (not at the edge of the screen)
   */
  isNested?: boolean;

  /**
   * Whether to enable toggle functionality
   */
  enableToggle?: boolean;
}

interface WidthUnit {
  value: number;
  unit: "rem" | "px";
}

/**
 * Parse width string into value and unit
 */
function parseWidth(width: string): WidthUnit {
  const unit = width.endsWith("rem") ? "rem" : "px";
  const value = Number.parseFloat(width);
  return { value, unit };
}

/**
 * Convert any width to pixels for calculations
 */
function toPx(width: string): number {
  const { value, unit } = parseWidth(width);
  return unit === "rem" ? value * 16 : value;
}

/**
 * Format width value with unit
 */
function formatWidth(value: number, unit: "rem" | "px"): string {
  return `${unit === "rem" ? value.toFixed(1) : Math.round(value)}${unit}`;
}

/**
 * A versatile hook for handling resizable sidebar (or inset) panels
 * Works for both sidebar (left side) and artifacts (right side) panels
 * Supports VS Code-like continuous drag to collapse/expand
 */
export function useSidebarResize({
  direction = "right",
  currentWidth,
  onResize,
  onToggle,
  isCollapsed = false,
  minResizeWidth = "14rem",
  maxResizeWidth = "24rem",
  enableToggle = true,
  enableAutoCollapse = true,
  autoCollapseThreshold = 1.5, // Default to collapsing at minWidth + 50%
  expandThreshold = 0.2,
  enableDrag = true,
  setIsDraggingRail = () => {},
  widthCookieName,
  widthCookieMaxAge = 60 * 60 * 24 * 7, // 1 week default
  isNested = false,
}: UseSidebarResizeProps) {
  // Refs for tracking drag state
  const dragRef = React.useRef<HTMLButtonElement>(null);
  const startWidth = React.useRef(0);
  const startX = React.useRef(0);
  const isDragging = React.useRef(false);
  const isInteractingWithRail = React.useRef(false);
  const lastWidth = React.useRef(0);
  const lastLoggedWidth = React.useRef(0);
  const dragStartPoint = React.useRef(0);
  const lastDragDirection = React.useRef<"expand" | "collapse" | null>(null);
  const lastTogglePoint = React.useRef(0);
  const lastToggleWidth = React.useRef(0);
  const toggleCooldown = React.useRef(false);
  const lastToggleTime = React.useRef(0);
  const dragDistanceFromToggle = React.useRef(0);
  const dragOffset = React.useRef(0);
  const railRect = React.useRef<DOMRect | null>(null);

  // Refs for auto-collapse threshold
  const autoCollapseThresholdPx = React.useRef(0);

  // Memoize min/max width calculations for performance
  const minWidthPx = React.useMemo(
    () => toPx(minResizeWidth),
    [minResizeWidth]
  );
  const maxWidthPx = React.useMemo(
    () => toPx(maxResizeWidth),
    [maxResizeWidth]
  );

  // Helper function to determine if width is increasing based on direction and mouse movement
  const isIncreasingWidth = React.useCallback(
    (currentX: number, referenceX: number): boolean => {
      return direction === "left"
        ? currentX < referenceX // For left-positioned handle, moving left increases width
        : currentX > referenceX; // For right-positioned handle, moving right increases width
    },
    [direction]
  );

  // Helper function to calculate width based on mouse position and direction
  const calculateWidth = React.useCallback(
    (
      e: MouseEvent,
      initialX: number,
      initialWidth: number,
      currentRailRect: DOMRect | null
    ): number => {
      if (isNested && currentRailRect) {
        // For nested sidebars, use the delta from start position for precise tracking
        const deltaX = e.clientX - initialX;

        if (direction === "left") {
          // For left-positioned handle (right panel)
          // Width increases as mouse moves left (negative deltaX)
          return initialWidth - deltaX;
        }
        // For right-positioned handle (left panel)
        // Width increases as mouse moves right (positive deltaX)
        return initialWidth + deltaX;
      }
      // For standard sidebars at window edges
      if (direction === "left") {
        // For left-positioned handle (right panel)
        return window.innerWidth - e.clientX;
      }
      // For right-positioned handle (left panel)
      return e.clientX;
    },
    [direction, isNested]
  );

  // Update auto-collapse threshold when dependencies change
  React.useEffect(() => {
    autoCollapseThresholdPx.current = enableAutoCollapse
      ? minWidthPx * autoCollapseThreshold
      : 0;
  }, [minWidthPx, enableAutoCollapse, autoCollapseThreshold]);

  // Persist width to cookie if cookie name is provided
  const persistWidth = React.useCallback(
    (width: string) => {
      if (widthCookieName) {
        document.cookie = `${widthCookieName}=${width}; path=/; max-age=${widthCookieMaxAge}`;
      }
    },
    [widthCookieName, widthCookieMaxAge]
  );

  // Handle mouse down on resize handle
  const handleMouseDown = React.useCallback(
    (e: React.MouseEvent) => {
      isInteractingWithRail.current = true;

      if (!enableDrag) {
        return;
      }

      // Store initial state
      const currentWidthPx = isCollapsed ? 0 : toPx(currentWidth);
      startWidth.current = currentWidthPx;
      startX.current = e.clientX;
      dragStartPoint.current = e.clientX;
      lastWidth.current = currentWidthPx;
      lastLoggedWidth.current = currentWidthPx;
      lastTogglePoint.current = e.clientX;
      lastToggleWidth.current = currentWidthPx;
      lastDragDirection.current = null;
      toggleCooldown.current = false;
      lastToggleTime.current = 0;
      dragDistanceFromToggle.current = 0;

      // Reset drag offset
      dragOffset.current = 0;

      // Store the rail element's position for nested sidebars
      if (isNested && dragRef.current) {
        railRect.current = dragRef.current.getBoundingClientRect();
      } else {
        railRect.current = null;
      }

      e.preventDefault();
    },
    [enableDrag, isCollapsed, currentWidth, isNested]
  );

  // Handle mouse movement and resizing
  React.useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isInteractingWithRail.current) return;

      const deltaX = Math.abs(e.clientX - startX.current);
      if (!isDragging.current && deltaX > 5) {
        isDragging.current = true;
        setIsDraggingRail(true);
      }

      if (isDragging.current) {
        // Get unit for width calculations
        const { unit } = parseWidth(currentWidth);

        // Get current rail position for ultra-precise tracking
        let currentRailRect = railRect.current;
        if (isNested && dragRef.current) {
          currentRailRect = dragRef.current.getBoundingClientRect();
        }

        // Determine current drag direction
        const currentDragDirection = isIncreasingWidth(
          e.clientX,
          lastTogglePoint.current
        )
          ? "expand"
          : "collapse";

        // Update direction tracking
        if (lastDragDirection.current !== currentDragDirection) {
          lastDragDirection.current = currentDragDirection;
        }

        // Calculate distance from last toggle point
        dragDistanceFromToggle.current = Math.abs(
          e.clientX - lastTogglePoint.current
        );

        // Check for toggle cooldown (prevent rapid toggling)
        const now = Date.now();
        if (toggleCooldown.current && now - lastToggleTime.current > 200) {
          toggleCooldown.current = false;
        }

        // Handle toggling between collapsed and expanded states
        if (!toggleCooldown.current) {
          // Handle collapsing when expanded
          if (enableAutoCollapse && onToggle && !isCollapsed) {
            // Calculate precise width based on mouse position
            const currentDragWidth = calculateWidth(
              e,
              startX.current,
              startWidth.current,
              currentRailRect
            );

            // Determine if we should collapse based on threshold
            let shouldCollapse = false;

            if (autoCollapseThreshold <= 1.0) {
              // For thresholds <= 1.0, collapse when width is below minWidth * threshold
              shouldCollapse =
                currentDragWidth <= minWidthPx * autoCollapseThreshold;
            } else {
              // For thresholds > 1.0, we need to drag beyond minWidth by a certain amount
              if (currentDragWidth <= minWidthPx) {
                // Calculate how much beyond minWidth we need to drag
                const extraDragNeeded =
                  minWidthPx * (autoCollapseThreshold - 1.0);

                // Only collapse if we've dragged far enough beyond minWidth
                const distanceBeyondMin = minWidthPx - currentDragWidth;

                shouldCollapse = distanceBeyondMin >= extraDragNeeded;
              }
            }

            if (currentDragDirection === "collapse" && shouldCollapse) {
              onToggle(); // Collapse
              lastTogglePoint.current = e.clientX;
              lastToggleWidth.current = 0; // Width is 0 when collapsed
              toggleCooldown.current = true;
              lastToggleTime.current = now;
              return;
            }
          }

          // Handle expanding when collapsed
          if (
            onToggle &&
            isCollapsed &&
            currentDragDirection === "expand" &&
            dragDistanceFromToggle.current > minWidthPx * expandThreshold
          ) {
            onToggle(); // Expand

            // Calculate initial width based on exact mouse position
            const initialWidth = calculateWidth(
              e,
              startX.current,
              startWidth.current,
              currentRailRect
            );

            // Clamp to min/max
            const clampedWidth = Math.max(
              minWidthPx,
              Math.min(maxWidthPx, initialWidth)
            );

            // Set initial width when expanding
            const formattedWidth = formatWidth(
              unit === "rem" ? clampedWidth / 16 : clampedWidth,
              unit
            );
            onResize(formattedWidth);
            persistWidth(formattedWidth);

            lastTogglePoint.current = e.clientX;
            lastToggleWidth.current = clampedWidth;
            toggleCooldown.current = true;
            lastToggleTime.current = now;
            return;
          }
        }

        // Skip width calculations if panel is collapsed
        if (isCollapsed) {
          return;
        }

        // Calculate new width based on mouse position and drag direction
        const newWidthPx = calculateWidth(
          e,
          startX.current,
          startWidth.current,
          currentRailRect
        );

        // Clamp width between min and max
        const clampedWidthPx = Math.max(
          minWidthPx,
          Math.min(maxWidthPx, newWidthPx)
        );

        // Convert to the target unit
        const newWidth = unit === "rem" ? clampedWidthPx / 16 : clampedWidthPx;

        // Format and update width
        const formattedWidth = formatWidth(newWidth, unit);
        onResize(formattedWidth);
        persistWidth(formattedWidth);

        // Update last width
        lastWidth.current = clampedWidthPx;
      }
    };

    const handleMouseUp = () => {
      if (!isInteractingWithRail.current) return;

      // Handle click (not drag) behavior
      if (!isDragging.current && onToggle && enableToggle) {
        onToggle();
      }

      // Reset all state
      isDragging.current = false;
      isInteractingWithRail.current = false;
      lastWidth.current = 0;
      lastLoggedWidth.current = 0;
      lastDragDirection.current = null;
      lastTogglePoint.current = 0;
      lastToggleWidth.current = 0;
      toggleCooldown.current = false;
      lastToggleTime.current = 0;
      dragDistanceFromToggle.current = 0;
      dragOffset.current = 0;
      railRect.current = null;
      setIsDraggingRail(false);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [
    onResize,
    onToggle,
    isCollapsed,
    currentWidth,
    persistWidth,
    setIsDraggingRail,
    minWidthPx,
    maxWidthPx,
    isIncreasingWidth,
    calculateWidth,
    isNested,
    enableAutoCollapse,
    autoCollapseThreshold,
    expandThreshold,
    enableToggle,
  ]);

  return {
    dragRef,
    isDragging,
    handleMouseDown,
  };
}

const SIDEBAR_COOKIE_NAME = "sidebar:state";
const SIDEBAR_COOKIE_MAX_AGE = 60 * 60 * 24 * 7;
const SIDEBAR_WIDTH = "17rem";
const SIDEBAR_WIDTH_ICON = "3rem";
const SIDEBAR_KEYBOARD_SHORTCUT = "b";

const MIN_SIDEBAR_WIDTH = "17rem";
const MAX_SIDEBAR_WIDTH = "17rem";

const CHAT_HISTORY_COOKIE_NAME = "sidebar:chat-history";

type SidebarContext = {
  state: "expanded" | "collapsed";
  open: boolean;
  setOpen: (open: boolean) => void;
  openMobile: boolean;
  setOpenMobile: (open: boolean) => void;
  isMobile: boolean;
  toggleSidebar: () => void;
  width: string;
  setWidth: (width: string) => void;
  isDraggingRail: boolean;
  setIsDraggingRail: (isDragging: boolean) => void;
  chatHistoryOpen: boolean;
  setChatHistoryOpen: (open: boolean) => void;
  toggleChatHistory: () => void;
};

const SidebarContext = React.createContext<SidebarContext | null>(null);

function useSidebar() {
  const context = React.useContext(SidebarContext);
  if (!context) {
    throw new Error("useSidebar must be used within a SidebarProvider.");
  }

  return context;
}

const SidebarProvider = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div"> & {
    defaultOpen?: boolean;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    defaultWidth?: string;
    defaultChatHistoryOpen?: boolean;
  }
>(
  (
    {
      defaultOpen = true,
      open: openProp,
      onOpenChange: setOpenProp,
      className,
      style,
      children,
      defaultWidth = SIDEBAR_WIDTH,
      defaultChatHistoryOpen,
      ...props
    },
    ref
  ) => {
    const isMobile = useIsMobile();
    const [width, setWidth] = React.useState(defaultWidth);
    const [openMobile, setOpenMobile] = React.useState(false);
    const [isDraggingRail, setIsDraggingRail] = React.useState(false);

    // This is the internal state of the sidebar.
    // We use openProp and setOpenProp for control from outside the component.
    const [_open, _setOpen] = React.useState(defaultOpen);
    const open = openProp ?? _open;
    const setOpen = React.useCallback(
      (value: boolean | ((value: boolean) => boolean)) => {
        const openState = typeof value === "function" ? value(open) : value;
        if (setOpenProp) {
          setOpenProp(openState);
        } else {
          _setOpen(openState);
        }

        // This sets the cookie to keep the sidebar state.
        document.cookie = `${SIDEBAR_COOKIE_NAME}=${openState}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`;
      },
      [setOpenProp, open]
    );

    // Helper to toggle the sidebar.
    const toggleSidebar = React.useCallback(() => {
      return isMobile
        ? setOpenMobile((previousOpen) => !previousOpen)
        : setOpen((previousOpen) => !previousOpen);
    }, [isMobile, setOpen]);

    // Chat history panel state
    const [chatHistoryOpen, _setChatHistoryOpen] = React.useState(
      defaultChatHistoryOpen ?? true
    );

    const setChatHistoryOpen = React.useCallback(
      (value: boolean) => {
        _setChatHistoryOpen(value);
        document.cookie = `${CHAT_HISTORY_COOKIE_NAME}=${value}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`;
      },
      []
    );

    const toggleChatHistory = React.useCallback(() => {
      setChatHistoryOpen(!chatHistoryOpen);
    }, [chatHistoryOpen, setChatHistoryOpen]);

    // Adds a keyboard shortcut to toggle the sidebar.
    React.useEffect(() => {
      const handleKeyDown = (event: KeyboardEvent) => {
        if (
          event.key === SIDEBAR_KEYBOARD_SHORTCUT &&
          (event.metaKey || event.ctrlKey)
        ) {
          event.preventDefault();
          toggleSidebar();
        }
      };

      window.addEventListener("keydown", handleKeyDown);
      return () => window.removeEventListener("keydown", handleKeyDown);
    }, [toggleSidebar]);

    // We add a state so that we can do data-state="expanded" or "collapsed".
    // This makes it easier to style the sidebar with Tailwind classes.
    const state = open ? "expanded" : "collapsed";

    const contextValue = React.useMemo<SidebarContext>(
      () => ({
        state,
        open,
        setOpen,
        isMobile,
        openMobile,
        setOpenMobile,
        toggleSidebar,
        width,
        setWidth,
        isDraggingRail,
        setIsDraggingRail,
        chatHistoryOpen,
        setChatHistoryOpen,
        toggleChatHistory,
      }),
      [
        state,
        open,
        setOpen,
        isMobile,
        openMobile,
        toggleSidebar,
        width,
        isDraggingRail,
        chatHistoryOpen,
        setChatHistoryOpen,
        toggleChatHistory,
      ]
    );

    return (
      <SidebarContext.Provider value={contextValue}>
        <TooltipProvider delayDuration={0}>
          <div
            style={
              {
                "--sidebar-width": width,
                "--sidebar-width-icon": SIDEBAR_WIDTH_ICON,
                ...style,
              } as React.CSSProperties
            }
            className={cn(
              "group/sidebar-wrapper flex h-svh w-full overflow-hidden has-data-[variant=inset]:bg-sidebar",
              className
            )}
            ref={ref}
            {...props}
          >
            {children}
          </div>
        </TooltipProvider>
      </SidebarContext.Provider>
    );
  }
);
SidebarProvider.displayName = "SidebarProvider";

const Sidebar = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div"> & {
    side?: "left" | "right";
    variant?: "sidebar" | "floating" | "inset";
    collapsible?: "offcanvas" | "icon" | "none";
  }
>(
  (
    {
      side = "left",
      variant = "sidebar",
      collapsible = "offcanvas",
      className,
      children,
      ...props
    },
    ref
  ) => {
    const {
      isMobile,
      state,
      openMobile,
      setOpenMobile,
      isDraggingRail,
    } = useSidebar();

    if (collapsible === "none") {
      return (
        <div
          className={cn(
            "flex h-full w-(--sidebar-width) flex-col bg-sidebar text-sidebar-foreground",
            className
          )}
          ref={ref}
          {...props}
        >
          {children}
        </div>
      );
    }

    if (isMobile) {
      return (
        <Sheet open={openMobile} onOpenChange={setOpenMobile} {...props}>
          <SheetContent
            data-sidebar="sidebar"
            data-mobile="true"
            className="w-full max-w-full bg-sidebar p-0 text-sidebar-foreground [&>button]:hidden"
            side={side}
          >
            <div className="flex h-full w-full flex-col">{children}</div>
          </SheetContent>
        </Sheet>
      );
    }

    return (
      <div
        ref={ref}
        className="group peer hidden md:block text-sidebar-foreground"
        data-state={state}
        data-collapsible={state === "collapsed" ? collapsible : ""}
        data-variant={variant}
        data-side={side}
        data-dragging={isDraggingRail}
      >
        {/* This is what handles the sidebar gap on desktop */}
        <div
          className={cn(
            "duration-200 relative h-[calc(100svh-var(--header-height,0px))] w-(--sidebar-width) bg-transparent transition-[width] ease-linear",
            "group-data-[collapsible=offcanvas]:w-0",
            "group-data-[side=right]:rotate-180",
            variant === "floating" || variant === "inset"
              ? "group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]"
              : "group-data-[collapsible=icon]:w-(--sidebar-width-icon)",
            "group-data-[dragging=true]:duration-0! group-data-[dragging=true]_*:!duration-0"
          )}
        />
        <div
          className={cn(
            "duration-200 fixed top-(--header-height,0px) bottom-0 z-10 hidden h-[calc(100svh-var(--header-height,0px))] w-(--sidebar-width) transition-[left,right,width] ease-linear md:flex",
            side === "left"
              ? "left-0 group-data-[collapsible=offcanvas]:left-[calc(var(--sidebar-width)*-1)]"
              : "right-0 group-data-[collapsible=offcanvas]:right-[calc(var(--sidebar-width)*-1)]",
            // Adjust the padding for floating and inset variants.
            variant === "floating" || variant === "inset"
              ? "p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]"
              : "group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-r group-data-[side=right]:border-l",
            "group-data-[dragging=true]:duration-0! group-data-[dragging=true]_*:!duration-0",
            className
          )}
          {...props}
        >
          <div
            data-sidebar="sidebar"
            className="flex h-full w-full flex-col bg-sidebar group-data-[variant=floating]:rounded-lg group-data-[variant=floating]:border group-data-[variant=floating]:border-sidebar-border group-data-[variant=floating]:shadow-sm"
          >
            {children}
          </div>
        </div>
      </div>
    );
  }
);
Sidebar.displayName = "Sidebar";

function mergeButtonRefs<T extends HTMLButtonElement>(
  refs: Array<React.MutableRefObject<T> | React.LegacyRef<T>>
): React.RefCallback<T> {
  return (value) => {
    for (const ref of refs) {
      if (typeof ref === "function") {
        ref(value);
      } else if (ref != null) {
        (ref as React.MutableRefObject<T | null>).current = value;
      }
    }
  };
}

const SidebarTrigger = React.forwardRef<
  React.ElementRef<typeof Button>,
  React.ComponentProps<typeof Button>
>(({ className, onClick, ...props }, ref) => {
  const { toggleSidebar } = useSidebar();

  return (
    <Button
      ref={ref}
      data-sidebar="trigger"
      variant="ghost"
      size="icon"
      className={cn("h-7 w-7", className)}
      onClick={(event) => {
        onClick?.(event);
        toggleSidebar();
      }}
      {...props}
    >
      <PanelLeft />
      <span className="sr-only">Toggle Sidebar</span>
    </Button>
  );
});
SidebarTrigger.displayName = "SidebarTrigger";

const SidebarRail = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<"button"> & {
    enableDrag?: boolean;
  }
>(({ className, enableDrag = true, ...props }, ref) => {
  const { toggleSidebar, setWidth, state, width, setIsDraggingRail } =
    useSidebar();

  const { dragRef, handleMouseDown } = useSidebarResize({
    direction: "right",
    enableDrag,
    onResize: setWidth,
    onToggle: toggleSidebar,
    currentWidth: width,
    isCollapsed: state === "collapsed",
    minResizeWidth: MIN_SIDEBAR_WIDTH,
    maxResizeWidth: MAX_SIDEBAR_WIDTH,
    setIsDraggingRail,
    widthCookieName: "sidebar:width",
    widthCookieMaxAge: 60 * 60 * 24 * 7, // 1 week
  });

  const combinedRef = React.useMemo(
    () => mergeButtonRefs([ref, dragRef]),
    [ref, dragRef]
  );

  return (
    <button
      ref={combinedRef}
      data-sidebar="rail"
      aria-label="Toggle Sidebar"
      tabIndex={-1}
      onMouseDown={handleMouseDown}
      title="Toggle Sidebar"
      className={cn(
        "absolute inset-y-0 z-20 hidden w-4 -translate-x-1/2 transition-all ease-linear group-data-[side=left]:-right-4 group-data-[side=right]:left-0 sm:flex",
        "in-data-[side=left]:cursor-w-resize in-data-[side=right]:cursor-e-resize",
        "[[data-side=left][data-state=collapsed]_&]:cursor-e-resize [[data-side=right][data-state=collapsed]_&]:cursor-w-resize",
        "group-data-[collapsible=offcanvas]:translate-x-0",
        "[[data-side=left][data-collapsible=offcanvas]_&]:-right-2",
        "[[data-side=right][data-collapsible=offcanvas]_&]:-left-2",
        className
      )}
      {...props}
    />
  );
});
SidebarRail.displayName = "SidebarRail";

const SidebarInset = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"main">
>(({ className, ...props }, ref) => {
  return (
    <main
      ref={ref}
      className={cn(
        "relative flex h-full min-h-0 min-w-0 flex-1 flex-col bg-background overflow-hidden",
        "peer-data-[variant=inset]:h-[calc(100svh-var(--header-height,0px)-(--spacing(4)))] md:peer-data-[variant=inset]:m-2 md:peer-data-[variant=inset]:peer-data-[state=collapsed]:ml-0 md:peer-data-[variant=inset]:ml-0 md:peer-data-[variant=inset]:rounded-xl md:peer-data-[variant=inset]:shadow-sm",
        className
      )}
      {...props}
    />
  );
});
SidebarInset.displayName = "SidebarInset";

const SidebarInput = React.forwardRef<
  React.ElementRef<typeof Input>,
  React.ComponentProps<typeof Input>
>(({ className, ...props }, ref) => {
  return (
    <Input
      ref={ref}
      data-sidebar="input"
      className={cn(
        "h-8 w-full bg-background shadow-none focus-visible:ring-2 focus-visible:ring-sidebar-ring",
        className
      )}
      {...props}
    />
  );
});
SidebarInput.displayName = "SidebarInput";

const SidebarHeader = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div">
>(({ className, ...props }, ref) => {
  return (
    <div
      ref={ref}
      data-sidebar="header"
      className={cn("flex flex-col gap-2 p-2", className)}
      {...props}
    />
  );
});
SidebarHeader.displayName = "SidebarHeader";

const SidebarFooter = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div">
>(({ className, ...props }, ref) => {
  return (
    <div
      ref={ref}
      data-sidebar="footer"
      className={cn("flex flex-col gap-2 p-2", className)}
      {...props}
    />
  );
});
SidebarFooter.displayName = "SidebarFooter";

const SidebarSeparator = React.forwardRef<
  React.ElementRef<typeof Separator>,
  React.ComponentProps<typeof Separator>
>(({ className, ...props }, ref) => {
  return (
    <Separator
      ref={ref}
      data-sidebar="separator"
      className={cn("my-2 w-auto bg-sidebar-border", className)}
      {...props}
    />
  );
});
SidebarSeparator.displayName = "SidebarSeparator";

const SidebarContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div">
>(({ className, ...props }, ref) => {
  return (
    <div
      ref={ref}
      data-sidebar="content"
      className={cn(
        "flex min-h-0 flex-1 flex-col gap-2 overflow-auto group-data-[collapsible=icon]:overflow-hidden",
        className
      )}
      {...props}
    />
  );
});
SidebarContent.displayName = "SidebarContent";

const SidebarGroup = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div">
>(({ className, ...props }, ref) => {
  return (
    <div
      ref={ref}
      data-sidebar="group"
      className={cn("relative flex w-full min-w-0 flex-col p-2", className)}
      {...props}
    />
  );
});
SidebarGroup.displayName = "SidebarGroup";

const SidebarGroupLabel = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div"> & { asChild?: boolean }
>(({ className, asChild = false, ...props }, ref) => {
  const Comp = asChild ? Slot : "div";

  return (
    <Comp
      ref={ref}
      data-sidebar="group-label"
      className={cn(
        "duration-200 flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70 outline-hidden ring-sidebar-ring transition-[margin,opa] ease-linear focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0",
        "group-data-[collapsible=icon]:-mt-8 group-data-[collapsible=icon]:opacity-0",
        className
      )}
      {...props}
    />
  );
});
SidebarGroupLabel.displayName = "SidebarGroupLabel";

const SidebarGroupAction = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<"button"> & { asChild?: boolean }
>(({ className, asChild = false, ...props }, ref) => {
  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      ref={ref}
      data-sidebar="group-action"
      className={cn(
        "absolute right-3 top-3.5 flex aspect-square w-5 items-center justify-center rounded-md p-0 text-sidebar-foreground outline-hidden ring-sidebar-ring transition-transform hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0",
        // Increases the hit area of the button on mobile.
        "after:absolute after:-inset-2 md:after:hidden",
        "group-data-[collapsible=icon]:hidden",
        className
      )}
      {...props}
    />
  );
});
SidebarGroupAction.displayName = "SidebarGroupAction";

const SidebarGroupContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div">
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    data-sidebar="group-content"
    className={cn("w-full text-sm", className)}
    {...props}
  />
));
SidebarGroupContent.displayName = "SidebarGroupContent";

const SidebarMenu = React.forwardRef<
  HTMLUListElement,
  React.ComponentProps<"ul">
>(({ className, ...props }, ref) => (
  <ul
    ref={ref}
    data-sidebar="menu"
    className={cn("flex w-full min-w-0 flex-col gap-1", className)}
    {...props}
  />
));
SidebarMenu.displayName = "SidebarMenu";

const SidebarMenuItem = React.forwardRef<
  HTMLLIElement,
  React.ComponentProps<"li">
>(({ className, ...props }, ref) => (
  <li
    ref={ref}
    data-sidebar="menu-item"
    className={cn("group/menu-item relative", className)}
    {...props}
  />
));
SidebarMenuItem.displayName = "SidebarMenuItem";

const sidebarMenuButtonVariants = cva(
  "peer/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm outline-hidden ring-sidebar-ring transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 group-has-data-[sidebar=menu-action]/menu-item:pr-8 aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[active=true]:bg-sidebar-accent data-[active=true]:font-medium data-[active=true]:text-sidebar-accent-foreground data-[state=open]:hover:bg-sidebar-accent data-[state=open]:hover:text-sidebar-accent-foreground group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:p-2 [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 cursor-pointer",
  {
    variants: {
      variant: {
        default: "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        outline:
          "bg-background shadow-[0_0_0_1px_hsl(var(--sidebar-border))] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground hover:shadow-[0_0_0_1px_hsl(var(--sidebar-accent))]",
      },
      size: {
        default: "h-8 text-sm",
        sm: "h-7 text-xs",
        lg: "h-12 text-sm group-data-[collapsible=icon]:p-0!",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

const SidebarMenuButton = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<"button"> & {
    asChild?: boolean;
    isActive?: boolean;
    tooltip?: string | React.ComponentProps<typeof TooltipContent>;
  } & VariantProps<typeof sidebarMenuButtonVariants>
>(
  (
    {
      asChild = false,
      isActive = false,
      variant = "default",
      size = "default",
      tooltip,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const { isMobile, state } = useSidebar();

    // Extract tooltip label for mobile display
    const tooltipLabel = tooltip
      ? typeof tooltip === "string"
        ? tooltip
        : tooltip.children
      : null;

    // On mobile with asChild, we need to clone the child element and inject the label
    // because Slot expects a single child
    if (isMobile && tooltipLabel && asChild) {
      const child = React.Children.only(children) as React.ReactElement<{
        className?: string;
        children?: React.ReactNode;
      }>;
      return React.cloneElement(child, {
        ref,
        "data-sidebar": "menu-button",
        "data-size": size,
        "data-active": isActive,
        className: cn(
          sidebarMenuButtonVariants({ variant, size }),
          "justify-start gap-2 px-3 h-10",
          className,
          child.props.className
        ),
        ...props,
        children: (
          <>
            <span className="shrink-0 size-5 flex items-center justify-center [&>svg]:size-4">
              {child.props.children}
            </span>
            <span className="truncate text-sm">{tooltipLabel}</span>
          </>
        ),
      } as React.HTMLAttributes<HTMLElement>);
    }

    // On mobile without asChild, render button with icon + label
    if (isMobile && tooltipLabel) {
      return (
        <button
          ref={ref}
          data-sidebar="menu-button"
          data-size={size}
          data-active={isActive}
          className={cn(
            sidebarMenuButtonVariants({ variant, size }),
            "justify-start gap-2 px-3 h-10",
            className
          )}
          {...props}
        >
          <span className="shrink-0 size-5 flex items-center justify-center [&>svg]:size-4">
            {children}
          </span>
          <span className="truncate text-sm">{tooltipLabel}</span>
        </button>
      );
    }

    const Comp = asChild ? Slot : "button";

    const button = (
      <Comp
        ref={ref}
        data-sidebar="menu-button"
        data-size={size}
        data-active={isActive}
        className={cn(sidebarMenuButtonVariants({ variant, size }), className)}
        {...props}
      >
        {children}
      </Comp>
    );

    if (!tooltip) {
      return button;
    }

    const tooltipProps =
      typeof tooltip === "string" ? { children: tooltip } : tooltip;

    return (
      <Tooltip>
        <TooltipTrigger asChild>{button}</TooltipTrigger>
        <TooltipContent
          side="right"
          align="center"
          hidden={state !== "collapsed" || isMobile}
          {...tooltipProps}
        />
      </Tooltip>
    );
  }
);
SidebarMenuButton.displayName = "SidebarMenuButton";

const SidebarMenuAction = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<"button"> & {
    asChild?: boolean;
    showOnHover?: boolean;
  }
>(({ className, asChild = false, showOnHover = false, ...props }, ref) => {
  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      ref={ref}
      data-sidebar="menu-action"
      className={cn(
        "absolute right-1 top-1.5 flex aspect-square w-5 items-center justify-center rounded-md p-0 text-sidebar-foreground outline-hidden ring-sidebar-ring transition-transform hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 peer-hover/menu-button:text-sidebar-accent-foreground [&>svg]:size-4 [&>svg]:shrink-0",
        // Increases the hit area of the button on mobile.
        "after:absolute after:-inset-2 md:after:hidden",
        "peer-data-[size=sm]/menu-button:top-1",
        "peer-data-[size=default]/menu-button:top-1.5",
        "peer-data-[size=lg]/menu-button:top-2.5",
        "group-data-[collapsible=icon]:hidden",
        showOnHover &&
          "group-focus-within/menu-item:opacity-100 group-hover/menu-item:opacity-100 data-[state=open]:opacity-100 peer-data-[active=true]/menu-button:text-sidebar-accent-foreground md:opacity-0",
        className
      )}
      {...props}
    />
  );
});
SidebarMenuAction.displayName = "SidebarMenuAction";

const SidebarMenuBadge = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div">
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    data-sidebar="menu-badge"
    className={cn(
      "absolute right-1 flex h-5 min-w-5 items-center justify-center rounded-md px-1 text-xs font-medium tabular-nums text-sidebar-foreground select-none pointer-events-none",
      "peer-hover/menu-button:text-sidebar-accent-foreground peer-data-[active=true]/menu-button:text-sidebar-accent-foreground",
      "peer-data-[size=sm]/menu-button:top-1",
      "peer-data-[size=default]/menu-button:top-1.5",
      "peer-data-[size=lg]/menu-button:top-2.5",
      "group-data-[collapsible=icon]:hidden",
      className
    )}
    {...props}
  />
));
SidebarMenuBadge.displayName = "SidebarMenuBadge";

const SidebarMenuSkeleton = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div"> & {
    showIcon?: boolean;
  }
>(({ className, showIcon = false, ...props }, ref) => {
  const width = "70%";

  return (
    <div
      ref={ref}
      data-sidebar="menu-skeleton"
      className={cn("rounded-md h-8 flex gap-2 px-2 items-center", className)}
      {...props}
    >
      {showIcon && (
        <Skeleton
          className="size-4 rounded-md"
          data-sidebar="menu-skeleton-icon"
        />
      )}
      <Skeleton
        className="h-4 flex-1 max-w-(--skeleton-width)"
        data-sidebar="menu-skeleton-text"
        style={
          {
            "--skeleton-width": width,
          } as React.CSSProperties
        }
      />
    </div>
  );
});
SidebarMenuSkeleton.displayName = "SidebarMenuSkeleton";

const SidebarMenuSub = React.forwardRef<
  HTMLUListElement,
  React.ComponentProps<"ul">
>(({ className, ...props }, ref) => (
  <ul
    ref={ref}
    data-sidebar="menu-sub"
    className={cn(
      "mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l border-sidebar-border px-2.5 py-0.5",
      "group-data-[collapsible=icon]:hidden",
      className
    )}
    {...props}
  />
));
SidebarMenuSub.displayName = "SidebarMenuSub";

const SidebarMenuSubItem = React.forwardRef<
  HTMLLIElement,
  React.ComponentProps<"li">
>(({ ...props }, ref) => <li ref={ref} {...props} />);
SidebarMenuSubItem.displayName = "SidebarMenuSubItem";

const SidebarMenuSubButton = React.forwardRef<
  HTMLAnchorElement,
  React.ComponentProps<"a"> & {
    asChild?: boolean;
    size?: "sm" | "md";
    isActive?: boolean;
  }
>(({ asChild = false, size = "md", isActive, className, ...props }, ref) => {
  const Comp = asChild ? Slot : "a";

  return (
    <Comp
      ref={ref}
      data-sidebar="menu-sub-button"
      data-size={size}
      data-active={isActive}
      className={cn(
        "flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-sidebar-foreground outline-hidden ring-sidebar-ring hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 [&>svg]:text-sidebar-accent-foreground",
        "data-[active=true]:bg-sidebar-accent data-[active=true]:text-sidebar-accent-foreground",
        size === "sm" && "text-xs",
        size === "md" && "text-sm",
        "group-data-[collapsible=icon]:hidden",
        className
      )}
      {...props}
    />
  );
});
SidebarMenuSubButton.displayName = "SidebarMenuSubButton";

export {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInput,
  SidebarInset,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSkeleton,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarProvider,
  SidebarRail,
  SidebarSeparator,
  SidebarTrigger,
  useSidebar,
};
