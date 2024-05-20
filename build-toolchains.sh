#!/usr/bin/env bash
set -euo pipefail

for cmd in wget; do
  if ! command -v $cmd &> /dev/null; then
    echo "${cmd} not found"
    exit 1
  fi
done

project_root="$(dirname "$(realpath "$0")")"

toolchains_dir="${project_root}/.toolchains"

avr_toolchain_dir="${toolchains_dir}/avr"

crosstool_prefix_dir="${toolchains_dir}/crosstool-ng"
crosstool_version="crosstool-ng-1.26.0"
crosstool_source_dir="${toolchains_dir}/crosstool-ng-${crosstool_version}"
crosstool_git_url="git@github.com:crosstool-ng/crosstool-ng.git"
#crosstool_tarball="crosstool-ng-${crosstool_version}.tar.xz"
#crosstool_tarball_url="http://crosstool-ng.org/download/crosstool-ng/${crosstool_tarball}"

redownload_crosstool="false"
rebuild_crosstool="false"
rebuild_avr="false"
rebuild_rpi="false"
while [ "$#" -gt 0 ]; do
  case "$1" in
  --redownload)
    redownload_crosstool="true"
    rebuild_crosstool="true"
    rebuild_avr="true"
    rebuild_rpi="true"
    shift 1
    ;;
  --rebuild)
    rebuild_crosstool="true"
    rebuild_avr="true"
    rebuild_rpi="true"
    shift 1
    ;;
  --rebuild-avr)
    rebuild_avr="true"
    shift 1
    ;;
  --rebuild-rpi)
    rebuild_rpi="true"
    shift 1
    ;;
  -*)
    echo "Error: Unknown option $1"
    exit 1
    ;;
  *)
    break
    ;;
  esac
done

mkdir -p "${toolchains_dir}"

function init_crosstool() {
  if  [ "${redownload_crosstool}" == "true" ] || [ ! -d "${crosstool_source_dir}" ]; then
    echo "Downloading crosstool-ng"
    rm -f "${crosstool_source_dir}"

    git clone -q -- "${crosstool_git_url}" "${crosstool_source_dir}"
    git --git-dir="${crosstool_source_dir}"/.git --work-tree="${crosstool_source_dir}" checkout -q "${crosstool_version}"
  fi

  if [ "${rebuild_crosstool}" == "true" ] || [ ! -d "${crosstool_prefix_dir}" ]; then
    rm -rf "${crosstool_prefix_dir}"

    pushd "${crosstool_source_dir}"

    ./bootstrap

    mkdir -p build
    pushd build

    ../configure --prefix="${crosstool_prefix_dir}"
    make
    make install

    popd
    popd
  fi
}

function init_toolchain() {
  rebuild="${1}"
  toolchain_name="${2}"

  if [ "${rebuild}" == "false" ] && [ -d "${toolchains_dir}/${toolchain_name}" ]; then
    return 0
  fi

  rm -rf "${toolchains_dir}/${toolchain_name:?}"
  rm -rf "${toolchains_dir}/${toolchain_name}-build"
  mkdir -p "${toolchains_dir}/${toolchain_name}-build"

  pushd "${toolchains_dir}/${toolchain_name}-build"

  export CT_PREFIX="${toolchains_dir}"
  export PATH="${crosstool_prefix_dir}/bin:$PATH"
  ct-ng "${toolchain_name}"
  ct-ng build

  popd
}

init_crosstool
init_toolchain "${rebuild_avr}" armv7-rpi2-linux-gnueabihf
init_toolchain "${rebuild_avr}" avr
