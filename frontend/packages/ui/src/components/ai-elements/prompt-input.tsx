"use client";

import {
  AudioLinesIcon,
  CornerDownLeftIcon,
  FolderUpIcon,
  Loader2Icon,
  MicIcon,
  PaperclipIcon,
  PlusIcon,
  SquareIcon,
  XIcon,
} from "lucide-react";
import { nanoid } from "nanoid";
import {
  
  
  Children,
  
  
  
  
  Fragment,
  
  
  
  
  
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState
} from "react";
import { Button } from "@workspace/ui/components/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@workspace/ui/components/dropdown-menu";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupTextarea,
} from "@workspace/ui/components/input-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@workspace/ui/components/select";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@workspace/ui/components/tooltip";
import { cn } from "@workspace/ui/lib/utils";
import type { ChatStatus, FileUIPart } from "ai";
import type {ChangeEvent, ChangeEventHandler, ClipboardEventHandler, ComponentProps, FormEvent, FormEventHandler, HTMLAttributes, KeyboardEventHandler, PropsWithChildren, ReactNode, RefObject} from "react";
// ============================================================================
// Provider Context & Types
// ============================================================================

// Extended FileUIPart type to include relativePath for folder uploads
export type ExtendedFileUIPart = FileUIPart & {
  id: string;
  /** Relative path for folder uploads (webkitRelativePath from the original File) */
  relativePath?: string;
};

export type AttachmentsContext = {
  files: Array<ExtendedFileUIPart>;
  add: (files: Array<File> | FileList) => void;
  /** Add files with explicit relative paths (for folder drag & drop) */
  addWithPaths: (
    filesWithPaths: Array<{ file: File; relativePath: string }>
  ) => void;
  remove: (id: string) => void;
  clear: () => void;
  openFileDialog: () => void;
  openFolderDialog: () => void;
  fileInputRef: RefObject<HTMLInputElement | null>;
  folderInputRef: RefObject<HTMLInputElement | null>;
};

export type TextInputContext = {
  value: string;
  setInput: (v: string) => void;
  clear: () => void;
};

export type PromptInputController = {
  textInput: TextInputContext;
  attachments: AttachmentsContext;
  /** INTERNAL: Allows PromptInput to register its file textInput + "open" callback */
  __registerFileInput: (
    ref: RefObject<HTMLInputElement | null>,
    open: () => void
  ) => void;
  /** INTERNAL: Allows PromptInput to register its folder input + "open" callback */
  __registerFolderInput: (
    ref: RefObject<HTMLInputElement | null>,
    open: () => void
  ) => void;
};

const PromptInputContext = createContext<PromptInputController | null>(null);
const ProviderAttachmentsContext = createContext<AttachmentsContext | null>(
  null
);

export const usePromptInputController = () => {
  const ctx = useContext(PromptInputContext);
  if (!ctx) {
    throw new Error(
      "Wrap your component inside <PromptInputProvider> to use usePromptInputController()."
    );
  }
  return ctx;
};

// Optional variants (do NOT throw). Useful for dual-mode components.
const useOptionalPromptInputController = () => {
  return useContext(PromptInputContext);
};

export const useProviderAttachments = () => {
  const ctx = useContext(ProviderAttachmentsContext);
  if (!ctx) {
    throw new Error(
      "Wrap your component inside <PromptInputProvider> to use useProviderAttachments()."
    );
  }
  return ctx;
};

const useOptionalProviderAttachments = () => {
  return useContext(ProviderAttachmentsContext);
};

export type PromptInputProviderProps = PropsWithChildren<{
  initialInput?: string;
}>;

/**
 * Optional global provider that lifts PromptInput state outside of PromptInput.
 * If you don't use it, PromptInput stays fully self-managed.
 */
