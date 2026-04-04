#!/usr/bin/env python3
"""Generate cover images for all articles via OpenRouter (Qwen VL image generation)."""

import json
import os
import sys
import base64
import urllib.request
import urllib.error

API_KEY = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("OPENROUTER_KEY", "")
if not API_KEY:
    print("Usage: python3 generate_covers.py <openrouter-key>")
    sys.exit(1)

OUTPUT_DIR = os.path.dirname(os.path.abspath(__file__))

ARTICLES = [
    {
        "id": "01",
        "prompt": "A minimalist tech illustration: a glowing golden token coin falling into a deep dark well, with quadratic curves radiating outward representing attention cost. Dark blue background with orange/gold accents. Clean vector style, no text."
    },
    {
        "id": "02",
        "prompt": "A minimalist tech illustration: a magnifying glass examining binary code patterns, with Python snake logo fragmenting into BPE token pieces on one side, and clean compact symbols on the other side. Blue-purple gradient background. Clean vector style, no text."
    },
    {
        "id": "03",
        "prompt": "A minimalist tech illustration: a funnel or filter constraining a chaotic stream of code symbols into an ordered, structured output. Grammar rules shown as geometric gates. Teal and orange color scheme. Clean vector style, no text."
    },
    {
        "id": "04",
        "prompt": "A minimalist tech illustration: a lightning bolt striking a CPU chip, with machine code binary streams emanating. Speed lines and a stopwatch showing milliseconds. Dark background with electric blue and white. Clean vector style, no text."
    },
    {
        "id": "05",
        "prompt": "A minimalist tech illustration: a tree diagram showing type inference — Greek letters (alpha, beta, tau) at leaf nodes flowing upward and unifying into concrete types (Int, Bool, List). Mathematical elegance. Purple and white on dark background. Clean vector style, no text."
    },
    {
        "id": "06",
        "prompt": "A minimalist tech illustration: a rocket launching from a terminal/command line interface, trailing a stream of compact code tokens. Stars and galaxies in background suggesting the future of programming. Orange and deep blue. Clean vector style, no text."
    },
    {
        "id": "07",
        "prompt": "A minimalist tech illustration: a crystal ball or prism splitting a beam of light into structured code paths — AST nodes, type constraints, grammar rules. Futuristic, forward-looking feel. Iridescent colors on dark background. Clean vector style, no text."
    },
    {
        "id": "08",
        "prompt": "A minimalist tech illustration: a bar chart with 5 colored columns (representing 5 programming languages) being measured by a precise digital caliper. Token counts as numbers floating above bars. Data-driven, scientific feel. White background with colorful bars. Clean vector style, no text."
    },
    {
        "id": "09",
        "prompt": "A minimalist tech illustration: a race track with three runners — a cheetah (C++), a falcon (Synoema JIT), and a turtle (Python interpreter). Timer showing milliseconds. Dynamic motion lines. Green track on dark background. Clean vector style, no text."
    },
    {
        "id": "10",
        "prompt": "A minimalist tech illustration: multiple AI brain icons (representing different LLM models) looking at the same code sample, each producing different colored output streams — some correct (green), some broken (red). Grid/matrix layout suggesting a benchmark. Clean vector style, no text."
    },
    {
        "id": "11",
        "prompt": "A minimalist tech illustration: a calculator or spreadsheet with dollar signs transforming into token symbols, with arrows showing cost reduction. Team of developer silhouettes of increasing size (5, 25, 100). Green and gold color scheme suggesting savings. Clean vector style, no text."
    },
    {
        "id": "12",
        "prompt": "A minimalist tech illustration: a microchip (GPU/TPU) with radiating circuit traces, each trace ending at a token symbol. Groq, Cerebras, NVIDIA chip silhouettes arranged around a central glowing token. Silicon blue and gold. Clean vector style, no text."
    },
    {
        "id": "13",
        "prompt": "A minimalist tech illustration: two parallel streams merging — a fast thin stream (draft model) and a slow thick stream (target model) — converging through a grammar filter gate. Speed multiplication effect (2x, 3x, 5x). Electric blue and orange. Clean vector style, no text."
    },
]


