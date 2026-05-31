<div align="center">  
  <h1>🌌 Galaxy</h1>
</div>

Galaxy is a Rust program using the Vulkan API to render an animated 2D space
simulation. The scene is built entirely from animated point sprites and includes:

- Tens of thousands of twinkling background **stars** (clustered + spiral distributions)
- Distant rotating **galaxies** with bright cores and spiral or elliptical structure
- Soft, drifting **nebulae**
- **Comets** with glowing comae and tapering tails
- **Supernovae** that periodically flash and throw off an expanding shockwave
- Rhythmically pulsing **pulsars**
- **Black holes** — dark event-horizon disks that occlude the sky behind them,
  ringed by a glowing, slowly rotating accretion disk
- Streaking **shooting stars**

All animation is driven on the GPU from a single `time` push constant, with each
object's behavior selected by a per-vertex `kind` tag.

<div align="center">  
  <img src="./assets/preview.gif" alt="preview"/>
</div>
