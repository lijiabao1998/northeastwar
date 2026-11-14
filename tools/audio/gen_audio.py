#!/usr/bin/env python3
# Copyright (c) 2026 李家寶. All rights reserved.
# Original synthesized audio. No sampled recordings.
"""Write the short original clips the slice loops and triggers."""

import math
import random
import struct
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "audio"
RATE = 22050


def save(name: str, samples: list[float]) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / name
    with wave.open(str(path), "w") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(RATE)
        frames = bytearray()
        for sample in samples:
            value = max(-1.0, min(1.0, sample))
            frames += struct.pack("<h", int(value * 32767))
        handle.writeframes(frames)
    print(path, len(samples))


def wind() -> None:
    rng = random.Random(2031)
    samples = []
    state = 0.0
    seconds = 4.0
    n = int(RATE * seconds)
    for i in range(n):
        state = state * 0.98 + (rng.random() * 2 - 1) * 0.02
        gust = 0.55 + 0.45 * math.sin(i / RATE * 0.7) * math.sin(i / RATE * 0.17)
        # Fade the join so the loop does not click.
        edge = min(i / 800.0, (n - i) / 800.0, 1.0)
        samples.append(state * gust * 0.85 * edge)
    save("wind.wav", samples)


def crunch() -> None:
    rng = random.Random(17)
    n = int(RATE * 0.16)
    samples = []
    for i in range(n):
        env = math.exp(-i / (RATE * 0.04))
        noise = rng.random() * 2 - 1
        samples.append(noise * env * 0.7)
    save("crunch.wav", samples)


def phone() -> None:
    samples = []
    n = int(RATE * 0.7)
    for i in range(n):
        t = i / RATE
        pulse = 1.0 if (t % 0.22) < 0.09 else 0.0
        tone = math.sin(2 * math.pi * 480 * t) * 0.5 + math.sin(2 * math.pi * 620 * t) * 0.5
        samples.append(tone * pulse * 0.35)
    save("phone.wav", samples)


def boom() -> None:
    samples = []
    n = int(RATE * 2.4)
    for i in range(n):
        t = i / RATE
        env = math.exp(-t * 1.6)
        freq = 70.0 - t * 18.0
        body = math.sin(2 * math.pi * freq * t)
        grit = math.sin(2 * math.pi * 140 * t) * math.exp(-t * 6.0)
        samples.append((body * 0.8 + grit * 0.25) * env * 0.9)
    save("boom.wav", samples)


if __name__ == "__main__":
    wind()
    crunch()
    phone()
    boom()
