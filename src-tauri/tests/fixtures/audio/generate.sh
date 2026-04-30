#!/usr/bin/env bash
# Generate 5 one-second silent audio fixtures covering mp3/flac/m4a/wav + no-tag fallback.
# Usage: cd src-tauri/tests/fixtures/audio && bash generate.sh
set -e
cd "$(dirname "$0")"

ffmpeg -y -f lavfi -i anullsrc=r=44100:cl=stereo -t 1 \
  -metadata title="Track A" -metadata artist="Test Artist 1" \
  -metadata album="Test Album 1" -metadata track=1 \
  a.mp3

ffmpeg -y -f lavfi -i anullsrc=r=44100:cl=stereo -t 1 \
  -metadata title="Track B" -metadata artist="Test Artist 1" \
  -metadata album="Test Album 1" -metadata track=2 \
  b.flac

ffmpeg -y -f lavfi -i anullsrc=r=44100:cl=stereo -t 1 \
  -metadata title="Track C" -metadata artist="Test Artist 2" \
  -metadata album="Test Album 2" \
  c.m4a

ffmpeg -y -f lavfi -i anullsrc=r=44100:cl=stereo -t 1 \
  -metadata title="Track D" -metadata artist="Test Artist 2" \
  -metadata album="Test Album 2" \
  d.wav

ffmpeg -y -f lavfi -i anullsrc=r=44100:cl=stereo -t 1 \
  e_no_tag.mp3

ls -la *.mp3 *.flac *.m4a *.wav
echo "Total size (KB):"
du -sk . | awk '{print $1}'
