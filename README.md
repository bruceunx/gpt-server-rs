# GPT Server 🚀🤖

A scalable and efficient GPT server built using the actix-web framework in Rust. This server enables seamless integration and management of multiple GPT models such as OpenAI's GPT, Claude, Gemini, and more. It provides robust RESTful APIs for interacting with these models. 🌟

## Features ✨

- Multi-Model Support 🧠
- Easily integrate AI models like OpenAI GPT, Claude, and Gemini.
- Asynchronous API Handling ⚡
- Powered by Actix-Web for high performance and non-blocking operations.
- Customizable Middleware 🛠️
- Add logging, authentication, or rate-limiting as needed.
- Scalable Architecture 📈
- Built for production-ready deployment.
- Error Handling 🚨
- Graceful error reporting for debugging ease.

## Requirements 🧰

- Rust (Latest stable version) 🦀
- Actix-Web (Core framework)
- HTTP Client Library (e.g., reqwest for external API requests)
- API keys for GPT services (OpenAI, Claude, Gemini, etc.).

## Installation 🛠️

1. Clone the repository:

```bash
git clone https://github.com/bruceunx/gpt-server-rs.git
cd gpt-server-rs
```

2. Install dependencies: Ensure you have the Rust toolchain installed. Use cargo to download and compile the dependencies:

```bash
cargo build
```

3. Set up environment variables: Create a .env file or set environment variables for API keys and configurations:

```env
OPENAI_API_KEY=your_openai_api_key
CLAUDE_API_KEY=your_claude_api_key
GEMINI_API_KEY=your_gemini_api_key
SERVER_PORT=8080
```

## Usage 🚀

1. Start the server:

```bash
cargo run
```

2. Access the server at `http://127.0.0.1:8080/v1/openai/chat`. Use tools like curl, Postman, or your favorite HTTP client to interact with the APIs.

## License 📜

This project is licensed under the MIT License.

## Contact 📬

For questions or support, feel free to reach out to 📧 Email:[bruceunx@outlook.com](bruceunx@outlook.com).
