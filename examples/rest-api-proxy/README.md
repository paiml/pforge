# REST API Proxy - pforge Example

Demonstrates using pforge to create HTTP tool handlers that proxy REST APIs.

## What It Does

Provides GitHub API access through MCP tools:
- `get_user` - Fetch user information
- `get_repos` - Get user repositories
- `search_repos` - Search for repositories

All without writing any handler code - just YAML configuration!

## Configuration

See `pforge.yaml` - all tools are defined declaratively:

```yaml
tools:
  - type: http
    name: get_user
    endpoint: "https://api.github.com/users/{{username}}"
    method: GET
    headers:
      User-Agent: "pforge-example"
```

## Features Demonstrated

- **HTTP Handler**: No code needed for REST API tools
- **URL Templates**: `{{username}}` parameter substitution
- **Custom Headers**: User-Agent, Accept, etc.
- **Multiple Endpoints**: Different API calls from one config

## Running

```bash
cd examples/rest-api-proxy
cargo run
```

## How It Works

1. pforge reads `pforge.yaml`
2. Creates HTTP handlers automatically
3. Handles parameter substitution
4. Manages request/response

Zero Rust code for the API handlers - pure configuration!
