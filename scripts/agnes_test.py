import json
import os
import urllib.request

url = "https://apihub.agnes-ai.com/v1/images/generations"
key = os.environ["AGNES_API_KEY"]
payload = {
    "model": "agnes-image-2.5-flash",
    "prompt": "A 176x220 pixel-art style mobile game main menu screen for a treasure island pirate adventure. Cream off-white background, brown title bar at top with a gold coin emblem centered, 6 menu items in a vertical list on the left (New Game / Load / Options / Help / About / Quit), green highlight bar on the selected row, a small character portrait on the right side. Minimalist flat vector UI, clean readable layout.",
    "size": "2K",
    "ratio": "9:16",
    "extra_body": {"response_format": "url"},
}
req = urllib.request.Request(
    url,
    data=json.dumps(payload).encode(),
    headers={"Content-Type": "application/json", "Authorization": f"Bearer {key}"},
    method="POST",
)
try:
    with urllib.request.urlopen(req, timeout=180) as resp:
        body = resp.read().decode()
        print("STATUS: OK")
        print(body[:2000])
except Exception as e:
    import urllib.error
    code = getattr(e, "code", "?")
    try:
        err = e.read().decode()
    except Exception:
        err = str(e)
    print(f"STATUS: ERR {code}")
    print(err[:1500])
