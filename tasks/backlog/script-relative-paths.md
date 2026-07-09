# Script-Relative Path Resolution for `include` and `@file`

Both `include "path"` and `@"path"` resolve against the process CWD, not the containing script's directory. This means a workflow script at `workflows/release/release.ash` must spell out `workflows/release/prepare.ash` — it leaks the parent chain into the include path.

From a workflow perspective, a script should contain no information beyond its parent folder. Includes and prompt files within the same workflow directory should resolve relative to the script, e.g. `include "prepare.ash"` or `include "./prepare.ash"`.

This requires threading the script's source path through the evaluator so `eval_include` and `eval_fp` can join it against the provided relative path.
