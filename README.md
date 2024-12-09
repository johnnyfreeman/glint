<div align="center">
  <h1><code>glint</code></h1>

  <p><b>A local-only, git-friendly scratchpad for testing API endpoints in your terminal. Define, chain, and automate HTTP requests using simple TOML configuration files.</b></p>
</div>

> [!WARNING]
> **This project is in its early stages and may undergo many breaking changes in the future.**

![glint](https://github.com/user-attachments/assets/a6663c10-64a0-4e6f-ae4e-a3ac85fd2291)

## Features

- [x] **Local-Only Design**: Glint keeps everything local. That means your files are yours, perfect for testing without worrying about data going to the cloud.
- [x] **Versioning and Sharing**: Since everything is just regular files, you can easily version and share them with Git. Keep a history of your requests and share them with your team so everyone's on the same page.
- [x] **HTTP Request Collections:** You can put together collections of HTTP requests in a `collection-name-here.toml` file.
- [x] **Dynamic Placeholders:** Use placeholders in your URLs, headers, and bodies, and they'll get filled in at runtime using Dependency Resolution.
- [x] **Flexible Dependency Resolution:** Fill in placeholders from lots of different sources:
  - [x] Environment Variables
  - [x] Environment files (TOML)
  - [x] Previous request responses
  - [x] User prompts
  - [x] 1Password vaults
- [x] **Pre-Output Masking:** Automatically mask or redact sensitive fields in output using JSON Path and regex.
- [ ] **Encrypted Credential Storage:** A vault for storing your credentials in a secure manner.

## Usage

### Running the Application

To run `glint` with an example file, just use this command:

```bash
glint examples/github.toml
```

You can check out some examples in the [examples/](examples/) directory.

### Running with Docker

To run `glint` using Docker, use the following command:

```bash
docker run --rm -v $(pwd)/examples:/glint johnnyfreeman/glint examples/github.toml
```

This command mounts your local `examples` directory to the container, allowing Glint to access the example request collections.

### How It Works

- **Placeholder Resolution:**

  - Placeholders like `{name}` and `{email}` get replaced with actual values at runtime.
  - If we can't find a value in your `env_file`, we'll ask you for it and save it for next time.
  - You can also use 1Password credentials to fill in placeholders for extra security.

- **Masking Sensitive Data:**

  Glint supports pre-output masking to protect sensitive data in API responses or logs. You can define masking rules using JSON Pointers to specify fields and choose how they are masked.
Example Masking Rule:

```toml
[[masking_rules]]
path = "$.user.ssn"
regex = "\\d{3}-\\d{2}-\\d{4}"
replace = "***-**-****"

[[masking_rules]]
path = "$.user.email"
regex = "(.{2})(.*)(@.*)"
replace = "$1***$3"
```

Example Input:

```json
{
  "user": {
    "name": "John Doe",
    "ssn": "123-45-6789",
    "email": "johndoe@example.com"
  }
}
```

Example Output:

```json
{
  "user": {
    "name": "John Doe",
    "ssn": "***-**-****",
    "email": "jo***@example.com"
  }
}
```

- **Request Execution:**

  - Requests run one after the other, in the order they're listed in your `requests.toml` file.
  - You can use values from previous responses to fill in placeholders in later requests.

- **Handling Dependencies:**

  - We support a bunch of different dependency sources:
    - `envvar`: Get values from environment variables.
    - `envfile`: Get values from a TOML config file.
    - `response`: Use values from earlier request responses.
    - `onepassword`: Grab values from a 1Password vault.

## Configuration

### Defining Requests

Each request is defined under the `[[requests]]` section in your `.toml` request collection file. Here's what you can include:

- **`name`**: A unique name for your request.
- **`method`**: The HTTP method (like `GET`, `POST`, etc.).
- **`url`**: The URL you're hitting, which can have placeholders.
- **`headers`**: Any headers you need to add.
- **`body`**: The request body, which can also have placeholders.
- **`dependencies`**: Dynamic values you need to resolve before sending the request.

### Dependencies

Dependencies tell us how to fill in placeholders. Here's what we support:

- **`EnvVar`**: Get values from environment variables.
  - **`name`**: The name of the environment variable.
  - **`prompt`**: (Optional) What to ask you if the variable isn't defined.
- **`EnvFile`**: Get values from a TOML config file.
  - **`env_file`**: Path to the environment file.
  - **`key`**: The key to look up in the file.
  - **`prompt`**: (Optional) What to ask you if the key isn't found.
- **`Response`**: Get the value from the response to another request.
  - **`request`**: The name of the other request.
  - **`target`**: JSON Pointer to grab the value (e.g., `/token`).
    - **`source`**: Either `JsonBody` or `HeaderValue`.
    - **`pointer`**: A JSON Pointer string if targeting `JsonBody`.
    - **`key`**: A header key if targeting `HeaderValue`.
- **`OnePassword`**: Get securely stored values from 1Password.
  - **`vault`**: The name of the vault.
  - **`item`**: The item name or identifier.
  - **`field`**: The specific field to use.

## Installation

1. **Clone the Repo:**

   ```bash
   git clone git@github.com:johnnyfreeman/glint.git
   cd glint
   ```

2. **Build and Install Glint:**

   ```bash
   cargo install --path .
   ```

3. **Using Docker:**

   To build the Docker image, use the following command:

   ```bash
   docker build -t johnnyfreeman/glint .
   ```

   To run Glint using Docker, use:

   ```bash
   docker run --rm -v $(pwd)/examples:/glint johnnyfreeman/glint examples/weather.toml
   ```

And that's it! You can now use `glint` from your terminal or within a Docker container.


