import { usePlayerStore } from "@/stores/playerStore";

export function usePlayer() {
  const store = usePlayerStore();
  return {
    ...store,
    play: () => {},
    pause: () => {},
    next: () => {},
    prev: () => {},
  };
}
