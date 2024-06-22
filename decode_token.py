from pathlib import Path
from base64 import b64decode
from sys import argv
from json import loads, dumps

if __name__ == "__main__":
    path = Path.home().joinpath("AppData", "Local", "tupy", f"spotify.{argv[1]}.token")
    if path.exists():
        with path.open("r") as f:
            print(dumps(loads(b64decode(f.read())), indent=2))