export function PromptInputProvider({
  initialInput: initialTextInput = "",
  children,
}: PromptInputProviderProps) {
  // ----- textInput state
  const [textInput, setTextInput] = useState(initialTextInput);
  const clearInput = useCallback(() => setTextInput(""), []);

  // ----- attachments state (global when wrapped)
  const [attachements, setAttachements] = useState<
    Array<ExtendedFileUIPart>
  >([]);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const folderInputRef = useRef<HTMLInputElement | null>(null);
  const openRef = useRef<() => void>(() => {});
  const openFolderRef = useRef<() => void>(() => {});

  const add = useCallback((files: Array<File> | FileList) => {
    const incoming = Array.from(files);
    if (incoming.length === 0) return;

    setAttachements((prev) =>
      prev.concat(
        incoming.map((file) => ({
          id: nanoid(),
          type: "file" as const,
          url: URL.createObjectURL(file),
          mediaType: file.type,
          filename: file.name,
          // Preserve webkitRelativePath for folder uploads
          relativePath: (file as File & { webkitRelativePath?: string })
            .webkitRelativePath,
        }))
      )
    );
  }, []);

  // Add files with explicit relative paths (for folder drag & drop)
  const addWithPaths = useCallback(
    (filesWithPaths: Array<{ file: File; relativePath: string }>) => {
      if (filesWithPaths.length === 0) return;

      setAttachements((prev) =>
        prev.concat(
          filesWithPaths.map(({ file, relativePath }) => ({
            id: nanoid(),
            type: "file" as const,
            url: URL.createObjectURL(file),
            mediaType: file.type,
            filename: file.name,
            relativePath,
          }))
        )
      );
    },
    []
  );

  const remove = useCallback((id: string) => {
    setAttachements((prev) => {
      const found = prev.find((f) => f.id === id);
      if (found?.url) URL.revokeObjectURL(found.url);
      return prev.filter((f) => f.id !== id);
    });
  }, []);

  const clear = useCallback(() => {
    setAttachements((prev) => {
      for (const f of prev) if (f.url) URL.revokeObjectURL(f.url);
      return [];
    });
  }, []);

  const openFileDialog = useCallback(() => {
    openRef.current();
  }, []);

  const openFolderDialog = useCallback(() => {
    openFolderRef.current();
  }, []);

  const attachments = useMemo<AttachmentsContext>(
    () => ({
      files: attachements,
      add,
      addWithPaths,
      remove,
      clear,
      openFileDialog,
      openFolderDialog,
      fileInputRef,
      folderInputRef,
    }),
    [attachements, add, addWithPaths, remove, clear, openFileDialog, openFolderDialog]
  );

  const __registerFileInput = useCallback(
    (ref: RefObject<HTMLInputElement | null>, open: () => void) => {
      fileInputRef.current = ref.current;
      openRef.current = open;
    },
    []
  );

  const __registerFolderInput = useCallback(
    (ref: RefObject<HTMLInputElement | null>, open: () => void) => {
      folderInputRef.current = ref.current;
      openFolderRef.current = open;
    },
    []
  );

  const controller = useMemo<PromptInputController>(
    () => ({
      textInput: {
        value: textInput,
        setInput: setTextInput,
        clear: clearInput,
      },
      attachments,
      __registerFileInput,
      __registerFolderInput,
    }),
    [textInput, clearInput, attachments, __registerFileInput, __registerFolderInput]
  );

  return (
    <PromptInputContext.Provider value={controller}>
      <ProviderAttachmentsContext.Provider value={attachments}>
        {children}
      </ProviderAttachmentsContext.Provider>
    </PromptInputContext.Provider>
  );
}

// ============================================================================
// Component Context & Hooks
// ============================================================================

const LocalAttachmentsContext = createContext<AttachmentsContext | null>(null);

export const usePromptInputAttachments = () => {
  // Dual-mode: prefer provider if present, otherwise use local
  const provider = useOptionalProviderAttachments();
  const local = useContext(LocalAttachmentsContext);
  const context = provider ?? local;
  if (!context) {
    throw new Error(
      "usePromptInputAttachments must be used within a PromptInput or PromptInputProvider"
    );
  }
  return context;
};

export type PromptInputAttachmentProps = HTMLAttributes<HTMLDivElement> & {
  data: FileUIPart & { id: string };
  className?: string;
};

export function PromptInputAttachment({
  data,
  className,
  ...props
}: PromptInputAttachmentProps) {
  const attachments = usePromptInputAttachments();

  const mediaType =
    data.mediaType.startsWith("image/") && data.url ? "image" : "file";

  return (
    <div
      className={cn(
        "group relative h-14 w-14 rounded-md border",
        className,
        mediaType === "image" ? "h-14 w-14" : "h-8 w-auto max-w-full"
      )}
      key={data.id}
      {...props}
    >
      {mediaType === "image" ? (
        <img
          alt={data.filename || "attachment"}
          className="size-full rounded-md object-cover"
          height={56}
          src={data.url}
          width={56}
        />
      ) : (
        <div className="flex size-full max-w-full cursor-pointer items-center justify-start gap-2 overflow-hidden px-2 text-muted-foreground">
          <PaperclipIcon className="size-4 shrink-0" />
          <Tooltip delayDuration={400}>
            <TooltipTrigger className="min-w-0 flex-1">
              <h4 className="w-full truncate text-left font-medium text-sm">
                {data.filename || "Unknown file"}
              </h4>
            </TooltipTrigger>
            <TooltipContent>
              <div className="text-muted-foreground text-xs">
                <h4 className="max-w-[240px] overflow-hidden whitespace-normal break-words text-left font-semibold text-sm">
                  {data.filename || "Unknown file"}
                </h4>
                {data.mediaType && <div>{data.mediaType}</div>}
              </div>
            </TooltipContent>
          </Tooltip>
        </div>
      )}
      <Button
        aria-label="Remove attachment"
        className="-right-1.5 -top-1.5 absolute h-6 w-6 rounded-full opacity-0 group-hover:opacity-100"
        onClick={() => attachments.remove(data.id)}
        size="icon"
        type="button"
        variant="outline"
      >
        <XIcon className="h-3 w-3" />
      </Button>
    </div>
  );
}

export type PromptInputAttachmentsProps = Omit<
  HTMLAttributes<HTMLDivElement>,
  "children"
> & {
  children: (attachment: FileUIPart & { id: string }) => ReactNode;
};

