# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/tui`

# Notes from Owner (Readonly Section)
- Try to follow a similar folder architecture to the GUI or CLI project as much as possible.
- This means not bloating the shit out of any file and overloading it with responsibilities.

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks. If no tasks, audit the GUI project against the TUI and look for gaps in functionality. Note that many of the mouse or drag heavy functionality are not really the primary UX, so some UX judgement calls are required).
- From owner: The current state of the TUI is essentially unacceptable. The whole point of ratatui is that you can get insanely good visuals. This means the panels should actually look like panels. There can actually be a theme file. The panels can be much larger and have real UIs behind them. You can absolutely display icons for processes (probably?).
    - This means there can be the concept of entries
    - This means you arent being aggressive enough with folder structure. views/* is the wrong place to dump panes. Sub folders.
    - Again, use the fucking GUI as reference. The TUI is meant to be seriously robust.
    - The panels dont have to look like shit. You can use squares/rectangle shapes with their own background colors. You can make it follow a nice layout. It doesnt all have to look like windows form groupboxes. This is ugly.
    - Also the CLI has a pretty reasonable pattern for command disaptching
^ You can break these up into subtasks, but do not lose the spirit at all of what I am asking,

- Also, the current UX is fucking mind numbing.
    - Process selector: Scrolling through processes 1 by 1 is actual lunacy. Showing only 1 item at any given time sucks, even a ring menu of 3 things would be better.
    - Struct viewer is actually bad UX for a TUI. this is a GUI only thing. Special handling is required for editing project / scan results. Kill this struct viewer.
    - Output view should show some output? its being crushed UI wise.
    - Settings does not need to be visible. Hotkey to pop into settings, it should be full screen. This does not need dedicated real estate.
    - Honestly same with scanners. Full screen takeover to element scanner + results.
    - (Output visible on all pages)
    - So 3 full screen pages thus far:
        - Project explorer + output
        - Element scanner + scan results + output
        - Settings + output

## Important Information
Append important discoveries. Compact regularly ( > ~40 lines, compact to 20 lines)

- 