<div align="center">
  <h1><code>glint</code></h1>

  <p><b>A local-only, git-friendly scratchpad for testing API endpoints in your terminal. Define, chain, and automate HTTP requests using simple TOML configuration files.</b></p>
</div>

> [!WARNING]
> **This project is in its early stages and may undergo many breaking changes in the future.**

![glint](https://github.com/user-attachments/assets/cc81d961-9a4a-4a0b-8703-6e47cced9762)

## Features

- **Local-Only Design**: Glint keeps everything local. That means your files are yours, perfect for testing without worrying about data going to the cloud.
- **Versioning and Sharing**: Since everything is just regular files, you can easily version and share them with Git. Keep a history of your requests and share them with your team so everyone's on the same page.
- **HTTP Request Collections:** You can put together collections of HTTP requests in a `requests.toml` file.
- **Dynamic Placeholders:** Use placeholders in your URLs, headers, and bodies, and they'll get filled in at runtime.
- **Flexible Dependency Resolution:** Fill in placeholders from lots of different sources:
  - Environment Variables
  - Environment files (TOML)
  - Previous request responses
  - User prompts
  - 1Password vaults
- **Automatic Value Caching:** If glint prompts you for a value, it gets stored in memory so you don't have to type it in again for follow-up requests.
- **Supports All HTTP Methods:** You can use any HTTP method you need in your request definitions.

## Usage

### Running the Application

To run `glint` with an example file, just use this command:

```bash
glint examples/github.toml
```

You can check out some examples in the [examples/](examples/) directory.

### How It Works

- **Placeholder Resolution:**

  - Placeholders like `{name}` and `{email}` get replaced with actual values at runtime.
  - If we can't find a value in your `env_file`, we'll ask you for it and save it for next time.
  - You can also use 1Password credentials to fill in placeholders for extra security.

- **Request Execution:**

  - Requests run one after the other, in the order they're listed in your `requests.toml` file.
  - You can use values from previous responses to fill in placeholders in later requests.

- **Handling Dependencies:**

  - We support a bunch of different dependency sources:
    - `envvar`: Get values from environment variables.
    - `envfile`: Get values from a TOML config file.
    - `request`: Use values from earlier request responses.
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

- **`envvar`**: Get values from environment variables.
  - **`name`**: The name of the environment variable.
  - **`prompt`**: (Optional) What to ask you if the variable isn't defined.
- **`envfile`**: Get values from a TOML config file.
  - **`env_file`**: Path to the environment file.
  - **`key`**: The key to look up in the file.
  - **`prompt`**: (Optional) What to ask you if the key isn't found.
- **`request`**: Get the value from the response to another request.
  - **`request`**: The name of the other request.
  - **`path`**: JSON Pointer to grab the value (e.g., `/token`).
- **`onepassword`**: Get securely stored values from 1Password.
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

And that's it! You can now use `glint` from your terminal.
