# ShFS

ShFS is a shared network filesystem.

## Notes
* The Project is **not yet** stable nor feature complete.
* Contributions are welcome.

## Goals
* ⚡ Native Speed (Caching, FUSE, Compression)
* 🔒 Secure (Encryption)
* 📦 Multiple Shares (Volumes)
* 🐳 Dockerized Deployment

## Building
### Building:
```cargo build --release```
### Building without FUSE:
```cargo build --release --no-default-features```

## Docker
### Building:
```docker-compose build```
### Exported Volumes:
* /config
* /volumes
### Usage:
Place your config in `/config/config.json` and forward a port to 30 inside the container

## Configuration

The config file is written in JSON

Example: config.json:
```
{
	"name": "MyServer",
	"volumes": [
		{
			"name": "First Vol",
			"root": "/volumes/1"
		},
		{
			"name": "Second Vol",
			"root": "/volumes/2",
			"readonly": true
		}
	]
}
```
### Possible Top Level Values:
* `name` : Optional : Name of the Server : Default=None
* `volumes` : Required : List of Volumes

### Volumes are additional JSON Objects with these possible values:
* `name` - Optional : Name of the Volume : Default: If nothing is provided `name` is the basename of the root path
* `description` - Optional : Description of the Volume : Default=None
* `root` - Required : Root Path of the Volume
* `discoverable` - Optional : If set to `false` the Volume will not show up in `shfs list` : Default=`true`
* `public` - Optional : If set to `true` the Volume is accessable to everyone : Default=`true`
* `trash_enabled` - Optional : Enabled the Trash Feature : Default=`false`
* `readonly` - Optional : Makes the Volume Read Only : Default=`false`