export function PromptInputAttachments({
  className,
  children,
  ...props
}: PromptInputAttachmentsProps) {
  const attachments = usePromptInputAttachments();
  const [height, setHeight] = useState(0);
  const contentRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const el = contentRef.current;
    if (!el) {
      return;
    }
    const ro = new ResizeObserver(() => {
      setHeight(el.getBoundingClientRect().height);
    });
    ro.observe(el);
    setHeight(el.getBoundingClientRect().height);
    return () => ro.disconnect();
  }, []);

  // biome-ignore lint/correctness/useExhaustiveDependencies: Force height measurement when attachments change
  useLayoutEffect(() => {
    const el = contentRef.current;
    if (!el) {
      return;
    }
    setHeight(el.getBoundingClientRect().height);
  }, [attachments.files.length]);

  if (attachments.files.length === 0) {
    return null;
  }

  return (
    <InputGroupAddon
      align="block-start"
      aria-live="polite"
      className={cn(
        "overflow-hidden transition-[height] duration-200 ease-out",
        className
      )}
      style={{ height: attachments.files.length ? height : 0 }}
      {...props}
    >
      <div className="space-y-2 py-1" ref={contentRef}>
        <div className="flex flex-wrap gap-2">
          {attachments.files
            .filter((f) => !(f.mediaType.startsWith("image/") && f.url))
            .map((file) => (
              <Fragment key={file.id}>{children(file)}</Fragment>
            ))}
        </div>
        <div className="flex flex-wrap gap-2">
          {attachments.files
            .filter((f) => f.mediaType.startsWith("image/") && f.url)
            .map((file) => (
              <Fragment key={file.id}>{children(file)}</Fragment>
            ))}
        </div>
      </div>
    </InputGroupAddon>
  );
}

export type PromptInputActionAddAttachmentsProps = ComponentProps<
  typeof DropdownMenuItem
> & {
  label?: string;
};

export const PromptInputActionAddAttachments = ({
  label,
  ...props
}: PromptInputActionAddAttachmentsProps) => {
  const attachments = usePromptInputAttachments();

  return (
    <DropdownMenuItem
      {...props}
      onSelect={() => {
        // Don't prevent default - let the dropdown close naturally
        // Use setTimeout to ensure the file dialog opens after dropdown closes
        setTimeout(() => {
          attachments.openFileDialog();
        }, 0);
      }}
    >
      <PaperclipIcon className="mr-2 size-4" /> {label}
    </DropdownMenuItem>
  );
};

export type PromptInputActionAddFolderAttachmentsProps = ComponentProps<
  typeof DropdownMenuItem
> & {
  label?: string;
};

export const PromptInputActionAddFolderAttachments = ({
  label,
  ...props
}: PromptInputActionAddFolderAttachmentsProps) => {
  const attachments = usePromptInputAttachments();

  return (
    <DropdownMenuItem
      {...props}
      onSelect={() => {
        // Don't prevent default - let the dropdown close naturally
        // Use setTimeout to ensure the folder dialog opens after dropdown closes
        setTimeout(() => {
          attachments.openFolderDialog();
        }, 0);
      }}
    >
      <FolderUpIcon className="mr-2 size-4" /> {label}
    </DropdownMenuItem>
  );
};

export type PromptInputMessage = {
  text?: string;
  files?: Array<FileUIPart>;
};

export type PromptInputProps = Omit<
  HTMLAttributes<HTMLFormElement>,
  "onSubmit"
> & {
  accept?: string; // e.g., "image/*" or leave undefined for any
  multiple?: boolean;
  // When true, accepts drops anywhere on document. Default false (opt-in).
  globalDrop?: boolean;
  // Render a hidden input with given name and keep it in sync for native form posts. Default false.
  syncHiddenInput?: boolean;
  // Minimal constraints
  maxFiles?: number;
  maxFileSize?: number; // bytes
  onError?: (err: {
    code: "max_files" | "max_file_size" | "accept";
    message: string;
  }) => void;
  onSubmit: (
    message: PromptInputMessage,
    event: FormEvent<HTMLFormElement>
  ) => void | Promise<void>;
};

