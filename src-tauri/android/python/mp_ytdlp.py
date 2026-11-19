# Téléchargement sur téléphone : recherche et téléchargement par yt-dlp, appelés depuis
# Kotlin (YtDlpBridge.kt, Chaquopy). yt-dlp n'est pas figé dans l'APK : c'est la version
# officielle téléchargée par l'appli (fetch/ytdlp_zip.rs), importée depuis son fichier zip.
#
# Pas de ffmpeg sur téléphone : le son est pris tel que YouTube le sert, de préférence en AAC
# (.m4a), que le lecteur de l'appli lit directement. Les étiquettes sont écrites ensuite par
# l'appli (Rust).

import json
import os
import sys
import threading

_cancel = threading.Event()
_progress = -1.0
_loaded_from = None


def _load(zip_path):
    """Importe yt-dlp depuis son fichier (rechargé si le fichier a changé de place)."""
    global _loaded_from
    # Certificats du téléphone, au format attendu par OpenSSL.
    if os.path.isdir("/system/etc/security/cacerts"):
        os.environ.setdefault("SSL_CERT_DIR", "/system/etc/security/cacerts")
    if zip_path and zip_path not in sys.path:
        sys.path.insert(0, zip_path)
    if _loaded_from not in (None, zip_path):
        for name in [n for n in sys.modules if n == "yt_dlp" or n.startswith("yt_dlp.")]:
            del sys.modules[name]
    import yt_dlp

    _loaded_from = zip_path
    return yt_dlp


def version(zip_path):
    return _load(zip_path).version.__version__


def search(zip_path, query):
    """Candidats d'une recherche (« ytsearch5:… »), sans téléchargement : même forme que
    `yt-dlp --flat-playlist --dump-single-json`, lue par parse_candidates côté Rust."""
    yt_dlp = _load(zip_path)
    # « noplaylist » : un lien de son partagé depuis un Mix YouTube (watch?v=…&list=RD…) reste
    # ce son-là, comme --no-playlist sur ordinateur (sinon tout le Mix partait dans la file).
    # Sans effet sur une recherche « ytsearch5: » ni sur un vrai lien de playlist.
    options = {"quiet": True, "no_warnings": True, "extract_flat": "in_playlist", "skip_download": True, "noplaylist": True}
    with yt_dlp.YoutubeDL(options) as ydl:
        info = ydl.extract_info(query, download=False) or {}
    entries = []
    for e in info.get("entries") or []:
        entries.append(
            {
                "title": e.get("title"),
                "channel": e.get("channel") or e.get("uploader"),
                "duration": e.get("duration"),
                "url": e.get("url"),
            }
        )
    # Nom de la playlist ou de l'album (lien lu par fetch_read_media_link), vide pour une recherche.
    return json.dumps({"title": info.get("title") or "", "entries": entries})


def _hook(d):
    global _progress
    if _cancel.is_set():
        raise Exception("Arrêt demandé.")
    if d.get("status") == "downloading":
        total = d.get("total_bytes") or d.get("total_bytes_estimate")
        done = d.get("downloaded_bytes") or 0
        _progress = min(1.0, done / total) if total else -1.0


def download(zip_path, url, out_dir):
    """Télécharge le son du lien dans out_dir ; renvoie le chemin du fichier."""
    global _progress
    yt_dlp = _load(zip_path)
    _cancel.clear()
    _progress = -1.0
    options = {
        # AAC d'abord (lu directement par l'appli), sinon le meilleur son disponible.
        "format": "bestaudio[ext=m4a]/bestaudio[acodec^=mp4a]/bestaudio",
        "outtmpl": os.path.join(out_dir, "%(title)s.%(ext)s"),
        "noplaylist": True,
        "quiet": True,
        "noprogress": True,
        "no_warnings": True,
        "retries": 3,
        "fragment_retries": 3,
        "progress_hooks": [_hook],
    }
    with yt_dlp.YoutubeDL(options) as ydl:
        info = ydl.extract_info(url, download=True) or {}
        downloads = info.get("requested_downloads") or []
        path = (downloads[0].get("filepath") if downloads else None) or ydl.prepare_filename(info)
    _progress = 1.0
    return path


def progress():
    return _progress


def cancel():
    _cancel.set()
