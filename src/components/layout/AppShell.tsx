import { Outlet } from "react-router";

export function AppShell() {
  return (
    <div className="flex h-full flex-col">
      <div className="flex-1">
        <Outlet />
      </div>
    </div>
  );
}
