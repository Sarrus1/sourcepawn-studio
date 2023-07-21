<div align="center">
  <h1><code>${plugin_name}</code></h1>
  <p>
    <strong>Short Description</strong>
  </p>
  <p style="margin-bottom: 0.5ex;">
    <img
        src="https://img.shields.io/github/downloads/${GithubName}/${plugin_name}/total"
    />
    <img
        src="https://img.shields.io/github/last-commit/${GithubName}/${plugin_name}"
    />
    <img
        src="https://img.shields.io/github/issues/${GithubName}/${plugin_name}"
    />
    <img
        src="https://img.shields.io/github/issues-closed/${GithubName}/${plugin_name}"
    />
    <img
        src="https://img.shields.io/github/repo-size/${GithubName}/${plugin_name}"
    />
    <img
        src="https://img.shields.io/github/workflow/status/${GithubName}/${plugin_name}/Compile%20and%20release"
    />
  </p>
</div>


## Requirements ##
- Sourcemod and Metamod


## Installation ##
1. Grab the latest release from the release page and unzip it in your sourcemod folder.
2. Restart the server or type `sm plugins load ${plugin_name}` in the console to load the plugin.
3. The config file will be automatically generated in cfg/sourcemod/

## Configuration ##
- You can modify the phrases in addons/sourcemod/translations/${plugin_name}.phrases.txt.
- Once the plugin has been loaded, you can modify the cvars in cfg/sourcemod/${plugin_name}.cfg.


## Usage ##
