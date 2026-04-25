import sys
import os
import re


def main():
    if len(sys.argv) < 2:
        print("Usage: python update_version.py <tag>")
        sys.exit(1)

    tag = sys.argv[1]

    # Strip the 'v' to get the raw version (e.g. 'v0.4.3' -> '0.4.3')
    version = tag.lstrip("v")

    # Create the Windows tuple format (e.g. '0.4.3' -> '0, 4, 3, 0')
    parts = version.split(".")
    while len(parts) < 4:
        parts.append("0")
    version_tuple = ", ".join(parts[:4])

    print(f"Updating versions to Tag: {tag}, Version: {version}")

    # 1. Process version_info.txt.in -> version_info.txt
    if os.path.exists("templates/version_info.txt.in"):
        with open("templates/version_info.txt.in", "r", encoding="utf-8") as f:
            content = f.read()

        content = content.replace("{{VERSION}}", version)
        content = content.replace("{{VERSION_TUPLE}}", version_tuple)

        with open("version_info.txt", "w", encoding="utf-8") as f:
            f.write(content)
        print(" Generated version_info.txt")
    else:
        print(" Warning: version_info.txt.in not found!")

    # 2. In-place update of README.md
    if os.path.exists("templates/README.md.in"):
        with open("templates/README.md.in", "r", encoding="utf-8") as f:
            readme = f.read()

        # Replace Linux/Mac TAG="vx.y.z"
        readme = re.sub(r'TAG="{{VERSION}}"', f'TAG="{tag}"', readme)
        # Replace Windows $TAG="vx.y.z"
        # readme = re.sub(r'\$TAG="{{VERSION}}"', f'$TAG="{tag}"', readme)

        with open("README.md", "w", encoding="utf-8") as f:
            f.write(readme)
        print(" Updated README.md")
    else:
        print(" Warning: README.md not found!")


if __name__ == "__main__":
    main()
