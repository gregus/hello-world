# Cesium Globe with Next.js

A Next.js project featuring a 3D interactive globe visualization using Cesium.js.

## Features

- **Cesium.js Integration**: Full 3D globe visualization with real terrain data
- **Next.js 15**: Built with the latest Next.js App Router
- **TypeScript**: Fully typed for better development experience
- **Interactive Controls**: Pan, zoom, and rotate the globe
- **World Terrain**: Includes Cesium's world terrain for realistic topography

## Getting Started

### Prerequisites

- Node.js 18+ and npm

### Installation

The dependencies are already installed. If you need to reinstall:

```bash
npm install
```

### Development

Run the development server:

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to see the 3D globe.

### Build

Build the application for production:

```bash
npm run build
```

Start the production server:

```bash
npm start
```

## Project Structure

```
├── app/
│   ├── page.tsx          # Main page displaying the globe
│   ├── layout.tsx        # Root layout
│   └── globals.css       # Global styles
├── components/
│   └── CesiumGlobe.tsx   # Cesium globe component
└── next.config.ts        # Next.js configuration with Cesium webpack setup
```

## Cesium Configuration

The project is configured to work with Cesium.js by:

1. **Webpack Configuration**: Custom webpack setup to handle Cesium's AMD modules and static assets
2. **Static Assets**: Cesium Workers, Widgets, Assets, and ThirdParty files are copied to the build
3. **Access Token**: Using a default Cesium Ion token (replace with your own for production)

### Getting Your Own Cesium Ion Token

For production use, get your free token from [Cesium Ion](https://cesium.com/ion/):

1. Sign up for a free account
2. Go to your dashboard and create an access token
3. Replace the token in `components/CesiumGlobe.tsx`

## Globe Controls

- **Left Click + Drag**: Rotate the globe
- **Right Click + Drag**: Pan the camera
- **Scroll Wheel**: Zoom in/out
- **Middle Click + Drag**: Tilt the camera

## Learn More

- [Cesium Documentation](https://cesium.com/learn/cesiumjs-learn/)
- [Next.js Documentation](https://nextjs.org/docs)
- [Cesium Sandcastle](https://sandcastle.cesium.com/) - Interactive examples

## License

This project uses Cesium.js which is Apache 2.0 licensed.
