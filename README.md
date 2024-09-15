# send

A simple HTTP client for the terminal. The CLI allows you to define HTTP requests with dependencies, placeholders, and dynamic data sourcing from various sources like environment variables, files, or previous request responses.

## Features

- **Chain Multiple HTTP Requests:** Define a sequence of HTTP requests in a `requests.toml` file.
- **Dynamic Placeholders:** Use placeholders in URLs, headers, and bodies that are resolved at runtime.
- **Flexible Dependency Resolution:** Resolve placeholders from various sources:
  - Environment files (TOML files)
  - Previous request responses
  - User prompts
- **Automatic Value Caching:** Save prompted values back to the environment file for future use.
- **Supports GET and POST Methods:** Easily configure different HTTP methods.

## Installation

1. **Clone the Repository:**

 ```bash
 git clone git@github.com:johnnyfreeman/send.git
 cd send
 ```

2. **Build and Install the Application:**

 ```bash
 cargo install
 ```

You can then run `send` from your terminal.

## Usage

### Defining Requests

Create a `requests.toml` file. This file defines the HTTP requests and their dependencies.

**Example `requests.toml`:**

```toml
[[requests]]
name = "Get Token"
method = "POST"
url = "{url}/token"
body = '''
{
  "name": "{name}",
  "email": "{email}"
}
'''
[requests.dependencies.url]
source = "envfile"
env_file = "my-api-environment.toml"
key = "url"
[requests.dependencies.name]
source = "envfile"
env_file = "my-api-environment.toml"
key = "name"
[requests.dependencies.email]
source = "envfile"
env_file = "my-api-environment.toml"
key = "email"

[[requests]]
name = "Get Appointments"
method = "GET"
url = "{url}/appointments"
[requests.headers]
Authorization = "Bearer {token}"
[requests.dependencies.url]
source = "envfile"
env_file = "my-api-environment.toml"
key = "url"
[requests.dependencies.token]
source = "request"
request = "Get Token"
path = "$.token"
[requests.dependencies.id]
source = "envfile"
env_file = "my-api-environment.toml"
key = "id"
prompt = "Enter appointment ID"
```

### Defining Environment Variables

Create an `my-api-environment.toml` file containing the necessary variables:

**Example `my-api-environment.toml`:**

```toml
url = "http://localhost:8000"
name = "John Doe"
email = "john.doe@example.com"
```

### Running the Application

Run the application:

```bash
send
```

### Behavior

- **Placeholder Resolution:**
  - Placeholders like `{name}` and `{email}` in your requests are replaced with actual values at runtime.
  - If a value is not found in the specified `env_file`, the application prompts you to enter it.
  - Entered values are saved back to the `env_file` for future use.

- **Request Execution:**
  - The application executes requests in the order they are defined in `requests.toml`.
  - Responses from previous requests can be used to resolve placeholders in subsequent requests.

- **Dependency Handling:**
  - Supports dependencies of types:
    - `envfile`: Reads values from a specified TOML file.
    - `request`: Extracts values from previous request responses.

## Configuration

### Request Definition

Each request is defined under the `[[requests]]` table in the `requests.toml` file. The request may include:

- **`name`**: A unique identifier for the request.
- **`method`**: HTTP method (e.g., `GET`, `POST`).
- **`url`**: The endpoint URL, which may include placeholders.
- **`headers`**: Optional HTTP headers.
- **`body`**: Optional request body, which may include placeholders.
- **`dependencies`**: A set of dependencies required to resolve placeholders.

### Dependencies

Dependencies specify how to resolve placeholders in the request. The supported dependency sources are:

- **`envfile`**: Reads the value from a specified TOML file.
  - **`env_file`**: The path to the TOML file.
  - **`key`**: The key within the TOML file to retrieve.
  - **`prompt`**: Optional prompt to display if the key is not found.
- **`request`**: Extracts the value from a previous request's response.
  - **`request`**: The name of the previous request.
  - **`path`**: The JSONPath expression to the value in the response (e.g., `$.token`).

### Placeholders

Placeholders are denoted by `{placeholder_name}` in the URL, headers, or body of the request. They are resolved at runtime based on the defined dependencies.

## Example

Given the `requests.toml` and `my-api-environment.toml` files above, when you run the application, it will:

1. **Get Token Request:**
   - Sends a `POST` request to `http://localhost:8000/token` with a JSON body containing your `name` and `email`.
   - If `name` or `email` are missing in `my-api-environment.toml`, the application prompts you to enter them and saves them to the file.

2. **Get Appointments Request:**
   - Sends a `GET` request to `http://localhost:8000/appointments`.
   - Uses the `token` obtained from the `Get Token` request in the `Authorization` header.
