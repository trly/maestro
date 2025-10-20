import { Octokit } from "@octokit/rest";
import type { Repository, RepositoryProvider } from "./types";
import { tokenStore } from "$lib/tokenStore";

export class GitHubProvider implements RepositoryProvider {
  name = "GitHub";
  private octokit: Octokit | null = null;
  private initialized = false;
  private initPromise: Promise<void> | null = null;

  constructor() {
    // Don't auto-initialize in constructor
  }

  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }
    
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = this.initializeToken();
    await this.initPromise;
  }

  private async initializeToken(): Promise<void> {
    try {
      const tokens = await tokenStore.getAllTokens();
      if (tokens.githubToken) {
        this.octokit = new Octokit({ auth: tokens.githubToken });
      }
    } finally {
      this.initialized = true;
    }
  }

  isConfigured(): boolean {
    return this.octokit !== null;
  }

  async searchRepositories(query: string): Promise<Repository[]> {
    await this.initialize();
    
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

      return data.items
        .filter((repo) => repo.owner)
        .map((repo) => ({
          provider: "github" as const,
          fullName: repo.full_name,
          name: repo.name,
          owner: repo.owner!.login,
          url: repo.html_url,
          description: repo.description || undefined,
        }));
    } catch (error) {
      return [];
    }
  }

  async getUserRepositories(): Promise<Repository[]> {
    await this.initialize();
    
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
      return [];
    }
  }
}
