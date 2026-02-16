# Agentic Current Task (Readonly)
Our current task, from `README.md`, is:
`pr/TODO`

### Architecture Plan
Modify sparingly as new information is learned. Keep minimal and simple.
The goal is to keep the architecture in mind and not drift into minefields.

----------------------

## Current Tasklist (ordered)
(Remove as completed, add remaining concrete tasks.)
- First, audit the struct viewer to come up with a plan and add/revise the list below pending the audit:
- Get the GUI struct viewer functional. The idea is that we should be able to click on a scan result, sync it to the struct viewer, and have values displayed for all selected items.
- The columns should have a data_value_box on the right side with editable data.
- Upon committing a value (possibly a separate commit button similar to scan results), whoever sent the last struct should probably get a callback indicating field edit
- Also, currently it appears multi-select is broken for scan results. The struct viewer appears to eliminate a few fields! The whole point was to always show fields in common between selected items, yet somehow some are getting discarded during the set intersection logic.

## Important Information
Append important discoveries. Compact regularly.

Information found in initial audit:
- 

Information discovered during iteration:
- 

## Agent Scratchpad and Notes
Append below and compact regularly to relevant recent notes, keep under ~20 lines.

- 

### Concise Session Log
Append logs for each session here. Compact redundancy occasionally.
- 
