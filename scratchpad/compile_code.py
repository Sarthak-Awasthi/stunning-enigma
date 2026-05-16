import os
import sys
from pathlib import Path

OUTPUT_FILE = "llm_codebase_context.md"

# Folders to ignore to save LLM tokens
IGNORED_DIRS = {
    ".git", "target", "build", "node_modules", "debug", "release", 
    ".idea", ".vscode", "vendor", "__pycache__"
}

# Extensions associated with your tech stack + Documentation
ALLOWED_EXTENSIONS = {
    # Rust
    ".rs",
    # C++
    ".cpp", ".hpp", ".h", ".c", ".cc", ".cxx", ".hxx",
    # Kirigami / QML / UI
    ".qml", ".js", ".qrc", ".ui",
    # SQLite DB
    ".sql",
    # Build files
    ".pro", ".pri", ".cmake",
    # Documentation / Markdown
    ".md", ".markdown", ".txt"
}

# Explicit files to include
ALLOWED_FILES = {"CMakeLists.txt", "Cargo.toml"}

def get_markdown_lang(path: Path) -> str:
    if path.name == "CMakeLists.txt": 
        return "cmake"
    if path.name == "Cargo.toml": 
        return "toml"

    ext = path.suffix.lower()
    if ext == ".rs": 
        return "rust"
    if ext in {".cpp", ".hpp", ".h", ".c", ".cc", ".cxx", ".hxx"}:
        return "cpp"
    if ext == ".qml":
        return "qml"
    if ext == ".js": 
        return "javascript"
    if ext == ".sql": 
        return "sql"
    if ext == ".cmake": 
        return "cmake"
    if ext in {".md", ".markdown"}: 
        return "markdown"
    
    return "text"

def is_allowed_file(path: Path) -> bool:
    name = path.name

    # SAFETY: Prevent reading the file we are generating
    if name == OUTPUT_FILE:
        return False

    # Ignore Qt generated C++ files
    if name.startswith("moc_") or name.startswith("qrc_") or name.startswith("ui_"):
        return False

    if name in ALLOWED_FILES:
        return True

    if path.suffix.lower() in ALLOWED_EXTENSIONS:
        return True

    return False

def main():
    # Read target directory from command line arguments, default to current dir
    target_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()

    print(f"Scanning directory: {target_dir}")

    with open(OUTPUT_FILE, "w", encoding="utf-8") as out:
        out.write("# Codebase Context\n\n")
        out.write("> This document contains the compiled source code and documentation for a project utilizing a Rust backend, C++, Kirigami-UI frontend, and an SQLite database.\n\n")

        # os.walk traverses the directory tree
        for root, dirs, files in os.walk(target_dir):
            # Modifying `dirs` in-place tells os.walk to skip ignored directories entirely
            dirs[:] = [d for d in dirs if not d.startswith('.') and d not in IGNORED_DIRS]

            for file in files:
                file_path = Path(root) / file
                
                if is_allowed_file(file_path):
                    # Read file with 'replace' to prevent crashes from binary noise
                    try:
                        with open(file_path, "r", encoding="utf-8", errors="replace") as f:
                            content = f.read().strip()
                    except Exception as e:
                        print(f"Failed to read {file_path}: {e}")
                        continue

                    # Skip empty files
                    if not content:
                        continue

                    lang = get_markdown_lang(file_path)
                    
                    # Try to make paths relative to the target dir so they are shorter
                    try:
                        display_path = file_path.relative_to(target_dir).as_posix()
                    except ValueError:
                        display_path = file_path.as_posix()

                    print(f"Adding: {display_path}")

                    # Write LLM-optimized Markdown formatting using 4 backticks
                    out.write(f"## File: `{display_path}`\n")
                    out.write(f"````{lang}\n")
                    out.write(content + "\n")
                    out.write("````\n\n")

    print(f"\n✅ Successfully compiled codebase to '{OUTPUT_FILE}'")

if __name__ == "__main__":
    main()