use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcegraphRepository {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub language: Option<String>,
    pub stars: i32,
    pub is_private: bool,
    pub is_fork: bool,
    pub is_archived: bool,
    pub external_repository: ExternalRepository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalRepository {
    pub service_type: String,
    #[serde(alias = "serviceID")]
    pub service_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySearchResult {
    pub repositories: Vec<SourcegraphRepository>,
    pub total_count: i32,
    pub has_next_page: bool,
}

pub struct SourcegraphClient {
    endpoint: String,
    access_token: String,
    http_client: reqwest::Client,
}

impl SourcegraphClient {
    pub fn new(endpoint: String, access_token: String) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .user_agent(format!("maestro/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self {
            endpoint,
            access_token,
            http_client,
        })
    }

    async fn query<T>(&self, query: &str, variables: serde_json::Value) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/.api/graphql", self.endpoint.trim_end_matches('/'));

        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("token {}", self.access_token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "GraphQL request failed with status {}: {}",
                status,
                error_text
            ));
        }

        #[derive(Deserialize)]
        struct GraphQLResponse<T> {
            data: Option<T>,
            errors: Option<Vec<serde_json::Value>>,
        }

        let result: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = result.errors {
            return Err(anyhow::anyhow!(
                "GraphQL errors: {}",
                serde_json::to_string_pretty(&errors)?
            ));
        }

        result
            .data
            .ok_or_else(|| anyhow::anyhow!("GraphQL response contained no data"))
    }

    pub async fn search_repositories(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<RepositorySearchResult> {
        // Use the search() API which supports full Sourcegraph search syntax.
        // For queries like "file:gradle-wrapper.properties", the API returns
        // FileMatch results which we need to extract repositories from.
        let graphql_query = r#"
			query SearchRepositories($query: String!) {
				search(query: $query, version: V3, patternType: standard) {
					results {
						results {
							__typename
							... on FileMatch {
								repository {
									id
									name
									description
									url
									language
									stars
									isPrivate
									isFork
									isArchived
									externalRepository {
										serviceType
										serviceID
									}
								}
							}
							... on Repository {
								id
								name
								description
								url
								language
								stars
								isPrivate
								isFork
								isArchived
								externalRepository {
									serviceType
									serviceID
								}
							}
						}
						matchCount
						limitHit
					}
				}
			}
		"#;

        // Add count:N to limit results
        let search_query = format!("{} count:{}", query, limit);

        let variables = serde_json::json!({
            "query": search_query,
        });

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            search: SearchResults,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SearchResults {
            results: SearchResultsData,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SearchResultsData {
            results: Vec<SearchResult>,
            match_count: i32,
            limit_hit: bool,
        }

        #[derive(Deserialize)]
        #[serde(tag = "__typename")]
        enum SearchResult {
            #[serde(rename_all = "camelCase")]
            FileMatch { repository: SourcegraphRepository },
            #[serde(rename_all = "camelCase")]
            Repository {
                id: String,
                name: String,
                description: Option<String>,
                url: String,
                language: Option<String>,
                stars: i32,
                is_private: bool,
                is_fork: bool,
                is_archived: bool,
                external_repository: ExternalRepository,
            },
            #[serde(other)]
            Other,
        }

        let response: Response = self.query(graphql_query, variables).await?;

        // Extract repositories from polymorphic results and deduplicate by name
        let mut seen_repos = std::collections::HashSet::new();
        let mut repositories = Vec::new();

        for result in response.search.results.results {
            let repo = match result {
                SearchResult::FileMatch { repository } => repository,
                SearchResult::Repository {
                    id,
                    name,
                    description,
                    url,
                    language,
                    stars,
                    is_private,
                    is_fork,
                    is_archived,
                    external_repository,
                } => SourcegraphRepository {
                    id,
                    name,
                    description,
                    url,
                    language,
                    stars,
                    is_private,
                    is_fork,
                    is_archived,
                    external_repository,
                },
                SearchResult::Other => continue,
            };

            if seen_repos.insert(repo.name.clone()) {
                repositories.push(repo);
            }
        }

        Ok(RepositorySearchResult {
            repositories,
            total_count: response.search.results.match_count,
            has_next_page: response.search.results.limit_hit,
        })
    }
}