export const PromptInput = ({
  className,
  accept,
  multiple,
  globalDrop,
  syncHiddenInput,
  maxFiles,
  maxFileSize,
  onError,
  onSubmit,
  children,
  ...props
}: PromptInputProps) => {
  // Try to use a provider controller if present
  const controller = useOptionalPromptInputController();
  const usingProvider = !!controller;

  // Refs
  const inputRef = useRef<HTMLInputElement | null>(null);
  const folderInputRef = useRef<HTMLInputElement | null>(null);
  const anchorRef = useRef<HTMLSpanElement>(null);
  const formRef = useRef<HTMLFormElement | null>(null);

  // Find nearest form to scope drag & drop
  useEffect(() => {
    const root = anchorRef.current?.closest("form");
    if (root instanceof HTMLFormElement) {
      formRef.current = root;
    }
  }, []);

  // ----- Local attachments (only used when no provider)
  const [items, setItems] = useState<Array<ExtendedFileUIPart>>([]);
  const files = usingProvider ? controller.attachments.files : items;

  const openFileDialogLocal = useCallback(() => {
    inputRef.current?.click();
  }, []);

  const openFolderDialogLocal = useCallback(() => {
    folderInputRef.current?.click();
  }, []);

  const matchesAccept = useCallback(
    (f: File) => {
      if (!accept || accept.trim() === "") {
        return true;
      }
      if (accept.includes("image/*")) {
        return f.type.startsWith("image/");
      }
      // NOTE: keep simple; expand as needed
      return true;
    },
    [accept]
  );

  const addLocal = useCallback(
    (fileList: Array<File> | FileList) => {
      const incoming = Array.from(fileList);
      const accepted = incoming.filter((f) => matchesAccept(f));
      if (incoming.length && accepted.length === 0) {
        onError?.({
          code: "accept",
          message: "No files match the accepted types.",
        });
        return;
      }
      const withinSize = (f: File) =>
        maxFileSize ? f.size <= maxFileSize : true;
      const sized = accepted.filter(withinSize);
      if (accepted.length > 0 && sized.length === 0) {
        onError?.({
          code: "max_file_size",
          message: "All files exceed the maximum size.",
        });
        return;
      }

      setItems((prev) => {
        const capacity =
          typeof maxFiles === "number"
            ? Math.max(0, maxFiles - prev.length)
            : undefined;
        const capped =
          typeof capacity === "number" ? sized.slice(0, capacity) : sized;
        if (typeof capacity === "number" && sized.length > capacity) {
          onError?.({
            code: "max_files",
            message: "Too many files. Some were not added.",
          });
        }
        const next: Array<ExtendedFileUIPart> = [];
        for (const file of capped) {
          next.push({
            id: nanoid(),
            type: "file",
            url: URL.createObjectURL(file),
            mediaType: file.type,
            filename: file.name,
            // Preserve webkitRelativePath for folder uploads
            relativePath: (file as File & { webkitRelativePath?: string })
              .webkitRelativePath,
          });
        }
        return prev.concat(next);
      });
    },
    [matchesAccept, maxFiles, maxFileSize, onError]
  );

  const add = useCallback(
    (incomingFiles: Array<File> | FileList) => {
      if (usingProvider) {
        controller.attachments.add(incomingFiles);
        return;
      }
      addLocal(incomingFiles);
    },
    [usingProvider, controller, addLocal]
  );

  // Add files with explicit relative paths (for folder drag & drop)
  const addWithPathsLocal = useCallback(
    (filesWithPaths: Array<{ file: File; relativePath: string }>) => {
      if (filesWithPaths.length === 0) return;

      const accepted = filesWithPaths.filter(({ file }) => matchesAccept(file));
      if (filesWithPaths.length && accepted.length === 0) {
        onError?.({
          code: "accept",
          message: "No files match the accepted types.",
        });
        return;
      }
      const withinSize = ({ file }: { file: File }) =>
        maxFileSize ? file.size <= maxFileSize : true;
      const sized = accepted.filter(withinSize);
      if (accepted.length > 0 && sized.length === 0) {
        onError?.({
          code: "max_file_size",
          message: "All files exceed the maximum size.",
        });
        return;
      }

      setItems((prev) => {
        const capacity =
          typeof maxFiles === "number"
            ? Math.max(0, maxFiles - prev.length)
            : undefined;
        const capped =
          typeof capacity === "number" ? sized.slice(0, capacity) : sized;
        if (typeof capacity === "number" && sized.length > capacity) {
          onError?.({
            code: "max_files",
            message: "Too many files. Some were not added.",
          });
        }
        const next: Array<ExtendedFileUIPart> = [];
        for (const { file, relativePath } of capped) {
          next.push({
            id: nanoid(),
            type: "file",
            url: URL.createObjectURL(file),
            mediaType: file.type,
            filename: file.name,
            relativePath,
          });
        }
        return prev.concat(next);
      });
    },
    [matchesAccept, maxFiles, maxFileSize, onError]
  );

  const addWithPaths = useCallback(
    (filesWithPaths: Array<{ file: File; relativePath: string }>) => {
      if (usingProvider) {
        controller.attachments.addWithPaths(filesWithPaths);
        return;
      }
      addWithPathsLocal(filesWithPaths);
    },
    [usingProvider, controller, addWithPathsLocal]
  );

  const remove = useCallback(
    (id: string) => {
      if (usingProvider) {
        controller.attachments.remove(id);
        return;
      }
      setItems((prev) => {
        const found = prev.find((file) => file.id === id);
        if (found?.url) {
          URL.revokeObjectURL(found.url);
        }
        return prev.filter((file) => file.id !== id);
      });
    },
    [usingProvider, controller]
  );

  const clear = useCallback(() => {
    if (usingProvider) {
      controller.attachments.clear();
      return;
    }
    setItems((prev) => {
      for (const file of prev) {
        if (file.url) {
          URL.revokeObjectURL(file.url);
        }
      }
      return [];
    });
  }, [usingProvider, controller]);

  const openFileDialog = useCallback(() => {
    if (usingProvider) {
      controller.attachments.openFileDialog();
      return;
    }
    openFileDialogLocal();
  }, [usingProvider, controller, openFileDialogLocal]);

  const openFolderDialog = useCallback(() => {
    if (usingProvider) {
      controller.attachments.openFolderDialog();
      return;
    }
    openFolderDialogLocal();
  }, [usingProvider, controller, openFolderDialogLocal]);

  // Let provider know about our hidden file input so external menus can call openFileDialog()
  useEffect(() => {
    if (!usingProvider) return;
    controller.__registerFileInput(inputRef, () => inputRef.current?.click());
    controller.__registerFolderInput(folderInputRef, () => folderInputRef.current?.click());
  }, [usingProvider, controller]);

  // Note: File input cannot be programmatically set for security reasons
  // The syncHiddenInput prop is no longer functional
  useEffect(() => {
    if (syncHiddenInput && inputRef.current && files.length === 0) {
      inputRef.current.value = "";
    }
  }, [files, syncHiddenInput]);

  // Attach drop handlers on nearest form and document (opt-in)
  useEffect(() => {
    const form = formRef.current;
    if (!form) return;

    const onDragOver = (e: DragEvent) => {
      const transfer = e.dataTransfer;
      if (transfer?.types.includes("Files")) {
        e.preventDefault();
      }
    };
    const onDrop = (e: DragEvent) => {
      const transfer = e.dataTransfer;
      if (!transfer) {
        return;
      }
      if (transfer.types.includes("Files")) {
        e.preventDefault();
      }
      if (transfer.files.length > 0) {
        add(transfer.files);
      }
    };
    form.addEventListener("dragover", onDragOver);
    form.addEventListener("drop", onDrop);
    return () => {
      form.removeEventListener("dragover", onDragOver);
      form.removeEventListener("drop", onDrop);
    };
  }, [add]);

  useEffect(() => {
    if (!globalDrop) return;

    // Helper to recursively read directory entries using File System Access API
    const readDirectoryRecursively = async (
      entry: FileSystemDirectoryEntry,
      basePath: string
    ): Promise<Array<{ file: File; relativePath: string }>> => {
      const results: Array<{ file: File; relativePath: string }> = [];
      const reader = entry.createReader();

      // readEntries returns batches, so we need to keep reading until empty
      const readBatch = (): Promise<Array<FileSystemEntry>> =>
        new Promise((resolve, reject) => {
          reader.readEntries(resolve, reject);
        });

      let batch: Array<FileSystemEntry>;
      do {
        batch = await readBatch();
        for (const childEntry of batch) {
          const childPath = basePath
            ? `${basePath}/${childEntry.name}`
            : childEntry.name;
          if (childEntry.isFile) {
            const fileEntry = childEntry as FileSystemFileEntry;
            const file = await new Promise<File>((resolve, reject) => {
              fileEntry.file(resolve, reject);
            });
            results.push({ file, relativePath: childPath });
          } else if (childEntry.isDirectory) {
            const dirEntry = childEntry as FileSystemDirectoryEntry;
            const childFiles = await readDirectoryRecursively(dirEntry, childPath);
            results.push(...childFiles);
          }
        }
      } while (batch.length > 0);

      return results;
    };

    // Process DataTransferItems to handle both files and folders
    const processDataTransferItems = async (
      transferItems: DataTransferItemList
    ): Promise<{
      droppedFiles: Array<File>;
      filesWithPaths: Array<{ file: File; relativePath: string }>;
      hasDirectories: boolean;
    }> => {
      const droppedFiles: Array<File> = [];
      const filesWithPaths: Array<{ file: File; relativePath: string }> = [];
      let hasDirectories = false;

      const itemsArray = Array.from(transferItems);

      for (const item of itemsArray) {
        if (item.kind !== "file") continue;

        // Try to get entry for folder detection (webkitGetAsEntry)
        const entry = item.webkitGetAsEntry();

        if (entry?.isDirectory) {
          hasDirectories = true;
          const dirEntry = entry as FileSystemDirectoryEntry;
          const dirFiles = await readDirectoryRecursively(
            dirEntry,
            dirEntry.name
          );
          filesWithPaths.push(...dirFiles);
        } else if (entry?.isFile) {
          const file = item.getAsFile();
          if (file) {
            droppedFiles.push(file);
          }
        } else {
          // Fallback: webkitGetAsEntry not available
          const file = item.getAsFile();
          if (file) {
            droppedFiles.push(file);
          }
        }
      }

      return { droppedFiles, filesWithPaths, hasDirectories };
    };

    const onDragOver = (e: DragEvent) => {
      const transfer = e.dataTransfer;
      if (transfer?.types.includes("Files")) {
        e.preventDefault();
      }
    };

    const onDrop = async (e: DragEvent) => {
      const transfer = e.dataTransfer;
      if (!transfer) {
        return;
      }
      if (transfer.types.includes("Files")) {
        e.preventDefault();
      }

      if (transfer.items.length === 0) {
        // Fallback to files if items not available
        if (transfer.files.length > 0) {
          add(transfer.files);
        }
        return;
      }

      try {
        const { droppedFiles, filesWithPaths, hasDirectories } =
          await processDataTransferItems(transfer.items);

        if (hasDirectories && filesWithPaths.length > 0) {
          // Folder drop - use addWithPaths to preserve relative paths
          addWithPaths(filesWithPaths);
        } else if (droppedFiles.length > 0) {
          // Regular file drop
          add(droppedFiles);
        }
      } catch (error) {
        console.error("Error processing dropped items:", error);
        // Fallback to simple file handling
        if (transfer.files.length > 0) {
          add(transfer.files);
        }
      }
    };

    document.addEventListener("dragover", onDragOver);
    document.addEventListener("drop", onDrop);
    return () => {
      document.removeEventListener("dragover", onDragOver);
      document.removeEventListener("drop", onDrop);
    };
  }, [add, addWithPaths, globalDrop]);

  useEffect(() => {
    return () => {
      if (!usingProvider) {
        for (const f of files) {
          if (f.url) URL.revokeObjectURL(f.url);
        }
      }
    };
  }, [usingProvider, files]);

  const handleChange: ChangeEventHandler<HTMLInputElement> = (event) => {
    if (event.currentTarget.files) {
      add(event.currentTarget.files);
    }
  };

  const handleFolderChange: ChangeEventHandler<HTMLInputElement> = (event) => {
    if (event.currentTarget.files) {
      add(event.currentTarget.files);
    }
  };

  const convertBlobUrlToDataUrl = async (url: string): Promise<string> => {
    const response = await fetch(url);
    const blob = await response.blob();
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result as string);
      reader.onerror = reject;
      reader.readAsDataURL(blob);
    });
  };

  const ctx = useMemo<AttachmentsContext>(
    () => ({
      files: files.map((item) => ({ ...item })),
      add,
      addWithPaths,
      remove,
      clear,
      openFileDialog,
      openFolderDialog,
      fileInputRef: inputRef,
      folderInputRef: folderInputRef,
    }),
    [files, add, addWithPaths, remove, clear, openFileDialog, openFolderDialog]
  );

  const handleSubmit: FormEventHandler<HTMLFormElement> = (event) => {
    event.preventDefault();

    const form = event.currentTarget;
    const text = usingProvider
      ? controller.textInput.value
      : (() => {
          const formData = new FormData(form);
          return (formData.get("message") as string) || "";
        })();

    // Reset form immediately after capturing text to avoid race condition
    // where user input during async blob conversion would be lost
    if (!usingProvider) {
      form.reset();
    }

    // Convert blob URLs to data URLs asynchronously
    Promise.all(
      files.map(async (item) => {
        if (item.url && item.url.startsWith("blob:")) {
          return {
            ...item,
            url: await convertBlobUrlToDataUrl(item.url),
          };
        }
        return item;
      })
    ).then((convertedFiles: Array<FileUIPart>) => {
      try {
        const result = onSubmit({ text, files: convertedFiles }, event);

        // Handle both sync and async onSubmit
        if (result instanceof Promise) {
          result
            .then(() => {
              clear();
              if (usingProvider) {
                controller.textInput.clear();
              }
            })
            .catch(() => {
              // Don't clear on error - user may want to retry
            });
        } else {
          // Sync function completed without throwing, clear attachments
          clear();
          if (usingProvider) {
            controller.textInput.clear();
          }
        }
      } catch {
        // Don't clear on error - user may want to retry
      }
    });
  };

  // Render with or without local provider
  const inner = (
    <>
      <span aria-hidden="true" className="hidden" ref={anchorRef} />
      <input
        accept={accept}
        aria-label="Upload files"
        className="hidden"
        multiple={multiple}
        onChange={handleChange}
        ref={inputRef}
        title="Upload files"
        type="file"
      />
      <input
        aria-label="Upload folder"
        className="hidden"
        multiple
        onChange={handleFolderChange}
        ref={folderInputRef}
        title="Upload folder"
        type="file"
        {...({ webkitdirectory: "", directory: "" } as React.InputHTMLAttributes<HTMLInputElement>)}
      />
      <form
        className={cn("w-full", className)}
        onSubmit={handleSubmit}
        {...props}
      >
        <InputGroup className="bg-background">{children}</InputGroup>
      </form>
    </>
  );

  return usingProvider ? (
    inner
  ) : (
    <LocalAttachmentsContext.Provider value={ctx}>
      {inner}
    </LocalAttachmentsContext.Provider>
  );
};

