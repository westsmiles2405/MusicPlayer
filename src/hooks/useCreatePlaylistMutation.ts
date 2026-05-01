import { useMutation, useQueryClient } from "@tanstack/react-query";
import { playlistRepo } from "@/repositories/playlistRepo";

export function useCreatePlaylistMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => playlistRepo.create(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
    },
  });
}
