# Projectwise v3.3.0 Demo
# All project names and data are fictional

Set Shell "bash"
Set Theme "Catppuccin Mocha"
Set FontSize 15
Set Width 1300
Set Height 700
Set Padding 18
Set TypingSpeed 35ms
Set PlaybackSpeed 0.7

Output docs/v3.3-demo.gif

# --- Setup: export PATH and demo registry ---
Hide
Type "export PATH=/home/typhoon/.local/bin:$PATH"
Enter
Type "export CLAUDE_PROJECTS_DIR=/tmp/cpm-demo-projects"
Enter
Sleep 0.5s
Type "clear"
Enter
Sleep 0.5s
Show

# --- Scene 1: Version ---
Type "cpm version"
Enter
Sleep 2s

# --- Scene 2: Registry list ---
Type "cpm registry list"
Enter
Sleep 2.5s

# --- Scene 3: Preview a favorite project ---
Type "cpm preview weather-api"
Enter
Sleep 3.5s

# --- Scene 4: Preview trading project ---
Type "cpm preview trading-bot"
Enter
Sleep 3.5s

# --- Scene 5: Shell integration ---
Type "cpm shell-init"
Enter
Sleep 4s

# --- Scene 6: Integrity check ---
Type "cpm integrity check"
Enter
Sleep 2.5s

# --- Scene 7: Cleanup report ---
Type "cpm cleanup report"
Enter
Sleep 3s

Sleep 1s