export type PromptInputBodyProps = HTMLAttributes<HTMLDivElement>;

export const PromptInputBody = ({
  className,
  ...props
}: PromptInputBodyProps) => (
  <div className={cn("contents", className)} {...props} />
);

export type PromptInputTextareaProps = ComponentProps<
  typeof InputGroupTextarea
>;

export const PromptInputTextarea = ({
  onChange,
  className,
  placeholder = "What would you like to know?",
  ...props
}: PromptInputTextareaProps) => {
  const controller = useOptionalPromptInputController();
  const attachments = usePromptInputAttachments();

  const handleKeyDown: KeyboardEventHandler<HTMLTextAreaElement> = (e) => {
    if (e.key === "Enter") {
      if (e.nativeEvent.isComposing) return;
      if (e.shiftKey) return;
      e.preventDefault();
      e.currentTarget.form?.requestSubmit();
    }
  };

  const handlePaste: ClipboardEventHandler<HTMLTextAreaElement> = (event) => {
    const items = event.clipboardData.items;

    const droppedFiles: Array<File> = [];

    for (const item of items) {
      if (item.kind === "file") {
        const file = item.getAsFile();
        if (file) {
          droppedFiles.push(file);
        }
      }
    }

    if (droppedFiles.length > 0) {
      event.preventDefault();
      attachments.add(droppedFiles);
    }
  };

  const controlledProps = controller
    ? {
        value: controller.textInput.value,
        onChange: (e: ChangeEvent<HTMLTextAreaElement>) => {
          controller.textInput.setInput(e.currentTarget.value);
          onChange?.(e);
        },
      }
    : {
        onChange,
      };

  return (
    <InputGroupTextarea
      className={cn("field-sizing-content max-h-48 min-h-16", className)}
      name="message"
      onKeyDown={handleKeyDown}
      onPaste={handlePaste}
      placeholder={placeholder}
      {...props}
      {...controlledProps}
    />
  );
};

