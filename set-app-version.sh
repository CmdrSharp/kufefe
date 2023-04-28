#!/bin/bash

if [ $# -eq 0 ]; then
  read -p "Enter the new version (in semver format): " new_version
else
  new_version=$1
fi

# Validate the input against semver standard
if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid semver format. Please enter a version in the format of 'x.y.z'"
  exit 1
fi

# Update the version in the Cargo.toml file
sed -i "s/^version = \".*\"$/version = \"$new_version\"/" Cargo.toml
sed -i "s/^appVersion: \".*\"$/appVersion: \"$new_version\"/" charts/kufefe/Chart.yaml
sed -i "s/^version: .*$/version: $new_version/" charts/kufefe/Chart.yaml

echo "Version updated to $new_version"
