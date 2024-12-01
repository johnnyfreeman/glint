<div align="center">
  <h1><code>glint</code></h1>

  <p><b>A local-only, git-friendly scratchpad for testing API endpoints in your terminal. Define, chain, and automate HTTP requests using simple TOML configuration files.</b></p>
</div>

> [!WARNING]
> **This project is in its early stages and may undergo many breaking changes in the future.**

![glint](https://github.com/user-attachments/assets/cc81d961-9a4a-4a0b-8703-6e47cced9762)

## Features

- **Http Request Collections:** Define collections of HTTP requests in a `requests.toml` file.
- **Dynamic Placeholders:** Use placeholders in URLs, headers, and bodies that are resolved at runtime.
- **Flexible Dependency Resolution:** Resolve placeholders from multiple sources:
  - Environment Variables
  - Environment files (TOML)
  - Previous request responses
  - User prompts
  - 1Password vaults
- **Automatic Value Caching:** Save user-prompted values to the environment file for future use.
- **Supports All HTTP Methods:** Easily configure HTTP methods in your request definitions.

## Installation

1. **Clone the Repository:**

   ```bash
   git clone git@github.com:johnnyfreeman/glint.git
   cd glint
   ```

2. **Build and Install the Application:**

   ```bash
   cargo install --path .
   ```

You can now run `glint` from your terminal.

## Usage

### Running the Application

To run `glint` with a specific example file, use the following command:

```bash
glint examples/github.toml
```

You can find more examples in the [examples/](examples/) directory.

### Application Behavior

- **Placeholder Resolution:**
  - Placeholders like `{name}` and `{email}` in your requests are replaced with actual values at runtime.
  - If a value is missing in the `env_file`, the application will prompt you to input it, saving it for future use.
  - Placeholders can also be resolved using 1Password credentials for added security.

- **Request Execution:**
  - Requests are executed sequentially, following the order defined in the `requests.toml` file.
  - You can extract values from previous responses to populate placeholders in subsequent requests.

- **Dependency Handling:**
  - Dependency sources include:
    - `envfile`: Reads values from a TOML file.
    - `envvar`: Reads values from environment variables.
    - `request`: Retrieves values from previous request responses.
    - `onepassword`: Retrieves values from a 1Password vault.

## Configuration

### Request Definition

Each request is defined under the `[[requests]]` table in the `requests.toml` file. The request can include the following fields:

- **`name`**: A unique identifier for the request.
- **`method`**: The HTTP method to use (e.g., `GET`, `POST`).
- **`url`**: The target URL, which may include placeholders.
- **`headers`**: Optional HTTP headers.
- **`body`**: Optional request body, which may include placeholders.
- **`dependencies`**: Dependencies required to resolve placeholders before sending the request.

### Dependencies

Dependencies specify how placeholders should be resolved. Supported sources include:

- **`envfile`**: Reads values from a TOML file.
  - **`env_file`**: Path to the environment file.
  - **`key`**: The key to look for in the file.
  - **`prompt`**: (Optional) Prompt message if the key is not found.
- **`envvar`**: Reads values from a TOML file.
  - **`name`**: The environment variable name.
  - **`prompt`**: (Optional) Prompt message if the variable is not found.
- **`request`**: Extracts values from a previous request's response.
  - **`request`**: The name of the previous request.
  - **`path`**: The JSONPath expression to extract the value (e.g., `$.token`).
- **`onepassword`**: Retrieves values from a 1Password vault.
  - **`vault`**: The name of the vault where the value is stored.
  - **`item`**: The item name or identifier containing the value.
  - **`field`**: The specific field within the item to use.

### Placeholders

Placeholders, denoted by `{placeholder_name}`, are dynamically resolved from their dependencies at runtime. They can be used in the URL, headers, or body of the request.