export type PromptInputToolbarProps = Omit<
  ComponentProps<typeof InputGroupAddon>,
  "align"
>;

export const PromptInputToolbar = ({
  className,
  ...props
}: PromptInputToolbarProps) => (
  <InputGroupAddon
    align="block-end"
    className={cn("justify-between gap-1", className)}
    {...props}
  />
);

export type PromptInputToolsProps = HTMLAttributes<HTMLDivElement>;

export const PromptInputTools = ({
  className,
  ...props
}: PromptInputToolsProps) => (
  <div className={cn("flex items-center gap-1", className)} {...props} />
);

export type PromptInputButtonProps = ComponentProps<typeof InputGroupButton>;

export const PromptInputButton = ({
  variant = "ghost",
  className,
  size,
  ...props
}: PromptInputButtonProps) => {
  const newSize =
    size ?? (Children.count(props.children) > 1 ? "sm" : "icon-sm");

  return (
    <InputGroupButton
      className={cn('cursor-pointer',className)}
      size={newSize}
      type="button"
      variant={variant}
      {...props}
    />
  );
};

export type PromptInputActionMenuProps = ComponentProps<typeof DropdownMenu>;
export const PromptInputActionMenu = (props: PromptInputActionMenuProps) => (
  <DropdownMenu {...props} />
);

