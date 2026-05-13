"use client";

import { useRouter } from "next/navigation";
import { ProjectSummary, ProjectType } from "../lib/types";

const projectTypeLabels: Record<ProjectType, string> = {
  nextjs: "Next.js",
  node: "Node",
  python: "Python",
  rust: "Rust",
  unknown: "Project",
};

const projectTypeColors: Record<ProjectType, string> = {
  nextjs: "bg-blue-900/30 text-blue-400 border-blue-800/50",
  node: "bg-green-900/30 text-green-400 border-green-800/50",
  python: "bg-yellow-900/30 text-yellow-400 border-yellow-800/50",
  rust: "bg-orange-900/30 text-orange-400 border-orange-800/50",
  unknown: "bg-zinc-800 text-zinc-400 border-zinc-700",
};

interface ProjectCardProps {
  project: ProjectSummary;
  onInitialize: (project: ProjectSummary) => void;
  onSelect?: (project: ProjectSummary) => void;
}

export function ProjectCard({ project, onInitialize, onSelect }: ProjectCardProps) {
  const router = useRouter();
  const typeLabel = projectTypeLabels[project.project_type] || "Project";
  const typeColorClass = projectTypeColors[project.project_type] || projectTypeColors.unknown;

  const handleCardClick = () => {
    if (project.has_context) {
      if (onSelect) {
        onSelect(project);
      } else {
        router.push(`/project/${project.id}`);
      }
    }
  };

  const handleInitClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onInitialize(project);
  };

  return (
    <div
      onClick={handleCardClick}
      className="p-5 border border-zinc-800/80 rounded-xl bg-zinc-900/40 hover:bg-zinc-900/80 hover:border-zinc-700/80 transition-all duration-150 cursor-pointer"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-medium text-zinc-100 truncate">{project.name}</h3>
            <span className={`shrink-0 px-2 py-0.5 text-xs rounded-md border ${typeColorClass}`}>
              {typeLabel}
            </span>
          </div>
          <p className="text-xs text-zinc-500 truncate">{project.path}</p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {project.has_context ? (
            <span className="px-2 py-1 text-xs bg-emerald-900/30 text-emerald-400 rounded-md border border-emerald-800/50 font-medium">
              Ready
            </span>
          ) : (
            <button
              onClick={handleInitClick}
              className="px-2 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-400 hover:text-zinc-200 rounded-md border border-zinc-700/50 transition-colors"
            >
              + Add context
            </button>
          )}
        </div>
      </div>

      {project.last_opened_at && (
        <p className="text-xs text-zinc-600 mt-3">
          Last opened {formatRelativeTime(project.last_opened_at)}
        </p>
      )}
    </div>
  );
}

function formatRelativeTime(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "today";
    if (diffDays === 1) return "yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    return date.toLocaleDateString();
  } catch {
    return dateStr;
  }
}