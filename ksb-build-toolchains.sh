#!/usr/bin/env bash
set -euo pipefail

toolchains_dir="${KSB_PROJECT_ROOT}/.toolchains"

crosstool_prefix_dir="${toolchains_dir}/crosstool-ng"
crosstool_version="crosstool-ng-1.26.0"
crosstool_source_dir="${toolchains_dir}/crosstool-ng-${crosstool_version}"
crosstool_git_url="git@github.com:crosstool-ng/crosstool-ng.git"

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

function shell_title() {
  echo -ne "\033]0;$1\007"
}

function init_crosstool() {
  if  [ "${redownload_crosstool}" == "true" ] || [ ! -d "${crosstool_source_dir}" ]; then
    echo "Downloading crosstool-ng"
    rm -f "${crosstool_source_dir}"

    git clone -q -- "${crosstool_git_url}" "${crosstool_source_dir}"
    git --git-dir="${crosstool_source_dir}"/.git --work-tree="${crosstool_source_dir}" checkout -q "${crosstool_version}"
  fi

  if [ "${rebuild_crosstool}" == "true" ] || [ ! -d "${crosstool_prefix_dir}" ]; then
    shell_title "Building Crosstool"
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
  shell_title ""
}

function use_toolchain_script() {
  toolchain_sysroot="${1}"
  toolchain_script_path="${2}"

# Should we set?
# - CC
# - CXX
# - AR
# - FC
# - LD
  cat << EOF > "${toolchain_script_path}"
#!/usr/bin/env bash
set -euo pipefail

export PATH="${toolchain_sysroot}/bin:\${PATH}"
EOF
  chmod +x "${toolchain_script_path}"
}

function init_toolchain() {
  rebuild="${1}"
  toolchain_name="${2}"

  if [ "${rebuild}" == "false" ] && [ -d "${toolchains_dir}/${toolchain_name}" ]; then
    return 0
  fi

  shell_title "Building ${toolchain_name}"

  toolchain_output_dir="${toolchains_dir}/${toolchain_name:?}"
  toolchain_build_dir="${toolchains_dir}/${toolchain_name}-build"

# TODO: Can't always remove old dirs?
  rm -rf "${toolchain_output_dir}"
  rm -rf "${toolchain_build_dir}"
  mkdir -p "${toolchain_build_dir}"

  pushd "${toolchain_build_dir}"

  export CT_PREFIX="${toolchains_dir}"
  export PATH="${crosstool_prefix_dir}/bin:$PATH"
  ct-ng "${toolchain_name}"
  ct-ng build

  popd

  use_toolchain_script "${toolchain_output_dir}" "${toolchains_dir}/use-${toolchain_name}.sh"

  shell_title ""
}

init_crosstool
init_toolchain "${rebuild_avr}" armv7-rpi2-linux-gnueabihf
init_toolchain "${rebuild_avr}" avr
