#!/bin/sh
# cmux-app launcher — auto-detects display backend for GTK4 GL compatibility.
#
# Two NVIDIA-proprietary-driver pitfalls this works around:
#   1. GTK4 may pick the Wayland/EGL backend even in X11 sessions when Wayland
#      libraries are present. Force GDK_BACKEND=x11 under X11 sessions.
#   2. On X11, GDK4 defaults to EGL for GtkGLArea, and EGL-on-X11 fails to
#      create a GL context with the NVIDIA proprietary driver ("Unable to
#      create a GL context") even though GLX works fine (full GL 4.6). Prefer
#      GLX via GDK_DEBUG=gl-glx when an NVIDIA GPU is present.

if [ -z "$GDK_BACKEND" ]; then
    case "${XDG_SESSION_TYPE}" in
        x11)
            export GDK_BACKEND=x11
            ;;
        wayland)
            # Check for NVIDIA proprietary driver — EGL often fails
            if command -v nvidia-smi >/dev/null 2>&1; then
                export GDK_BACKEND=x11
            fi
            ;;
    esac
fi

# Prefer GLX over EGL for GtkGLArea on NVIDIA. Append so any user-set
# GDK_DEBUG is preserved, and stay idempotent if gl-glx is already present.
if command -v nvidia-smi >/dev/null 2>&1 || [ -e /dev/nvidia0 ]; then
    case ",${GDK_DEBUG}," in
        *,gl-glx,*) : ;;
        *) export GDK_DEBUG="${GDK_DEBUG:+$GDK_DEBUG,}gl-glx" ;;
    esac
fi

exec /usr/bin/cmux-app.bin "$@"
