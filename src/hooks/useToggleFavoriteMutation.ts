import { useMutation, useQueryClient } from "@tanstack/react-query";
import { favoriteRepo } from "@/repositories/favoriteRepo";
import type { Track } from "@/repositories/trackRepo";

export function useToggleFavoriteMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ track, favorite }: { track: Track; favorite: boolean }) =>
      favoriteRepo.set(track.id, favorite),
    onMutate: async ({ track, favorite }) => {
      await Promise.all([
        queryClient.cancelQueries({ queryKey: ["favoriteTracks"] }),
        queryClient.cancelQueries({ queryKey: ["tracks"] }),
        queryClient.cancelQueries({ queryKey: ["search"] }),
      ]);

      const previousFavorites = queryClient.getQueryData<Track[]>([
        "favoriteTracks",
      ]);

      queryClient.setQueryData<Track[]>(
        ["favoriteTracks"],
        (current = []) => {
          if (favorite) {
            const exists = current.some((item) => item.id === track.id);
            return exists
              ? current
              : [{ ...track, isFavorite: true }, ...current];
          }
          return current.filter((item) => item.id !== track.id);
        },
      );

      return { previousFavorites };
    },
    onError: (_error, _vars, context) => {
      if (context?.previousFavorites) {
        queryClient.setQueryData(
          ["favoriteTracks"],
          context.previousFavorites,
        );
      }
    },
    onSuccess: (_data, vars) => {
      queryClient.invalidateQueries({ queryKey: ["favoriteTracks"] });
      queryClient.invalidateQueries({ queryKey: ["tracks"] });
      queryClient.invalidateQueries({ queryKey: ["search"] });
      if (vars.track.albumId) {
        queryClient.invalidateQueries({
          queryKey: ["albumTracks", vars.track.albumId],
        });
      }
      if (vars.track.primaryArtistId) {
        queryClient.invalidateQueries({
          queryKey: ["artistTracks", vars.track.primaryArtistId],
        });
      }
    },
  });
}
