CUR_VERSION=$(sed -En 's/^version = "([0-9]+\.[0-9]+\.[0-9]+)"/\1/p' Cargo.toml)
test "v${CUR_VERSION}" == $1