export type PromptInputActionMenuTriggerProps = PromptInputButtonProps;

export const PromptInputActionMenuTrigger = ({
  className,
  children,
  ...props
}: PromptInputActionMenuTriggerProps) => (
  <DropdownMenuTrigger asChild>
    <PromptInputButton className={className} {...props}>
      {children ?? <PlusIcon className="size-4" />}
    </PromptInputButton>
  </DropdownMenuTrigger>
);

export type PromptInputActionMenuContentProps = ComponentProps<
  typeof DropdownMenuContent
>;
export const PromptInputActionMenuContent = ({
  className,
  ...props
}: PromptInputActionMenuContentProps) => (
  <DropdownMenuContent align="start" className={cn(className)} {...props} />
);

export type PromptInputActionMenuItemProps = ComponentProps<
  typeof DropdownMenuItem
>;
export const PromptInputActionMenuItem = ({
  className,
  ...props
}: PromptInputActionMenuItemProps) => (
  <DropdownMenuItem className={cn(className)} {...props} />
);

// Note: Actions that perform side-effects (like opening a file dialog)
// are provided in opt-in modules (e.g., prompt-input-attachments).

export type PromptInputSubmitProps = Omit<
  ComponentProps<typeof InputGroupButton>,
  "type"
> & {
  status?: ChatStatus;
  onCancel?: () => void;
};

export const PromptInputSubmit = ({
  className,
  variant,
  size = "icon-sm",
  status,
  onCancel,
  children,
  disabled,
  ...props
}: PromptInputSubmitProps & { disabled?: boolean }) => {
  const controller = useOptionalPromptInputController();
  const attachments = useOptionalProviderAttachments();

  const isStreaming = status === "streaming";
  const isSubmitted = status === "submitted";
  const canCancel = isStreaming && onCancel;

  // Check if there's content to submit
  const hasText = controller
    ? controller.textInput.value.trim().length > 0
    : true;
  const hasFiles = attachments ? attachments.files.length > 0 : false;
  const hasContent = hasText || hasFiles;

  // Disable when no content (unless streaming/submitted or explicitly disabled)
  const isDisabled = disabled ?? (!canCancel && !isSubmitted && !hasContent);

  let Icon = <CornerDownLeftIcon className="size-4" />;

  if (isSubmitted) {
    Icon = <Loader2Icon className="size-4 animate-spin" />;
  } else if (isStreaming) {
    Icon = <SquareIcon className="size-4" />;
  } else if (status === "error") {
    Icon = <XIcon className="size-4" />;
  }

  const resolvedVariant = variant ?? "default";

  return (
    <InputGroupButton
      aria-label={canCancel ? "Stop" : "Submit"}
      className={cn(className)}
      size={size}
      type={canCancel ? "button" : "submit"}
      variant={resolvedVariant}
      onClick={canCancel ? onCancel : undefined}
      disabled={isDisabled}
      {...props}
    >
      {children ?? Icon}
    </InputGroupButton>
  );
};

interface SpeechRecognition extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  start: () => void;
  stop: () => void;
  onstart: ((this: SpeechRecognition, ev: Event) => void) | null;
  onend: ((this: SpeechRecognition, ev: Event) => void) | null;
  onresult:
    | ((this: SpeechRecognition, ev: SpeechRecognitionEvent) => void)
    | null;
  onerror:
    | ((this: SpeechRecognition, ev: SpeechRecognitionErrorEvent) => void)
    | null;
}

