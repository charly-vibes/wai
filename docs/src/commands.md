# Commands

Complete CLI reference for wai.

## Initialization

| Command | Description |
|---------|-------------|
| `wai init` | Initialize wai in current directory |

## Creating artifacts

| Command | Description |
|---------|-------------|
| `wai new project <name>` | Create a new project |
| `wai new area <name>` | Create a new area |
| `wai new resource <name>` | Create a new resource |
| `wai add research <content>` | Add research notes to current project |
| `wai add plan <content>` | Add a plan document |
| `wai add design <content>` | Add a design document |

## Viewing & navigating

| Command | Description |
|---------|-------------|
| `wai show [name]` | Show PARA overview or item details |
| `wai move <item> <category>` | Move item between PARA categories |
| `wai search <query>` | Search across all artifacts |
| `wai timeline <project>` | View chronological project timeline |
| `wai status` | Show project status with suggestions |

## Project phases

| Command | Description |
|---------|-------------|
| `wai phase` | Show current project phase |
| `wai phase next` | Advance to next phase |
| `wai phase back` | Return to previous phase |
| `wai phase set <phase>` | Set a specific phase |

## Agent config

| Command | Description |
|---------|-------------|
| `wai sync` | Sync agent configs to tool-specific locations |
| `wai config add <type> <file>` | Add an agent config file |
| `wai config list` | List agent config files |
| `wai config edit` | Open config in editor |

## Handoffs

| Command | Description |
|---------|-------------|
| `wai handoff create <project>` | Generate a handoff document |

## Plugins

| Command | Description |
|---------|-------------|
| `wai plugin list` | List detected plugins |

## Import

| Command | Description |
|---------|-------------|
| `wai import <path>` | Import existing tool configs |
