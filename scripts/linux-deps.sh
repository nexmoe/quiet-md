#!/usr/bin/env bash
set -euo pipefail
[[ $(uname -s) == Linux ]] || { echo 'Run this script inside Linux.' >&2; exit 1; }
source /etc/os-release
case " $ID ${ID_LIKE:-} " in
  *' arch '*)
    # Software Vulkan fallback for VMs or systems without a usable hardware driver.
    sudo pacman -S --needed imagemagick base-devel clang cmake pkgconf fontconfig freetype2 libxcb libxkbcommon libxkbcommon-x11 wayland openssl vulkan-icd-loader vulkan-tools vulkan-swrast mesa noto-fonts noto-fonts-cjk xdg-desktop-portal xdg-utils
    ;;
  *' ubuntu '*|*' debian '*)
    sudo apt-get update
    sudo apt-get install imagemagick build-essential clang cmake pkg-config libfontconfig1-dev libfreetype-dev libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libssl-dev libx11-xcb-dev libvulkan-dev vulkan-tools mesa-vulkan-drivers fonts-noto-core fonts-noto-cjk xdg-desktop-portal xdg-utils
    ;;
  *) echo "No dependency recipe for $ID yet." >&2; exit 1 ;;
esac
