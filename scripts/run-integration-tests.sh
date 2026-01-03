#!/bin/bash
#
# Run integration tests in Docker
#
# Authentication (choose one):
#   Option 1: Set ANTHROPIC_API_KEY environment variable
#   Option 2: Set CLAUDE_CODE_OAUTH_TOKEN environment variable
#             (Generate with: claude setup-token)
#   Option 3: Create .env file with either variable (see .env.example)
#
# Usage:
#   ./scripts/run-integration-tests.sh                    # Run all tests
#   ./scripts/run-integration-tests.sh test_oneshot       # Run specific test
#   ./scripts/run-integration-tests.sh --shell            # Interactive shell
#   ./scripts/run-integration-tests.sh --verbose          # Verbose output

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$PROJECT_DIR/.env"

# Load .env file if it exists
if [ -f "$ENV_FILE" ]; then
    echo "Loading credentials from .env file..."
    set -a
    source "$ENV_FILE"
    set +a
fi

# Check for authentication
if [ -z "$ANTHROPIC_API_KEY" ] && [ -z "$CLAUDE_CODE_OAUTH_TOKEN" ]; then
    echo "Error: Authentication required"
    echo ""
    echo "Choose one of the following options:"
    echo ""
    echo "Option 1: Anthropic API Key"
    echo "  export ANTHROPIC_API_KEY=sk-ant-xxxxx"
    echo "  $0"
    echo ""
    echo "Option 2: Claude Code OAuth Token (Pro/Max users)"
    echo "  Run 'claude setup-token' to generate a token, then:"
    echo "  export CLAUDE_CODE_OAUTH_TOKEN=your-token"
    echo "  $0"
    echo ""
    echo "Option 3: Create .env file"
    echo "  cp .env.example .env"
    echo "  # Edit .env with your credentials"
    echo "  $0"
    echo ""
    exit 1
fi

# Show which auth method is being used
if [ -n "$ANTHROPIC_API_KEY" ]; then
    echo "Using: Anthropic API Key"
elif [ -n "$CLAUDE_CODE_OAUTH_TOKEN" ]; then
    echo "Using: Claude Code OAuth Token"
fi

# Parse arguments
TEST_FILTER=""
VERBOSE=""
SHELL_MODE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --shell|-s)
            SHELL_MODE=1
            shift
            ;;
        --verbose|-v)
            VERBOSE="--nocapture"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options] [test_filter]"
            echo ""
            echo "Options:"
            echo "  --shell, -s     Start interactive shell instead of running tests"
            echo "  --verbose, -v   Show test output (--nocapture)"
            echo "  --help, -h      Show this help message"
            echo ""
            echo "Authentication:"
            echo "  Set ANTHROPIC_API_KEY or CLAUDE_CODE_OAUTH_TOKEN environment variable,"
            echo "  or create a .env file (see .env.example)"
            echo ""
            echo "Examples:"
            echo "  $0                          Run all integration tests"
            echo "  $0 test_oneshot             Run tests matching 'test_oneshot'"
            echo "  $0 --verbose test_callback  Run callback tests with output"
            echo "  $0 --shell                  Start shell for debugging"
            exit 0
            ;;
        *)
            TEST_FILTER="$1"
            shift
            ;;
    esac
done

# Build environment args
ENV_ARGS=""
if [ -n "$ANTHROPIC_API_KEY" ]; then
    ENV_ARGS="$ENV_ARGS -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY"
fi
if [ -n "$CLAUDE_CODE_OAUTH_TOKEN" ]; then
    ENV_ARGS="$ENV_ARGS -e CLAUDE_CODE_OAUTH_TOKEN=$CLAUDE_CODE_OAUTH_TOKEN"
fi

# Build the Docker image
echo "Building Docker image..."
docker build -f "$PROJECT_DIR/Dockerfile.integration-tests" -t claude-sdk-tests "$PROJECT_DIR"

# Run tests
if [ -n "$SHELL_MODE" ]; then
    echo "Starting interactive shell..."
    docker run --rm -it \
        $ENV_ARGS \
        -e RUST_BACKTRACE=1 \
        claude-sdk-tests \
        bash
elif [ -n "$TEST_FILTER" ]; then
    echo "Running tests matching: $TEST_FILTER"
    docker run --rm \
        $ENV_ARGS \
        -e RUST_BACKTRACE=1 \
        claude-sdk-tests \
        cargo test --features integration-tests -- "$TEST_FILTER" --test-threads=1 $VERBOSE
else
    echo "Running all integration tests..."
    docker run --rm \
        $ENV_ARGS \
        -e RUST_BACKTRACE=1 \
        claude-sdk-tests \
        cargo test --features integration-tests -- --test-threads=1 $VERBOSE
fi
