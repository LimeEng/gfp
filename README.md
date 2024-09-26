# gfp

Grafana does not yet support the automatic provisioning of public dashboards. The URL to a public dashboard is randomly generated, meaning that each time Grafana is set up with provisioned dashboards, manual intervention is required to obtain and possibly update the public URL of the dashboard. This application automates the process of making dashboards public and retrieving their URLs.

As of right now, there is no versioning. Only a single image is available at any time.

Please note that template variables are as of yet not supported! Here is the issue [tracking any progress](https://github.com/grafana/grafana/issues/67346)

## Usage

The easiest way to set up **gfp** is to use Docker Compose. Here is an example configuration:

```yaml
name: gfp

services:
  gfp:
    image: ghcr.io/limeeng/gfp:latest
    container_name: gfp
    restart: unless-stopped
    environment:
      GFP_GRAFANA_DOMAIN: ${GFP_GRAFANA_DOMAIN}     # required
      GFP_GRAFANA_USERNAME: ${GFP_GRAFANA_USERNAME} # required
      GFP_GRAFANA_PASSWORD: ${GFP_GRAFANA_PASSWORD} # required
      RUST_LOG: info           # optional
      # GFP_PORT: 8080         # optional
      # GFP_CACHE_SECONDS: 300 # optional
    expose:
      - 8080
```
Create a `.env` file in the same directory as your `compose.yaml` to define your environment variables:

```
GFP_GRAFANA_DOMAIN=https://your-grafana-instance
GFP_GRAFANA_USERNAME=your-username
GFP_GRAFANA_PASSWORD=your-password
```
