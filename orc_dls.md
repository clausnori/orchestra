"=== Orchestra Commands ===

DIRECTORY:
OPEN DIR "path"          - view directory structure
CREATE DIR "path"        - create new directory

FILE:
OPEN FILE "path"         - view file content
CREATE FILE "path"       - create new file
DELETE FILE "path" LINE n - delete line from file

EDIT:
EDIT FILE "path" LINE n PUT "text"     - replace line n
INSERT FILE "path" LINE n INSERT "text" - insert at line n (shifts down)

EXECUTION:
RUN "command"            - execute shell command

META:
COMMENTS "text"          - document current action
CALLBACK "message"       - report completion/next step
  