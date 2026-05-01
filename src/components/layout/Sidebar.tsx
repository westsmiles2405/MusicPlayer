import { NavLink } from "react-router";
import { useQuery } from "@tanstack/react-query";
import { playlistRepo } from "@/repositories/playlistRepo";

export function Sidebar() {
  const playlists = useQuery({
    queryKey: ["playlists"],
    queryFn: () => playlistRepo.list(),
  });

  return (
    <aside className="sidebar">
      <nav aria-label="侧边栏">
        <section className="sidebar__section">
          <h3 className="sidebar__heading">资料库</h3>
          <ul className="sidebar__list">
            <li>
              <NavLink
                to="/library/recent"
                className={({ isActive }) =>
                  isActive
                    ? "sidebar__link sidebar__link--active"
                    : "sidebar__link"
                }
              >
                最近添加
              </NavLink>
            </li>
            <li>
              <NavLink
                to="/library/songs"
                className={({ isActive }) =>
                  isActive
                    ? "sidebar__link sidebar__link--active"
                    : "sidebar__link"
                }
              >
                歌曲
              </NavLink>
            </li>
            <li>
              <NavLink
                to="/library/albums"
                className={({ isActive }) =>
                  isActive
                    ? "sidebar__link sidebar__link--active"
                    : "sidebar__link"
                }
              >
                专辑
              </NavLink>
            </li>
            <li>
              <NavLink
                to="/library/artists"
                className={({ isActive }) =>
                  isActive
                    ? "sidebar__link sidebar__link--active"
                    : "sidebar__link"
                }
              >
                艺人
              </NavLink>
            </li>
          </ul>
        </section>

        <section className="sidebar__section">
          <h3 className="sidebar__heading">播放列表</h3>
          <ul className="sidebar__list">
            <li>
              <NavLink
                to="/playlists"
                end
                className={({ isActive }) =>
                  isActive
                    ? "sidebar__link sidebar__link--active"
                    : "sidebar__link"
                }
              >
                全部播放列表
              </NavLink>
            </li>
            {playlists.data?.map((p) => (
              <li key={p.id}>
                <NavLink
                  to={`/playlists/${p.id}`}
                  className={({ isActive }) =>
                    isActive
                      ? "sidebar__link sidebar__link--active"
                      : "sidebar__link"
                  }
                >
                  {p.name}
                </NavLink>
              </li>
            ))}
          </ul>
        </section>

        <section className="sidebar__section">
          <h3 className="sidebar__heading">其他</h3>
          <ul className="sidebar__list">
            <li>
              <NavLink
                to="/settings"
                className={({ isActive }) =>
                  isActive
                    ? "sidebar__link sidebar__link--active"
                    : "sidebar__link"
                }
              >
                设置
              </NavLink>
            </li>
          </ul>
        </section>
      </nav>
    </aside>
  );
}
