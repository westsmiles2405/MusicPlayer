import { Routes, Route } from "react-router";

export default function App() {
  return (
    <Routes>
      <Route
        path="/"
        element={
          <div className="flex h-full items-center justify-center text-2xl font-semibold text-apple-red">
            MusicPlayer v0.1.0
          </div>
        }
      />
    </Routes>
  );
}
