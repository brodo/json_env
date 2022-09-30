# json_env

`json_env` is [dotenv](https://github.com/motdotla/dotenv), but with JSON.
`json_env` loads an environment variables from a file called `.env.json` in the current directory and starts a subprocess
with them. 
Storing configuration in the environment separate from code is based on [The Twelve-Factor](http://12factor.net/config) App methodology.

## How to install

With homebrew:
```shell
$ brew tap brodo/json_env
$ brew install json_env
```

With NPM
```shell
$ npm i -g @brodo/json_env
```

With cargo:
```shell
$ cargo install json_env
```

Or download the binaries for your platform on the [releases page](https://github.com/brodo/json_env/releases/) and
put them in your $PATH.

## How to use

Just run json_env with any program as a parameter: 
```shell
$ json_env my_program
```

Additional command line arguments that are passed to `json_env` are forwarded to the child process:
```shell
$ json_env echo "Test"

Test
```

### Example
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

## License

json_env is licensed under the Apache 2.0 license.

