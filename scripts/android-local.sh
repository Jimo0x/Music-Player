#!/usr/bin/env bash
# Compilation Android sur ce PC (mêmes étapes que la CI, .github/workflows/ci.yml), pour essayer
# sur l'émulateur avant de pousser. Outils installés dans C:\Android (JDK 17, SDK, NDK, émulateur).
#
#   scripts/android-local.sh                  APK x86_64 (émulateur)
#   scripts/android-local.sh aarch64          APK pour un vrai téléphone (cible Rust à ajouter :
#                                             rustup target add aarch64-linux-android)
#   scripts/android-local.sh x86_64 --install compile, installe et lance sur l'émulateur ouvert
set -euo pipefail

TARGET="${1:-x86_64}"
INSTALL="${2:-}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

export JAVA_HOME='C:\Android\jdk-17'
export ANDROID_HOME='C:\Android\sdk'
export NDK_HOME='C:\Android\sdk\ndk\30.0.16248370'
export PATH="/c/Android/jdk-17/bin:/c/Android/sdk/platform-tools:$PATH"

cd "$ROOT"
# Projet Android généré une fois (src-tauri/gen/android, hors git), puis complété comme en CI.
if [ ! -d src-tauri/gen/android ]; then
  npx tauri android init --ci
fi
# « tauri icon » réécrit aussi les icônes de src-tauri/icons, suivies par git : remises après.
npx tauri icon src-tauri/icons/app-icon-source.png > /dev/null
git checkout -- src-tauri/icons
node scripts/android-patch.mjs

npx tauri android build --apk --target "$TARGET"
APK="$(find src-tauri/gen/android/app/build/outputs/apk -name '*.apk' -printf '%T@ %p\n' | sort -n | tail -n 1 | cut -d' ' -f2-)"
echo "APK : $APK"

if [ "$INSTALL" = "--install" ]; then
  adb install -r "$APK"
  adb shell monkey -p dev.mahouna.musicplayer -c android.intent.category.LAUNCHER 1 > /dev/null
fi
