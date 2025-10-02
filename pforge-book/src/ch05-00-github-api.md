# GitHub API: HTTP Handler Overview

HTTP handlers wrap REST APIs as MCP tools with zero boilerplate. This chapter demonstrates building a GitHub API integration using HTTP handlers.

## Why HTTP Handlers?

**Use HTTP handlers when**:
- Wrapping existing REST APIs
- No complex logic needed (just proxying)
- URL parameters can be templated
- Response doesn't need transformation

**Don't use HTTP handlers when**:
- Complex authentication flow (OAuth, JWT refresh)
- Response needs parsing/transformation
- API requires request signing
- Stateful session management needed

## GitHub API Server Example

```yaml
forge:
  name: github-api
  version: 0.1.0
  transport: stdio

tools:
  - type: http
    name: get_user
    description: "Get GitHub user information"
    endpoint: "https://api.github.com/users/{{username}}"
    method: GET
    headers:
      User-Agent: "pforge-github-client"
      Accept: "application/vnd.github.v3+json"
    params:
      username:
        type: string
        required: true
        description: "GitHub username"

  - type: http
    name: get_repos
    description: "List user repositories"
    endpoint: "https://api.github.com/users/{{username}}/repos"
    method: GET
    headers:
      User-Agent: "pforge-github-client"
      Accept: "application/vnd.github.v3+json"
    params:
      username:
        type: string
        required: true

  - type: http
    name: search_repos
    description: "Search GitHub repositories"
    endpoint: "https://api.github.com/search/repositories"
    method: GET
    headers:
      User-Agent: "pforge-github-client"
      Accept: "application/vnd.github.v3+json"
    query:
      q: "{{query}}"
      sort: "{{sort}}"
      order: "{{order}}"
    params:
      query:
        type: string
        required: true
      sort:
        type: string
        required: false
        default: "stars"
      order:
        type: string
        required: false
        default: "desc"
```

## HTTP Handler Anatomy

### 1. Endpoint and Method

```yaml
endpoint: "https://api.github.com/users/{{username}}"
method: GET
```

**Supported methods**: GET, POST, PUT, DELETE, PATCH

### 2. URL Templating

```yaml
endpoint: "https://api.example.com/{{resource}}/{{id}}"

# Input: { "resource": "users", "id": "123" }
# URL: https://api.example.com/users/123
```

### 3. Headers

```yaml
headers:
  User-Agent: "pforge-client"
  Accept: "application/json"
  Content-Type: "application/json"
  X-API-Key: "{{api_key}}"  # Can be templated
```

### 4. Query Parameters

```yaml
query:
  page: "{{page}}"
  limit: "{{limit}}"

# Input: { "page": "2", "limit": "50" }
# URL: ?page=2&limit=50
```

### 5. Request Body (POST/PUT)

```yaml
tools:
  - type: http
    name: create_issue
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/issues"
    method: POST
    headers:
      Authorization: "token {{token}}"
    body:
      title: "{{title}}"
      body: "{{description}}"
      labels: "{{labels}}"
    params:
      owner:
        type: string
        required: true
      repo:
        type: string
        required: true
      token:
        type: string
        required: true
      title:
        type: string
        required: true
      description:
        type: string
        required: false
      labels:
        type: array
        items: { type: string }
        required: false
```

## Input/Output Structure

### HTTP Input

```json
{
  "body": {  // Optional - for POST/PUT/PATCH
    "key": "value"
  },
  "query": {  // Optional - query parameters
    "param": "value"
  }
}
```

### HTTP Output

```json
{
  "status": 200,
  "body": { /* JSON response */ },
  "headers": {
    "content-type": "application/json",
    "x-ratelimit-remaining": "59"
  }
}
```

## Real-World Example: Complete GitHub Integration

```yaml
forge:
  name: github-mcp
  version: 0.1.0
  transport: stdio

tools:
  # User operations
  - type: http
    name: get_user
    description: "Get user profile"
    endpoint: "https://api.github.com/users/{{username}}"
    method: GET
    headers:
      User-Agent: "pforge-github"

  # Repository operations
  - type: http
    name: get_repo
    description: "Get repository details"
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}"
    method: GET
    headers:
      User-Agent: "pforge-github"

  - type: http
    name: list_commits
    description: "List repository commits"
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/commits"
    method: GET
    query:
      per_page: "{{per_page}}"
      page: "{{page}}"
    params:
      owner: { type: string, required: true }
      repo: { type: string, required: true }
      per_page: { type: integer, required: false, default: 30 }
      page: { type: integer, required: false, default: 1 }

  # Issue operations
  - type: http
    name: list_issues
    description: "List repository issues"
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/issues"
    method: GET
    query:
      state: "{{state}}"
      labels: "{{labels}}"
    params:
      owner: { type: string, required: true }
      repo: { type: string, required: true }
      state: { type: string, required: false, default: "open" }
      labels: { type: string, required: false }

  - type: http
    name: create_issue
    description: "Create a new issue"
    endpoint: "https://api.github.com/repos/{{owner}}/{{repo}}/issues"
    method: POST
    headers:
      Authorization: "token {{token}}"
    body:
      title: "{{title}}"
      body: "{{body}}"
    params:
      owner: { type: string, required: true }
      repo: { type: string, required: true }
      token: { type: string, required: true }
      title: { type: string, required: true }
      body: { type: string, required: false }
```

## Error Handling

HTTP handlers return errors on:

1. **Network failures**: Connection refused, timeout
2. **HTTP 4xx/5xx**: Client/server errors
3. **Invalid JSON**: Response parsing failed

**Error format**:

```json
{
  "error": "Http: Request failed: 404 Not Found"
}
```

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Dispatch overhead | 10-20μs |
| HTTP request time | 50-500ms (network dependent) |
| JSON parsing | 1-10μs/KB |
| Memory per request | ~5KB |

## When to Use Native vs HTTP Handler

**HTTP Handler** - Simple API proxying:
```yaml
type: http
endpoint: "https://api.example.com/{{resource}}"
method: GET
```

**Native Handler** - Complex logic:
```rust
async fn handle(&self, input: Input) -> Result<Output> {
    // Validate input
    // Make HTTP request
    // Transform response
    // Handle pagination
    Ok(output)
}
```

## Next Steps

Chapter 5.1 covers HTTP configuration in depth, including advanced header management, authentication patterns, and retry strategies.

---

> "APIs are tools. HTTP handlers make them accessible." - pforge HTTP philosophy