interface SpeechRecognitionEvent extends Event {
  results: SpeechRecognitionResultList;
}

type SpeechRecognitionResultList = {
  readonly length: number;
  item: (index: number) => SpeechRecognitionResult;
  [index: number]: SpeechRecognitionResult;
};

type SpeechRecognitionResult = {
  readonly length: number;
  item: (index: number) => SpeechRecognitionAlternative;
  [index: number]: SpeechRecognitionAlternative;
  isFinal: boolean;
};

type SpeechRecognitionAlternative = {
  transcript: string;
  confidence: number;
};

interface SpeechRecognitionErrorEvent extends Event {
  error: string;
}

declare global {
  interface Window {
    SpeechRecognition?: {
      new (): SpeechRecognition;
    };
    webkitSpeechRecognition?: {
      new (): SpeechRecognition;
    };
  }
}

export type PromptInputSpeechButtonProps = ComponentProps<
  typeof PromptInputButton
> & {
  textareaRef?: RefObject<HTMLTextAreaElement | null>;
  onTranscriptionChange?: (text: string) => void;
};

export const PromptInputSpeechButton = ({
  className,
  textareaRef,
  onTranscriptionChange,
  ...props
}: PromptInputSpeechButtonProps) => {
  const [isListening, setIsListening] = useState(false);
  const [recognition, setRecognition] = useState<SpeechRecognition | null>(
    null
  );
  const recognitionRef = useRef<SpeechRecognition | null>(null);

  useEffect(() => {
    if (
      typeof window !== "undefined" &&
      ("SpeechRecognition" in window || "webkitSpeechRecognition" in window)
    ) {
      const SpeechRecognitionCtor =
        window.SpeechRecognition ?? window.webkitSpeechRecognition;
      if (!SpeechRecognitionCtor) {
        return;
      }
      const speechRecognition = new SpeechRecognitionCtor();

      speechRecognition.continuous = true;
      speechRecognition.interimResults = true;
      speechRecognition.lang = "en-US";

      speechRecognition.onstart = () => {
        setIsListening(true);
      };

      speechRecognition.onend = () => {
        setIsListening(false);
      };

      speechRecognition.onresult = (event) => {
        let finalTranscript = "";

        for (const result of Array.from(event.results)) {
          if (result.isFinal) {
            finalTranscript += result.item(0).transcript;
          }
        }

        if (finalTranscript && textareaRef?.current) {
          const textarea = textareaRef.current;
          const currentValue = textarea.value;
          const newValue =
            currentValue + (currentValue ? " " : "") + finalTranscript;

          textarea.value = newValue;
          textarea.dispatchEvent(new Event("input", { bubbles: true }));
          onTranscriptionChange?.(newValue);
        }
      };

      speechRecognition.onerror = (event) => {
        console.error("Speech recognition error:", event.error);
        setIsListening(false);
      };

      recognitionRef.current = speechRecognition;
      setRecognition(speechRecognition);
    }

    return () => {
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
    };
  }, [textareaRef, onTranscriptionChange]);

  const toggleListening = useCallback(() => {
    if (!recognition) return;

    if (isListening) {
      recognition.stop();
    } else {
      recognition.start();
    }
  }, [recognition, isListening]);

  return (
    <PromptInputButton
      className={cn(
        "relative transition-all duration-200",
        isListening && "animate-pulse bg-accent text-accent-foreground",
        className
      )}
      disabled={!recognition}
      onClick={toggleListening}
      {...props}
    >
      {isListening ? (
        <AudioLinesIcon className="size-4" />
      ) : (
        <MicIcon className="size-4" />
      )}
    </PromptInputButton>
  );
};

export type PromptInputModelSelectProps = ComponentProps<typeof Select>;

export const PromptInputModelSelect = (props: PromptInputModelSelectProps) => (
  <Select {...props} />
);

export type PromptInputModelSelectTriggerProps = ComponentProps<
  typeof SelectTrigger
>;

export const PromptInputModelSelectTrigger = ({
  className,
  ...props
}: PromptInputModelSelectTriggerProps) => (
  <SelectTrigger
    className={cn(
      "border-none bg-transparent font-medium text-muted-foreground shadow-none transition-colors",
      'hover:bg-accent hover:text-foreground [&[aria-expanded="true"]]:bg-accent [&[aria-expanded="true"]]:text-foreground',
      className
    )}
    {...props}
  />
);

export type PromptInputModelSelectContentProps = ComponentProps<
  typeof SelectContent
>;

export const PromptInputModelSelectContent = ({
  className,
  ...props
}: PromptInputModelSelectContentProps) => (
  <SelectContent className={cn(className)} {...props} />
);

export type PromptInputModelSelectItemProps = ComponentProps<typeof SelectItem>;

export const PromptInputModelSelectItem = ({
  className,
  ...props
}: PromptInputModelSelectItemProps) => (
  <SelectItem className={cn(className)} {...props} />
);

export type PromptInputModelSelectValueProps = ComponentProps<
  typeof SelectValue
>;

export const PromptInputModelSelectValue = ({
  className,
  ...props
}: PromptInputModelSelectValueProps) => (
  <SelectValue className={cn(className)} {...props} />
);
