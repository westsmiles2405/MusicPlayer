import { useEffect } from "react";
import { useUIStore } from "@/stores/uiStore";

export function useSystemTheme() {
  const setResolvedTheme = useUIStore((s) => s.setResolvedTheme);
  const resolvedTheme = useUIStore((s) => s.resolvedTheme);

  useEffect(() => {
    if (typeof window.matchMedia !== "function") {
      setResolvedTheme("light");
      return;
    }
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = (isDark: boolean) =>
      setResolvedTheme(isDark ? "dark" : "light");
    apply(mq.matches);
    const onChange = (e: MediaQueryListEvent) => apply(e.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [setResolvedTheme]);

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
  }, [resolvedTheme]);
}
