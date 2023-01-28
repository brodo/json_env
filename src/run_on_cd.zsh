# Define the json_env_hook function
function json_env_hook() {
  # Save the exit code and the output of "json_env --print-config-path" in a variable
  local json_env_output
  json_env_output="$(json_env --print-config-path)"
  local json_env_exit_code=$?

  # If the exit code is 0, that means that we are in a folder that contains a ".env.json"
  # file or has a parent folder that contains a ".env.json" file
  if [ $json_env_exit_code -eq 0 ]; then
    # Save the exit code and the exit code of "json_env --is-whitelisted" in a variable
    (json_env --is-whitelisted > /dev/null)
    local json_env_whitelisted_exit_code
    json_env_whitelisted_exit_code=$?

    # If the exit code is 0, that means that the current folder is whitelisted
    if [ $json_env_whitelisted_exit_code -eq 0 ]; then
      # Source the output of executing the "json_env" program with the "--export" parameter
      eval "$(json_env --export)"
    else
      echo "json_env: The config file at $json_env_output is not whitelisted. Run 'json_env --whitelist' to whitelist it."
    fi
  fi
}

# Use the add-zsh-hook function to run the json_env_hook function
# whenever the chpwd event is triggered
add-zsh-hook chpwd json_env_hook
