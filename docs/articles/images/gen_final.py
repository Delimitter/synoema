#!/usr/bin/env python3
"""Generate cover images via OpenRouter Gemini 2.5 Flash Image. Images are in message.images field."""
import json, urllib.request, base64, os, sys, time

API_KEY = sys.argv[1]
OUT = os.path.dirname(os.path.abspath(__file__))

PROMPTS = {
    "01": "minimalist dark blue tech illustration: golden glowing token coin falling into deep well with quadratic curves radiating outward, orange gold accents, clean vector style, no text, blog cover 1024x768",
    "02": "minimalist tech illustration: magnifying glass examining code split into BPE token pieces, python fragments into tokens on left clean compact symbols on right, blue purple gradient, vector, no text",
    "03": "minimalist tech illustration: chaotic code stream flowing through geometric funnel filter becoming ordered structured output, grammar constraint gates, teal orange, clean vector, no text",
    "04": "minimalist tech illustration: lightning bolt striking CPU chip with binary streams and speed lines, JIT compilation concept, dark background electric blue white, vector, no text",
    "05": "minimalist tech illustration: type inference tree with Greek math symbols at leaves unifying upward into concrete types, Hindley-Milner concept, purple white dark background, vector, no text",
    "06": "minimalist tech illustration: rocket launching from terminal trailing compact code tokens, stars galaxies background, programming language launch, orange deep blue, vector, no text",
    "07": "minimalist tech illustration: crystal prism splitting light into structured code paths AST nodes type constraints, futuristic iridescent colors dark background, vector, no text",
    "08": "minimalist tech illustration: five colorful bar chart columns being measured by digital caliper, token benchmark data concept, scientific feel, white background, vector, no text",
    "09": "minimalist tech illustration: race track with cheetah falcon turtle representing fast medium slow languages, runtime benchmark, motion lines timer milliseconds, green dark background, vector, no text",
    "10": "minimalist tech illustration: grid of AI brain icons producing colored output streams green correct red broken, LLM benchmark matrix, clean tech, vector, no text",
    "11": "minimalist tech illustration: calculator with dollar signs transforming into token symbols, cost reduction arrows, developer team silhouettes growing, green gold, vector, no text",
    "12": "minimalist tech illustration: GPU microchip with radiating circuit traces ending at token symbols, Groq Cerebras chip silhouettes around glowing central token, silicon blue gold, vector, no text",
    "13": "minimalist tech illustration: two parallel streams thin fast and thick slow merging through grammar filter gate, speculative decoding speed 2x 3x effect, electric blue orange, vector, no text",
}

def gen(aid, prompt):
    url = "https://openrouter.ai/api/v1/chat/completions"
    headers = {"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}
    body = json.dumps({
        "model": "google/gemini-2.5-flash-image",
        "messages": [{"role": "user", "content": f"Create this image: {prompt}"}],
    }).encode()
    req = urllib.request.Request(url, data=body, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=180)
        data = json.loads(resp.read())
        msg = data.get("choices", [{}])[0].get("message", {})
        images = msg.get("images", [])
        if not images:
            print(f"  [{aid}] No images field. Keys: {list(msg.keys())}")
            return None
        img_entry = images[0]
        img_url = img_entry.get("image_url", {}).get("url", "")
        if img_url.startswith("data:"):
            b64 = img_url.split(",", 1)[1]
            img_bytes = base64.b64decode(b64)
            ext = "png" if "png" in img_url else "jpg"
            path = os.path.join(OUT, f"cover_{aid}.{ext}")
            with open(path, "wb") as f:
                f.write(img_bytes)
            print(f"  [{aid}] OK — {len(img_bytes)} bytes → {path}")
            return path
        else:
            print(f"  [{aid}] Unexpected URL format: {img_url[:80]}")
            return None
    except urllib.error.HTTPError as e:
        body = e.read().decode()[:200] if e.fp else ""
        print(f"  [{aid}] HTTP {e.code}: {body}")
        return None
    except Exception as e:
        print(f"  [{aid}] Error: {e}")
        return None

if __name__ == "__main__":
    print(f"Generating {len(PROMPTS)} covers via Gemini 2.5 Flash Image...")
    results = []
    for aid in sorted(PROMPTS.keys()):
        path = gen(aid, PROMPTS[aid])
        results.append((aid, path))
        time.sleep(2)

    print(f"\nDone: {sum(1 for _,p in results if p)}/{len(results)} generated")
    for aid, path in results:
        print(f"  [{aid}] {'OK' if path else 'FAIL'}")
