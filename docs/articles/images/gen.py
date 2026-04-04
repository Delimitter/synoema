#!/usr/bin/env python3
"""Generate cover images for articles via OpenRouter Gemini 2.5 Flash Image."""
import json, urllib.request, base64, os, sys, time

API_KEY = sys.argv[1] if len(sys.argv) > 1 else ""
OUT = os.path.dirname(os.path.abspath(__file__))

PROMPTS = {
    "01": "minimalist dark blue illustration: golden glowing token coin falling into deep well with quadratic curves radiating outward, orange gold accents, clean vector style, no text, tech blog cover",
    "02": "minimalist illustration: magnifying glass examining code being split into BPE token pieces, python snake fragmenting into tokens on left side versus compact clean symbols on right, blue purple gradient, vector style, no text",
    "03": "minimalist illustration: chaotic colorful code stream flowing through a geometric funnel filter becoming ordered structured output, teal and orange, clean vector, no text, tech concept",
    "04": "minimalist illustration: lightning bolt striking CPU chip with binary machine code streams, speed lines, dark background electric blue white, compilation speed concept, vector style, no text",
    "05": "minimalist illustration: tree diagram with Greek math symbols at leaves unifying upward into concrete types, mathematical type inference concept, purple white dark background, vector, no text",
    "06": "minimalist illustration: rocket launching from computer terminal trailing compact code tokens, stars galaxies background, orange deep blue, startup launch concept, vector style, no text",
    "07": "minimalist illustration: crystal prism splitting light beam into structured colored code paths showing AST nodes and type constraints, futuristic iridescent colors dark background, vector, no text",
    "08": "minimalist illustration: five colorful bar chart columns being measured by digital caliper, data benchmark concept with floating numbers, white background, scientific feel, vector style, no text",
    "09": "minimalist illustration: race track with cheetah falcon and turtle racing representing fast medium slow programming languages, dynamic motion timer showing milliseconds, green track dark background, vector, no text",
    "10": "minimalist illustration: grid of AI brain icons each producing colored output streams some green correct some red broken, benchmark matrix layout, clean tech, vector style, no text",
    "11": "minimalist illustration: calculator with dollar signs transforming into token symbols, arrows showing cost reduction, team developer silhouettes of increasing size, green gold savings concept, vector, no text",
    "12": "minimalist illustration: microchip GPU with radiating circuit traces each ending at token symbol, multiple chip silhouettes around central glowing token, silicon blue gold, vector style, no text",
    "13": "minimalist illustration: two parallel streams merging through grammar filter gate, thin fast stream and thick slow stream converging, speed multiplication 2x 3x 5x effect, electric blue orange, vector, no text",
}

