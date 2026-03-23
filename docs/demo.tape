# Projectwise Demo — VHS recording
# All project names are fictional demo data

Set Shell "bash"
Set Theme "Catppuccin Mocha"
Set FontSize 16
Set Width 1200
Set Height 700
Set Padding 20
Set TypingSpeed 40ms
Set PlaybackSpeed 0.5

Output docs/demo.gif

# Setup: use fake demo registry
Env CLAUDE_PROJECTS_DIR "/tmp/cpm-demo-projects"

# --- Scene 1: Version ---
Type "cpm version"
Enter
Sleep 1.5s

# --- Scene 2: Registry list ---
Type "cpm registry list"
Enter
Sleep 2s

# --- Scene 3: Preview a project ---
Type "cpm preview weather-api"
Enter
Sleep 3s

# --- Scene 4: Preview another (favorite) ---
Type "cpm preview blog-engine"
Enter
Sleep 3s

# --- Scene 5: Project info (JSON) ---
Type "cpm info ml-pipeline"
Enter
Sleep 3s

# --- Scene 6: Integrity check ---
Type "cpm integrity check"
Enter
Sleep 2.5s

# --- Scene 7: FZF selector ---
Type "cpm select"
Sleep 2s
# Show the picker briefly, then exit
Type "weather"
Sleep 1.5s
# Press Escape to cancel
Escape
Sleep 1s

# --- Scene 8: Cleanup report ---
Type "cpm cleanup report"
Enter
Sleep 2.5s

# Done
Sleep 1s
