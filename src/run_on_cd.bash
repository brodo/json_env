
# Define the json_env_hook function
function json_env_hook() {
    # Check if the current directory contains a file called ".env.json"
    if [ -f ".env.json" ]; then
        # Execute the "json_env" program with the "--export" parameter
        . <(json_env --export)
    fi
}

# Run the json_env_hook function before displaying the prompt
PROMPT_COMMAND="json_env_hook"