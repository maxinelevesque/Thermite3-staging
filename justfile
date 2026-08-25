# justfile — contributor-facing recipes.
#
# RFC-15 §3.2. `just use <stack>` materializes a contributor's own choice of
# tooling from opt-in/; a contributor who wants none of it runs none of these
# and sees a repository with no agent surface at all.
#
# The gates live behind `make` for now — see issue #27, which tracks moving
# everything here.

# List the stacks and which are installed.
default:
    @dev/use-stack.sh

# Same, spelled out.
list:
    @dev/use-stack.sh

# Materialize a stack: just use claude | crosslink | day
# Refuses when a target exists and differs; pass --force to overwrite.
use stack="" *flags="":
    @dev/use-stack.sh {{stack}} {{flags}}
