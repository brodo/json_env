
# Define the json_env_hook function
function json_env_hook() {
    # Check if the current directory contains a file called ".env.json"
    if [ -f ".env.json" ]; then
        # Execute the "json_env" program with the "--export" parameter
        . <(json_env --export)
    fi
}

# Use the add-zsh-hook function to run the json_env_hook function
# whenever the chpwd event is triggered
add-zsh-hook chpwd json_env_hook