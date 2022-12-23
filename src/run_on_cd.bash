
# Define the json_env_hook function
function json_env_hook() {
    # Check if the current directory contains a file called ".env.json"
    if [ -f ".env.json" ]; then
        # Source the output of executing the "json_env" program with the "--export" parameter
        eval "$(json_env --export)"
    fi
}

# Run the json_env_hook function before displaying the prompt
function cd () { builtin cd "$@" && json_env_hook; }