#!/usr/bin/env bash
# Compilation Android sur ce PC (mêmes étapes que la CI, .github/workflows/ci.yml), pour essayer
# sur l'émulateur avant de pousser. Android Studio (SDK, NDK, émulateur « MusicPlayer_API34 »).
#
#   scripts/android-local.sh                  APK x86_64 (émulateur)
#   scripts/android-local.sh aarch64          APK pour un vrai téléphone (cible Rust à ajouter :
#                                             rustup target add aarch64-linux-android)
#   scripts/android-local.sh x86_64 --install compile, installe et lance sur l'émulateur ouvert
set -euo pipefail

TARGET="${1:-x86_64}"
INSTALL="${2:-}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# SDK installé par Android Studio (emplacement par défaut), Java fourni avec Android Studio.
SDK_DIR="$(cygpath -w "$LOCALAPPDATA/Android/Sdk")"
# Java 17, comme la CI : celui d'Android Studio (Java 25) est trop récent pour le Gradle généré
# par Tauri (« Unsupported class file major version 69 »).
if [ -x /c/Android/jdk-17/bin/java.exe ]; then
  export JAVA_HOME='C:\Android\jdk-17'
else
  export JAVA_HOME='C:\Program Files\Android\Android Studio\jbr'
fi
export ANDROID_HOME="$SDK_DIR"
NDK_VERSION="$(ls "$LOCALAPPDATA/Android/Sdk/ndk" 2>/dev/null | sort -V | tail -n 1)"
[ -n "$NDK_VERSION" ] || { echo "NDK absent : Android Studio → SDK Manager → SDK Tools → NDK (Side by side)." >&2; exit 1; }
export NDK_HOME="$(cygpath -w "$LOCALAPPDATA/Android/Sdk/ndk/$NDK_VERSION")"
export PATH="$(cygpath -u "$JAVA_HOME")/bin:$LOCALAPPDATA/Android/Sdk/platform-tools:$PATH"

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
  # APK de version non signé : signé avec une clé de débogage locale (émulateur seulement ; les
  # versions publiées gardent leur clé, dans la CI).
  KS="$HOME/.android/debug.keystore"
  [ -f "$KS" ] || keytool -genkeypair -keystore "$(cygpath -w "$KS")" -storepass android -keypass android     -alias androiddebugkey -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Android Debug,O=Android,C=US"
  BT="$(ls -d "$LOCALAPPDATA/Android/Sdk/build-tools/"* | sort -V | tail -n 1)"
  SIGNED="$TEMP/music-player-local.apk"
  "$BT/apksigner.bat" sign --ks "$(cygpath -w "$KS")" --ks-pass pass:android --out "$(cygpath -w "$SIGNED")" "$(cygpath -w "$APK")"
  APK="$SIGNED"
  adb install -r "$APK"
  adb shell monkey -p dev.mahouna.musicplayer -c android.intent.category.LAUNCHER 1 > /dev/null
fi