def generate_image(article):
    """Generate image via OpenRouter using an image generation model."""
    article_id = article["id"]
    prompt = article["prompt"]

    url = "https://openrouter.ai/api/v1/images/generations"
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json",
    }
    body = json.dumps({
        "model": "qwen/qwen2.5-vl-72b-instruct",
        "prompt": prompt,
        "n": 1,
        "size": "1024x1024",
    }).encode()

    req = urllib.request.Request(url, data=body, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=120)
        data = json.loads(resp.read())
        return data
    except urllib.error.HTTPError as e:
        error_body = e.read().decode() if e.fp else ""
        print(f"  [{article_id}] HTTP {e.code}: {error_body[:200]}")
        return None
    except Exception as e:
        print(f"  [{article_id}] Error: {e}")
        return None


def generate_image_chat(article):
    """Generate image via chat completions with image generation request."""
    article_id = article["id"]
    prompt = article["prompt"]

    url = "https://openrouter.ai/api/v1/chat/completions"
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json",
    }
    body = json.dumps({
        "model": "qwen/qwen2.5-vl-72b-instruct",
        "messages": [
            {
                "role": "user",
                "content": f"Generate an image based on this description: {prompt}"
            }
        ],
        "max_tokens": 500,
    }).encode()

    req = urllib.request.Request(url, data=body, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=120)
        data = json.loads(resp.read())
        return data
    except urllib.error.HTTPError as e:
        error_body = e.read().decode() if e.fp else ""
        print(f"  [{article_id}] HTTP {e.code}: {error_body[:200]}")
        return None
    except Exception as e:
        print(f"  [{article_id}] Error: {e}")
        return None


def try_flux_image(article):
    """Try FLUX model for actual image generation via OpenRouter."""
    article_id = article["id"]
    prompt = article["prompt"]

    url = "https://openrouter.ai/api/v1/images/generations"
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json",
    }
    body = json.dumps({
        "model": "black-forest-labs/flux-1.1-pro",
        "prompt": prompt,
        "n": 1,
        "size": "1024x768",
    }).encode()

    req = urllib.request.Request(url, data=body, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=120)
        data = json.loads(resp.read())

        # Handle URL-based response
        if "data" in data and len(data["data"]) > 0:
            img_data = data["data"][0]
            if "url" in img_data:
                # Download the image
                img_url = img_data["url"]
                img_req = urllib.request.urlopen(img_url, timeout=60)
                img_bytes = img_req.read()
                out_path = os.path.join(OUTPUT_DIR, f"cover_{article_id}.png")
                with open(out_path, "wb") as f:
                    f.write(img_bytes)
                print(f"  [{article_id}] Saved {out_path} ({len(img_bytes)} bytes)")
                return out_path
            elif "b64_json" in img_data:
                img_bytes = base64.b64decode(img_data["b64_json"])
                out_path = os.path.join(OUTPUT_DIR, f"cover_{article_id}.png")
                with open(out_path, "wb") as f:
                    f.write(img_bytes)
                print(f"  [{article_id}] Saved {out_path} ({len(img_bytes)} bytes)")
                return out_path

        print(f"  [{article_id}] Unexpected response: {json.dumps(data)[:200]}")
        return None
    except urllib.error.HTTPError as e:
        error_body = e.read().decode() if e.fp else ""
        print(f"  [{article_id}] HTTP {e.code}: {error_body[:300]}")
        return None
    except Exception as e:
        print(f"  [{article_id}] Error: {e}")
        return None


if __name__ == "__main__":
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    print(f"Generating {len(ARTICLES)} cover images...")
    print(f"Output: {OUTPUT_DIR}")
    print()

    results = []
    for article in ARTICLES:
        print(f"  [{article['id']}] Generating...")
        path = try_flux_image(article)
        results.append((article["id"], path))

    print()
    print("Results:")
    for aid, path in results:
        status = f"OK: {path}" if path else "FAILED"
        print(f"  [{aid}] {status}")
