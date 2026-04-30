import { Routes, Route, Link, useLocation } from "react-router";
import { ScanProgressBar } from "@/components/layout/ScanProgressBar";
import { LibraryPage } from "@/pages/LibraryPage";
import { SettingsPage } from "@/pages/SettingsPage";

function Nav() {
  const { pathname } = useLocation();
  return (
    <nav className="flex gap-4 px-8 py-3 text-sm border-b border-white/10 bg-black/60 backdrop-blur-md">
      <Link
        to="/"
        className={pathname === "/" ? "text-white font-medium" : "text-white/60 hover:text-white"}
      >
        资料库
      </Link>
      <Link
        to="/settings"
        className={pathname === "/settings" ? "text-white font-medium" : "text-white/60 hover:text-white"}
      >
        设置
      </Link>
    </nav>
  );
}

export default function App() {
  return (
    <>
      <Nav />
      <Routes>
        <Route path="/" element={<LibraryPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
      <ScanProgressBar />
    </>
  );
}