def gen(article_id, prompt):
    url = "https://openrouter.ai/api/v1/chat/completions"
    headers = {"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}
    body = json.dumps({
        "model": "google/gemini-2.5-flash-image",
        "messages": [{"role": "user", "content": f"Generate this image: {prompt}"}],
        "max_tokens": 4096,
    }).encode()

    req = urllib.request.Request(url, data=body, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=180)
        data = json.loads(resp.read())
        choices = data.get("choices", [])
        if not choices:
            print(f"  [{article_id}] No choices in response")
            return None

        msg = choices[0].get("message", {})
        content = msg.get("content", "")

        # Content may be a list with image parts
        if isinstance(content, list):
            for part in content:
                if isinstance(part, dict):
                    if part.get("type") == "image_url":
                        img_url = part.get("image_url", {}).get("url", "")
                        if img_url.startswith("data:"):
                            # Base64 encoded
                            b64 = img_url.split(",", 1)[1] if "," in img_url else img_url
                            img_bytes = base64.b64decode(b64)
                            ext = "png" if "png" in img_url else "jpg"
                            path = os.path.join(OUT, f"cover_{article_id}.{ext}")
                            with open(path, "wb") as f:
                                f.write(img_bytes)
                            print(f"  [{article_id}] Saved {path} ({len(img_bytes)} bytes)")
                            return path
                        else:
                            # URL - download
                            img_req = urllib.request.urlopen(img_url, timeout=60)
                            img_bytes = img_req.read()
                            path = os.path.join(OUT, f"cover_{article_id}.png")
                            with open(path, "wb") as f:
                                f.write(img_bytes)
                            print(f"  [{article_id}] Saved {path} ({len(img_bytes)} bytes)")
                            return path

        # Check if there's an image in the response metadata
        print(f"  [{article_id}] No image in response. Content type: {type(content).__name__}, len: {len(str(content))}")
        print(f"  [{article_id}] Preview: {str(content)[:150]}")
        return None

    except urllib.error.HTTPError as e:
        body = e.read().decode()[:300] if e.fp else ""
        print(f"  [{article_id}] HTTP {e.code}: {body}")
        return None
    except Exception as e:
        print(f"  [{article_id}] Error: {e}")
        return None


def gen_gpt_image(article_id, prompt):
    """Try GPT-5 Image Mini as fallback."""
    url = "https://openrouter.ai/api/v1/chat/completions"
    headers = {"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}
    body = json.dumps({
        "model": "openai/gpt-5-image-mini",
        "messages": [{"role": "user", "content": f"Generate an image: {prompt}"}],
        "max_tokens": 4096,
    }).encode()

    req = urllib.request.Request(url, data=body, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=180)
        data = json.loads(resp.read())
        choices = data.get("choices", [])
        if not choices:
            return None

        msg = choices[0].get("message", {})
        content = msg.get("content", "")

        if isinstance(content, list):
            for part in content:
                if isinstance(part, dict):
                    iurl = ""
                    if part.get("type") == "image_url":
                        iurl = part.get("image_url", {}).get("url", "")
                    elif "url" in part:
                        iurl = part["url"]

                    if iurl:
                        if iurl.startswith("data:"):
                            b64 = iurl.split(",", 1)[1]
                            img = base64.b64decode(b64)
                        else:
                            img = urllib.request.urlopen(iurl, timeout=60).read()
                        path = os.path.join(OUT, f"cover_{article_id}.png")
                        with open(path, "wb") as f:
                            f.write(img)
                        print(f"  [{article_id}] GPT saved {path} ({len(img)} bytes)")
                        return path

        # Try raw content check
        if isinstance(content, str) and len(content) > 1000:
            # Might be base64
            try:
                img = base64.b64decode(content)
                path = os.path.join(OUT, f"cover_{article_id}.png")
                with open(path, "wb") as f:
                    f.write(img)
                print(f"  [{article_id}] GPT saved (b64 string) {path}")
                return path
            except:
                pass

        print(f"  [{article_id}] GPT: no image. Content: {str(content)[:150]}")
        return None
    except urllib.error.HTTPError as e:
        body = e.read().decode()[:300] if e.fp else ""
        print(f"  [{article_id}] GPT HTTP {e.code}: {body}")
        return None
    except Exception as e:
        print(f"  [{article_id}] GPT Error: {e}")
        return None


if __name__ == "__main__":
    if not API_KEY:
        print("Usage: python3 gen.py <api-key>")
        sys.exit(1)

    print(f"Generating {len(PROMPTS)} covers...")

    # Try first article with both models to see which works
    test_id = "01"
    print(f"\n--- Testing Gemini Flash Image ---")
    r1 = gen(test_id, PROMPTS[test_id])

    if not r1:
        print(f"\n--- Testing GPT-5 Image Mini ---")
        r1 = gen_gpt_image(test_id, PROMPTS[test_id])

    if r1:
        model_fn = gen if "Gemini" in str(r1) else gen_gpt_image
        # Determine which worked
        if os.path.exists(os.path.join(OUT, f"cover_01.png")) or os.path.exists(os.path.join(OUT, f"cover_01.jpg")):
            print(f"\nFirst image worked! Generating remaining...")
            for aid, prompt in PROMPTS.items():
                if aid == "01":
                    continue
                print(f"  [{aid}]...")
                # Try GPT first since it likely worked
                r = gen_gpt_image(aid, prompt)
                if not r:
                    r = gen(aid, prompt)
                time.sleep(1)  # Rate limit
    else:
        print("\nBoth models failed to produce images.")
        print("Will need manual image generation or different approach.")
