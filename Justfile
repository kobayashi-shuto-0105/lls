git_revision := `git rev-parse --short HEAD`
app_version := `awk -F'"' '/^\[package\]/{p=1} p && /^version *=/{print $2; exit}' Cargo.toml`
build_date := `date -u +%Y-%m-%dT%H:%M:%SZ`

image_name := "ghcr.io/kobayashi-shuto-0105/lls"

build:
    cargo clippy -- -D warnings
    cargo build --release

test:
    cargo test --all-targets --all-features

container-local:
    docker build \
        --build-arg GIT_REVISION={{git_revision}} \
        --build-arg BUILD_DATE={{build_date}} \
        --build-arg VERSION={{app_version}} \
        -t lls:dev \
        -t {{image_name}}:latest \
        -t {{image_name}}:{{app_version}} \
        -f Containerfile \
        .

container:
    docker buildx build --push \
        --platform linux/amd64,linux/arm64 \
        --build-arg GIT_REVISION={{git_revision}} \
        --build-arg BUILD_DATE={{build_date}} \
        --build-arg VERSION={{app_version}} \
        -t {{image_name}}:latest \
        -t {{image_name}}:{{app_version}} \
        -f Containerfile \
        .

container-smoke:
    docker run --rm lls:dev --help
