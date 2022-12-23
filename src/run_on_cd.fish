
function json_env_hook
    # Check if the current directory contains a file called ".env.json"
    if test -f ".env.json"
        # Execute the "json_env" program with the "--export" parameter
        # and "source" the output in the current shell environment
        . (json_env --export)
    end
end

function json_env_postexec
    # Check if the previous command was the builtin cd function
    if test $prev_cmd = "cd"
        # Call the json_env_hook function
        json_env_hook
    end
end

# Use the fish_postexec function to run the json_env_postexec function
# after each command is executed
fish_postexec json_env_postexec

