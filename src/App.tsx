import { Route, Routes } from "react-router";
import { AppShell } from "@/components/layout";
import RecentPage from "@/pages/RecentPage";
import SongsPage from "@/pages/SongsPage";
import AlbumsPage from "@/pages/AlbumsPage";
import AlbumDetailPage from "@/pages/AlbumDetailPage";
import ArtistsPage from "@/pages/ArtistsPage";
import ArtistDetailPage from "@/pages/ArtistDetailPage";
import PlaylistsPage from "@/pages/PlaylistsPage";
import PlaylistDetailPage from "@/pages/PlaylistDetailPage";
import SettingsPage from "@/pages/SettingsPage";
import SearchPage from "@/pages/SearchPage";
import FavoritesPage from "@/pages/FavoritesPage";
import RecentPlaysPage from "@/pages/RecentPlaysPage";
import NotFoundRedirect from "@/pages/NotFoundRedirect";

export default function App() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<NotFoundRedirect />} />
        <Route path="search" element={<SearchPage />} />
        <Route path="library">
          <Route path="recent" element={<RecentPage />} />
          <Route path="songs" element={<SongsPage />} />
          <Route path="albums" element={<AlbumsPage />} />
          <Route path="albums/:id" element={<AlbumDetailPage />} />
          <Route path="artists" element={<ArtistsPage />} />
          <Route path="artists/:id" element={<ArtistDetailPage />} />
          <Route path="favorites" element={<FavoritesPage />} />
          <Route path="recent-plays" element={<RecentPlaysPage />} />
        </Route>
        <Route path="playlists" element={<PlaylistsPage />} />
        <Route path="playlists/:id" element={<PlaylistDetailPage />} />
        <Route path="settings" element={<SettingsPage />} />
        <Route path="*" element={<NotFoundRedirect />} />
      </Route>
    </Routes>
  );
}
