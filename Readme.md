# json_env

`json_env` is [dotenv](https://github.com/motdotla/dotenv), but with JSON.
`json_env` loads an environment variables from a file called `.env.json` in the current directory and starts a subprocess
with them. 
Storing configuration in the environment separate from code is based on [The Twelve-Factor](http://12factor.net/config) App methodology.

.env.json:
```json
{
    "NODE_ENV": "DEV",
    "MY_USER": "Carl",
    "NUM_USERS": 10,
    "nested": {
        "hello": "world",
        "boo": "far"
    }
}
```

Shell:
```shell
$ json_env env

MY_USER=Carl
NODE_ENV=DEV
NUM_USERS=10
nested={"boo":"far","hello":"world"}
[...]
```

Additional command line arguments that are passed to `json_env` are forwarded to the child process:

Shell:
```shell

$ json_env echo "Test"

Test
```

