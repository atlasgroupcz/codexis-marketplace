import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { KeyRound } from "lucide-react";
import { ConnectionSection } from "@workspace/ui/components/connection-section";
import { Skeleton } from "@workspace/ui/components/skeleton";
import { Toaster, toast } from "@workspace/ui/components/sonner";
import type { ConnectionSectionLabels } from "@workspace/ui/components/connection-section";
import type { Settings } from "@/lib/api";
import { deleteToken, fetchSettings, saveToken } from "@/lib/api";

/**
 * Whether a token is stored. `unknown` until the first fetch answers, and
 * treated as its own state rather than folded into "not connected" - the page
 * would otherwise open by claiming, in red, something it has not checked.
 */
type ConnState = "unknown" | "connected" | "disconnected";

/**
 * Datify's brand mark, its two shapes taken verbatim from their logo.svg.
 *
 * They sit either side of a gap there, and that gap is exactly what this page
 * is about - so the mark carries the answer: the bar slides away and both go
 * grey when no token is stored, and closes up in Datify blue once one is. The
 * wording beside it says the same thing, so nothing depends on noticing it.
 *
 * While unknown it stays closed and grey, so the gap only ever opens to report
 * a real answer instead of flicking open and shut as the first fetch lands.
 */
function DatifyMark({ state }: { state: ConnState }) {
  return (
    <svg
      viewBox="0 0 150 96.39"
      className="datify-mark h-11 w-auto shrink-0"
      data-state={state}
      aria-hidden="true"
    >
      <rect data-part="bar" width="19.88" height="96.39" />
      <path
        data-part="bracket"
        d="M134.28,18.07v60.25c0,9.98-8.09,18.07-18.07,18.07H39.79l-19.87-19.88h87.6c3.8,0,6.88-3.08,6.88-6.88V26.77c0-3.8-3.08-6.88-6.88-6.88h-32.67L55.11.14V0h61.11c9.98,0,18.07,8.09,18.07,18.07Z"
      />
    </svg>
  );
}

/**
 * The connection state on the section heading, above the field - the same dot
 * and wording the other component apps show, so it reads identically across
 * the marketplace. Green and red are the state; Datify's blue stays on the
 * actions.
 */
function ConnectionStatus({ state }: { state: ConnState }) {
  const { t } = useTranslation();
  // Nothing to report yet - hold the row's height so the heading does not jump
  // when the answer arrives.
  if (state === "unknown") {
    return <span className="h-4" aria-hidden="true" />;
  }
  const connected = state === "connected";
  const colour = connected ? "var(--success)" : "var(--destructive)";
  return (
    <span
      className="inline-flex shrink-0 items-center gap-1.5 text-[11px] font-semibold tracking-normal normal-case"
      style={{ color: colour }}
    >
      <span
        aria-hidden="true"
        className="size-2 rounded-full"
        style={{ background: colour }}
      />
      {connected ? t("connection.connected") : t("connection.notConnected")}
    </span>
  );
}

export function App() {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);

  const refetch = useCallback(async () => {
    try {
      setSettings(await fetchSettings());
      setLoadFailed(false);
    } catch {
      setLoadFailed(true);
    }
  }, []);

  useEffect(() => {
    void refetch();
  }, [refetch]);

  // Three states, not two: `unknown` until the first fetch answers.
  const state: ConnState =
    settings === null
      ? "unknown"
      : settings.configured
        ? "connected"
        : "disconnected";

  const labels: ConnectionSectionLabels = {
    connected: t("connection.connected"),
    notConnected: t("connection.notConnected"),
    remove: t("connection.remove"),
    confirmRemove: t("connection.confirmRemove"),
    save: t("connection.save"),
    saving: t("connection.saving"),
  };

  /**
   * Must reject on a token Datify refuses, so ConnectionSection renders the
   * reason inline instead of the page reporting a save that did not happen.
   */
  const handleSave = async (values: Record<string, string>) => {
    await saveToken(values.apiKey ?? "");
    await refetch();
    toast.success(t("app.saved"));
  };

  const handleDelete = () => {
    deleteToken()
      .then(refetch)
      .then(() => toast.success(t("app.deleted")))
      .catch((e: unknown) =>
        toast.error(e instanceof Error ? e.message : t("app.saveFailed")),
      );
  };

  return (
    <div className="mx-auto max-w-xl space-y-6 p-6">
      <header className="flex items-center gap-4">
        <DatifyMark state={state} />
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-lg font-semibold tracking-tight">
            Datify
          </h1>
          {/* Same rule as the status dot: leadIdle is the "no token yet"
              wording, so showing it before the answer arrives asserts a state
              nothing has checked. Reserve the line instead. */}
          {state === "unknown" ? (
            <Skeleton className="mt-0.5 h-5 w-56" />
          ) : (
            <p className="text-muted-foreground mt-0.5 text-sm">
              {state === "connected"
                ? t("app.leadConnected")
                : t("app.leadIdle")}
            </p>
          )}
        </div>
      </header>

      {loadFailed && (
        <p className="text-destructive text-sm" role="alert">
          {t("app.loadFailed")}
        </p>
      )}

      <section className="border-border rounded-xl border p-5">
        {/* Same heading treatment the settings dialog uses for this section in
            the other apps, minus its own padding: there the breathing room
            above comes from a separator and the dialog's own spacing, so
            reusing pt-3 here left it stuck to the card's edge. */}
        <div className="text-muted-foreground mb-4 flex items-center justify-between gap-2 text-[10.5px] leading-none font-semibold tracking-[0.08em] uppercase">
          <span className="flex items-center gap-1.5">
            <KeyRound className="size-3 shrink-0" />
            {t("app.connectionSection")}
          </span>
          <ConnectionStatus state={state} />
        </div>
        {settings === null ? (
          // Rendering the section before the answer arrives would show the
          // empty field for a frame and then swap it for the stored key.
          //
          // This mirrors ConnectionSection's own three blocks at their real
          // heights - field, reserved error slot, then the get-key link over
          // its hint beside the action button. A shorter placeholder is why
          // the card used to grow as the answer landed.
          <div className="space-y-2.5" aria-busy="true">
            <Skeleton className="h-9 w-full" />
            <div className="min-h-[1.25rem]" />
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0 space-y-1">
                <Skeleton className="h-5 w-32" />
                <Skeleton className="h-4 w-44" />
              </div>
              <Skeleton className="h-8 w-20 shrink-0" />
            </div>
          </div>
        ) : (
          <ConnectionSection
            configured={settings.configured}
            maskedKey={settings.maskedKey}
            fields={[
              {
                name: "apiKey",
                label: t("settings.label"),
                type: "password",
                placeholder: t("settings.placeholder"),
                mono: true,
              },
            ]}
            getKey={{
              href: settings.generateUrl,
              label: t("settings.getKey"),
              hint: t("settings.getKeyHint"),
            }}
            labels={labels}
            onSave={handleSave}
            onDelete={handleDelete}
          />
        )}
      </section>

      {settings?.keySource && settings.keySource !== settings.settingsFile && (
        <p className="text-muted-foreground text-xs">
          {t("app.overridden", { source: settings.keySource })}
        </p>
      )}

      <Toaster />
    </div>
  );
}
