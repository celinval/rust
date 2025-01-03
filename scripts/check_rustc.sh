#!/bin/bash
# Runs the Rust bootstrap script in order to ensure the changes to this repo
# are compliant with the Rust repository tests.
#
# TODO: Need to enable full tidy run.

set -eu

usage() {
    echo "Usage: $0 [--help] [--bless]"
    echo "Options:"
    echo "  -h, --help      Show this help message"
    echo "  --bless         Update library files using tidy"
}

TIDY_MODE=""
# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        --bless)
            TIDY_MODE="--bless"
            shift 1
            ;;
        *)
            echo "Error: Unknown option `$1`"
            usage
            exit 1
            ;;
    esac
done

# Set the working directory for your local repository
REPO_DIR=$(git rev-parse --show-toplevel)

# Temporary directory for upstream repository
TMP_RUST_DIR=$(mktemp -d -t "check_rustc_XXXXXX")

# Checkout your local repository
echo "Checking out local repository..."
cd "$REPO_DIR"

# Get the commit ID from rustc --version
echo "Retrieving commit ID..."
COMMIT_ID=$(rustc --version | sed -e "s/.*(\(.*\) .*/\1/")
echo "$COMMIT_ID for rustc is"

# Get the full commit ID for shallow clone
curl -H "Connection: close" -o "${TMP_RUST_DIR}/output.json" -s --show-error \
    "https://api.github.com/repos/rust-lang/rust/commits?sha=${COMMIT_ID}&per_page=1"

COMMIT_ID=$(cat "${TMP_RUST_DIR}/output.json" | jq -r '.[0].sha')

# Clone the rust-lang/rust repository
echo "Cloning rust-lang/rust repository into ${TMP_RUST_DIR}..."
pushd "$TMP_RUST_DIR" > /dev/null
git init
git remote add origin https://github.com/rust-lang/rust.git
git fetch --depth 1 origin $COMMIT_ID

echo "Checking out commit $COMMIT_ID..."
git checkout "$COMMIT_ID"
git submodule update --init --depth 1
popd

# Copy your library to the upstream directory
echo "Copying library to upstream directory..."
rm -rf "${TMP_RUST_DIR}/library"
cp -r "${REPO_DIR}/library" "${TMP_RUST_DIR}"

# Configure repository
pushd "${TMP_RUST_DIR}"
./configure --set=llvm.download-ci-llvm=true
export RUSTFLAGS="--check-cfg cfg(kani) --check-cfg cfg(feature,values(any()))"

# Run tidy
if [ "${TIDY_MODE}" == "--bless" ];
then
    echo "Run rustfmt"
    # TODO: This should be:
    # ./x test tidy --bless
    ./x fmt
    cp -r "${TMP_RUST_DIR}/library" "${REPO_DIR}"
else
    # TODO: This should be:
    # ./x test tidy
    echo "Check format"
    if ! ./x fmt --check; then
        echo "Format check failed. Run $0 --bless to fix the failures."
        # Clean up the temporary directory
        popd
        rm -rf "$TMP_RUST_DIR"
        exit 1
    fi
fi

# Run tests
cd "$TMP_RUST_DIR"
echo "Running tests..."
./x test --stage 0 library/std

echo "Tests completed."

# Clean up the temporary directory
rm -rf "$TMP_RUST_DIR"
