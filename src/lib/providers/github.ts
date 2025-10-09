import { Octokit } from "@octokit/rest";
import type { Repository, RepositoryProvider } from "./types";

export class GitHubProvider implements RepositoryProvider {
  name = "GitHub";
  private octokit: Octokit | null = null;

  constructor() {
    const token = import.meta.env.VITE_MAESTRO_GITHUB_TOKEN;
    if (token) {
      this.octokit = new Octokit({ auth: token });
    }
  }

  isConfigured(): boolean {
    return this.octokit !== null;
  }

  async searchRepositories(query: string): Promise<Repository[]> {
    if (!this.octokit) {
      throw new Error("GitHub token not configured");
    }

    if (!query.trim()) {
      return this.getUserRepositories();
    }

    try {
      const { data } = await this.octokit.search.repos({
        q: query,
        per_page: 20,
        sort: "updated",
      });

      return data.items.map((repo) => ({
        provider: "github" as const,
        fullName: repo.full_name,
        name: repo.name,
        owner: repo.owner.login,
        url: repo.html_url,
        description: repo.description || undefined,
      }));
    } catch (error) {
      console.error("GitHub search error:", error);
      return [];
    }
  }

  async getUserRepositories(): Promise<Repository[]> {
    if (!this.octokit) {
      throw new Error("GitHub token not configured");
    }

    try {
      const { data } = await this.octokit.repos.listForAuthenticatedUser({
        per_page: 100,
        sort: "updated",
        affiliation: "owner,collaborator,organization_member",
      });

      return data.map((repo) => ({
        provider: "github" as const,
        fullName: repo.full_name,
        name: repo.name,
        owner: repo.owner.login,
        url: repo.html_url,
        description: repo.description || undefined,
      }));
    } catch (error) {
      console.error("GitHub repos error:", error);
      return [];
    }
  }
}
